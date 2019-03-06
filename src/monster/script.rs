use nom::types::CompleteByteSlice;
use nom::{ErrorKind, IResult, Needed, Slice};

use std::error::Error;

#[derive(Debug, PartialEq)]
pub enum Action {
    // 0x00-0x30
    // 0x31-0x5e
    Spell {
        aoe: bool,
        spell: u8,
    },
    // 0x5f-0xbf
    EnemyAbility {
        ability: u8,
    },
    // 0xc0-0xe7
    PlayerCommand {
        command: u8,
    },
    // 0xe8
    ChangeCreatureType {
        t: u8,
    },
    // 0xe9
    ChangePhysicalAttackValue {
        index: u8,
    },
    // 0xea
    ChangePhysicalDefenseValue {
        index: u8,
    },
    // 0xeb
    ChangeMagicalDefenseValue {
        index: u8,
    },
    // 0xec
    ModifySpeed {
        data: u8,
    },
    // 0xed
    SetElementalDefenses {
        defenses: u8,
    },
    // 0xee
    SetSpellPower {
        power: u8,
    },
    // 0xef
    SetWeakness {
        weakness: u8,
    },
    // 0xf0
    SetSprite {
        index: u8,
    },
    // 0xf1
    // 0xf2
    ShowMessage {
        suppress_next: bool,
        index: u8,
    },
    // 0xf3
    ChangeMusic {
        index: u8,
    },
    // 0xf4 0x01
    IncrementConditionFlag,
    // 0xf4 0b1xxxxxxx
    SetConditionFlag {
        value: u8,
    },
    // 0xf5 0b1xxxxxxx
    SetReaction {
        value: u8,
    },
    // 0xf7
    DarkenScreen {
        value: u8,
    },
    // 0xf8
    DebugDisplay {
        value: u8,
    },
    // 0xf9
    Target {
        // TODO: make enum
        value: u8,
    },
    // 0xfb
    ChainInto,
    // 0xfc
    EndChain,
    // 0xfd
    StartChain,
    // 0xfe
    Wait,
}

#[derive(Debug, PartialEq)]
pub struct Script {
    pub actions: Vec<Action>,
}

macro_rules! ctag {
    ($i:expr, $tag:expr) => {
        tag!($i, &[$tag][..])
    };
}

macro_rules! parse_simple_action {
    ($i:expr, $tag:expr, $t:expr) => {
        map!($i, ctag!($tag), |_| $t)
    };
}

macro_rules! parse_simple_action_arg {
    ($i:expr, $tag:expr, $e:expr) => {{
        map!(
            $i,
            do_parse!(ctag!($tag) >> value: take!(1) >> (value[0])),
            $e
        )
    }};
}

macro_rules! parse_range {
    ($i:expr, $min:expr, $max:expr) => {{
        use nom::lib::std::result::Result::*;
        use nom::{need_more, InputTake};
        use nom::{ErrorKind, IResult, Needed};

        // Expand to i32 so we don't run against the ends of u8.
        let min = $min as i32;
        let max = $max as i32;
        let res: IResult<_, _>;
        if $i.len() < 1 {
            res = need_more($i, Needed::Size(1));
        } else if min <= ($i[0] as i32) && ($i[0] as i32) <= max {
            res = Ok($i.take_split(1))
        } else {
            res = Err(nom::Err::Error(nom::Context::Code($i, ErrorKind::Tag)))
        }
        res
    };};
}

named!(parse_single_spell<CompleteByteSlice, Action>,
    do_parse!(
        spell: parse_range!(0x00, 0x30) >>
        (Action::Spell{spell: spell[0], aoe: false})
    ));

named!(parse_aoe_spell<CompleteByteSlice, Action>,
    do_parse!(
        spell: parse_range!(0x31, 0x5e) >>
        (Action::Spell{spell: spell[0] - 0x30, aoe: true})
    ));

named!(parse_enemy_ability<CompleteByteSlice, Action>,
    do_parse!(
        ability: parse_range!(0x5f, 0xbf) >>
        (Action::EnemyAbility{ability: ability[0]})
    ));

named!(parse_player_command<CompleteByteSlice, Action>,
    do_parse!(
        command: parse_range!(0xc0, 0xe7) >>
        (Action::PlayerCommand{command: command[0] - 0xc0})
    ));

fn parse_condition_flag(input: CompleteByteSlice) -> IResult<CompleteByteSlice, Action> {
    if input.len() < 2 {
        return nom::need_more(input, Needed::Size(2));
    }

    let tag = input[0];
    let data = input[1];
    let rest = input.slice(2..);

    if tag != 0xf4 {
        return Err(nom::Err::Error(nom::Context::Code(input, ErrorKind::Tag)));
    }
    if data == 0x01 {
        Ok((rest, Action::IncrementConditionFlag))
    } else if (data & 0x80) == 0x80 {
        Ok((rest, Action::SetConditionFlag { value: data & 0x7f }))
    } else {
        Err(nom::Err::Error(nom::Context::Code(input, ErrorKind::Tag)))
    }
}

fn parse_set_reaction(input: CompleteByteSlice) -> IResult<CompleteByteSlice, Action> {
    if input.len() < 2 {
        return nom::need_more(input, Needed::Size(2));
    }

    let tag = input[0];
    let data = input[1];
    let rest = input.slice(2..);

    if tag != 0xf5 {
        return Err(nom::Err::Error(nom::Context::Code(input, ErrorKind::Tag)));
    }
    if (data & 0x80) == 0x80 {
        Ok((rest, Action::SetReaction { value: data & 0x7f }))
    } else {
        Err(nom::Err::Error(nom::Context::Code(input, ErrorKind::Tag)))
    }
}

named!(parse_action<CompleteByteSlice, Action>, alt!(
    parse_single_spell |
    parse_aoe_spell |
    parse_enemy_ability |
    parse_player_command |
    parse_simple_action_arg!(0xe8, |v| Action::ChangeCreatureType{t: v}) |
    parse_simple_action_arg!(0xe9, |v| Action::ChangePhysicalAttackValue{index: v}) |
    parse_simple_action_arg!(0xea, |v| Action::ChangePhysicalDefenseValue{index: v}) |
    parse_simple_action_arg!(0xeb, |v| Action::ChangeMagicalDefenseValue{index: v}) |
    parse_simple_action_arg!(0xec, |v| Action::ModifySpeed{data: v}) |
    parse_simple_action_arg!(0xed, |v| Action::SetElementalDefenses{defenses: v}) |
    parse_simple_action_arg!(0xee, |v| Action::SetSpellPower{power: v}) |
    parse_simple_action_arg!(0xef, |v| Action::SetWeakness{weakness: v}) |
    parse_simple_action_arg!(0xf0, |v| Action::SetSprite{index: v}) |
    parse_simple_action_arg!(0xf1, |v| Action::ShowMessage{suppress_next: false, index: v}) |
    parse_simple_action_arg!(0xf2, |v| Action::ShowMessage{suppress_next: true, index: v}) |
    parse_simple_action_arg!(0xf3, |v| Action::ChangeMusic{index: v}) |
    parse_condition_flag |
    parse_set_reaction |
    parse_simple_action_arg!(0xf7, |v| Action::DarkenScreen{value: v}) |
    parse_simple_action_arg!(0xf8, |v| Action::DebugDisplay{value: v}) |
    parse_simple_action_arg!(0xf9, |v| Action::Target{value: v}) |

    parse_simple_action!(0xfb, Action::ChainInto) |
    parse_simple_action!(0xfc, Action::EndChain) |
    parse_simple_action!(0xfd, Action::StartChain) |
    parse_simple_action!(0xfe, Action::Wait)
));

named!(parse_script<CompleteByteSlice, Script>, do_parse!(
    a: many_till!(parse_action, ctag!(0xff)) >>
    (Script{actions: a.0})
));

named!(parse_scripts<CompleteByteSlice, Vec<Script>>,
     complete!(many_m_n!(0, 0x100, parse_script)));

pub fn parse(data: &[u8]) -> Result<Vec<Script>, Box<Error>> {
    Ok(parse_scripts(CompleteByteSlice(data))
        .map_err(|x| format!("{}", x))?
        .1)
}

#[cfg(test)]
mod tests {
    use super::super::super::rom_map;
    use super::super::super::test_utils;
    use super::*;

    #[test]
    fn parse_single_spell_test() {
        assert_eq!(
            Action::Spell {
                spell: 0x1,
                aoe: false
            },
            parse_action(CompleteByteSlice(&[0x01])).unwrap().1
        );
        assert_eq!(
            Action::Spell {
                spell: 0x25,
                aoe: false
            },
            parse_action(CompleteByteSlice(&[0x25])).unwrap().1
        );
    }

    #[test]
    fn parse_aoe_spell_test() {
        assert_eq!(
            Action::Spell {
                spell: 0x1,
                aoe: true
            },
            parse_action(CompleteByteSlice(&[0x31])).unwrap().1
        );
        assert_eq!(
            Action::Spell {
                spell: 0x25,
                aoe: true
            },
            parse_action(CompleteByteSlice(&[0x55])).unwrap().1
        );
    }

    #[test]
    fn parse_enemy_ability_test() {
        assert_eq!(
            Action::EnemyAbility { ability: 0x5f },
            parse_action(CompleteByteSlice(&[0x5f])).unwrap().1
        );
        assert_eq!(
            Action::EnemyAbility { ability: 0x70 },
            parse_action(CompleteByteSlice(&[0x70])).unwrap().1
        );
        assert_eq!(
            Action::EnemyAbility { ability: 0xbf },
            parse_action(CompleteByteSlice(&[0xbf])).unwrap().1
        );
    }

    #[test]
    fn parse_player_command_test() {
        assert_eq!(
            Action::PlayerCommand { command: 0x00 },
            parse_action(CompleteByteSlice(&[0xc0])).unwrap().1
        );
        assert_eq!(
            Action::PlayerCommand { command: 0x0f },
            parse_action(CompleteByteSlice(&[0xcf])).unwrap().1
        );
        assert_eq!(
            Action::PlayerCommand { command: 0x27 },
            parse_action(CompleteByteSlice(&[0xe7])).unwrap().1
        );
    }

    #[test]
    fn parse_condition_flags() {
        assert_eq!(
            Action::IncrementConditionFlag,
            parse_action(CompleteByteSlice(&[0xf4, 0x01])).unwrap().1
        );
        assert_eq!(
            Action::SetConditionFlag { value: 0x4f },
            parse_action(CompleteByteSlice(&[0xf4, 0xcf])).unwrap().1
        );
        assert_eq!(
            Err(nom::Err::Error(nom::Context::Code(
                CompleteByteSlice(&[244, 79]),
                ErrorKind::Alt
            ))),
            parse_action(CompleteByteSlice(&[0xf4, 0x4f]))
        );
    }

    #[test]
    fn parse_set_reaction_flags() {
        assert_eq!(
            Action::SetReaction { value: 0x4f },
            parse_action(CompleteByteSlice(&[0xf5, 0xcf])).unwrap().1
        );
        assert_eq!(
            Err(nom::Err::Error(nom::Context::Code(
                CompleteByteSlice(&[245, 79]),
                ErrorKind::Alt
            ))),
            parse_action(CompleteByteSlice(&[0xf5, 0x4f]))
        );
    }

    #[test]
    fn parse_simple_commands_test() {
        assert_eq!(
            Action::ChangeCreatureType { t: 0x0 },
            parse_action(CompleteByteSlice(&[0xe8, 0x00])).unwrap().1
        );
        assert_eq!(
            Action::ChangePhysicalAttackValue { index: 0x1 },
            parse_action(CompleteByteSlice(&[0xe9, 0x01])).unwrap().1
        );
        assert_eq!(
            Action::ChangePhysicalDefenseValue { index: 0x2 },
            parse_action(CompleteByteSlice(&[0xea, 0x02])).unwrap().1
        );
        assert_eq!(
            Action::ChangeMagicalDefenseValue { index: 0x3 },
            parse_action(CompleteByteSlice(&[0xeb, 0x03])).unwrap().1
        );
        assert_eq!(
            Action::ModifySpeed { data: 0x4 },
            parse_action(CompleteByteSlice(&[0xec, 0x04])).unwrap().1
        );
        assert_eq!(
            Action::SetElementalDefenses { defenses: 0x5 },
            parse_action(CompleteByteSlice(&[0xed, 0x05])).unwrap().1
        );
        assert_eq!(
            Action::SetSpellPower { power: 0x6 },
            parse_action(CompleteByteSlice(&[0xee, 0x06])).unwrap().1
        );
        assert_eq!(
            Action::SetWeakness { weakness: 0x7 },
            parse_action(CompleteByteSlice(&[0xef, 0x07])).unwrap().1
        );
        assert_eq!(
            Action::SetSprite { index: 0x8 },
            parse_action(CompleteByteSlice(&[0xf0, 0x08])).unwrap().1
        );
        assert_eq!(
            Action::ShowMessage {
                suppress_next: false,
                index: 0x9
            },
            parse_action(CompleteByteSlice(&[0xf1, 0x09])).unwrap().1
        );
        assert_eq!(
            Action::ShowMessage {
                suppress_next: true,
                index: 0xa
            },
            parse_action(CompleteByteSlice(&[0xf2, 0x0a])).unwrap().1
        );
        assert_eq!(
            Action::ChangeMusic { index: 0xb },
            parse_action(CompleteByteSlice(&[0xf3, 0x0b])).unwrap().1
        );
        assert_eq!(
            Action::DarkenScreen { value: 0xc },
            parse_action(CompleteByteSlice(&[0xf7, 0x0c])).unwrap().1
        );
        assert_eq!(
            Action::DebugDisplay { value: 0xd },
            parse_action(CompleteByteSlice(&[0xf8, 0x0d])).unwrap().1
        );
        assert_eq!(
            Action::Target { value: 0xe },
            parse_action(CompleteByteSlice(&[0xf9, 0x0e])).unwrap().1
        );
        assert_eq!(
            Action::ChainInto,
            parse_action(CompleteByteSlice(&[0xfb])).unwrap().1
        );
        assert_eq!(
            Action::EndChain,
            parse_action(CompleteByteSlice(&[0xfc])).unwrap().1
        );
        assert_eq!(
            Action::StartChain,
            parse_action(CompleteByteSlice(&[0xfd])).unwrap().1
        );
        assert_eq!(
            Action::Wait,
            parse_action(CompleteByteSlice(&[0xfe])).unwrap().1
        );
    }

    #[test]
    fn parse_script_test() {
        assert_eq!(
            Script {
                actions: vec!(
                    Action::EnemyAbility { ability: 111 },
                    Action::Wait,
                    Action::PlayerCommand { command: 0 },
                    Action::Wait,
                    Action::EnemyAbility { ability: 137 },
                    Action::Wait,
                    Action::PlayerCommand { command: 0 }
                )
            },
            parse_script(CompleteByteSlice(&[
                0x6f, 0xfe, 0xc0, 0xfe, 0x89, 0xfe, 0xc0, 0xff
            ]))
            .unwrap()
            .1
        );
    }

    #[test]
    fn parse_scripts_test() {
        assert_eq!(
            vec!(
                Script {
                    actions: vec!(
                        Action::EnemyAbility { ability: 111 },
                        Action::Wait,
                        Action::PlayerCommand { command: 0 },
                        Action::Wait,
                        Action::EnemyAbility { ability: 137 },
                        Action::Wait,
                        Action::PlayerCommand { command: 0 }
                    )
                },
                Script {
                    actions: vec!(
                        Action::PlayerCommand { command: 0 },
                        Action::Wait,
                        Action::PlayerCommand { command: 0 },
                        Action::Wait,
                        Action::EnemyAbility { ability: 152 }
                    )
                },
            ),
            parse_scripts(CompleteByteSlice(&[
                0x6f, 0xfe, 0xc0, 0xfe, 0x89, 0xfe, 0xc0, 0xff, 0xc0, 0xfe, 0xc0, 0xfe, 0x98, 0xff
            ]))
            .unwrap()
            .1
        );
    }

    #[cfg_attr(feature = "ci_tests", ignore)]
    #[test]
    fn parse_test() {
        let data = test_utils::load_rom().unwrap();
        parse(&data[rom_map::AI_EARTH_ATTACK_SCRIPTS_START..=rom_map::AI_EARTH_ATTACK_SCRIPTS_END])
            .unwrap();
        parse(&data[rom_map::AI_MOON_ATTACK_SCRIPTS_START..=rom_map::AI_MOON_ATTACK_SCRIPTS_END])
            .unwrap();
    }

}
