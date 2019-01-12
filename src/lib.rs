#[macro_use]
extern crate simple_error;

#[macro_use]
extern crate lazy_static;

pub mod rom_map;

use std::collections::HashMap;
use std::error::Error;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Stats {
    base: u8,
    mult: u8,
    rate: u8,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Speed {
    min: u8,
    max: u8,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Status {
    ImmuneToElements,
    AbsorbsElements,
    ResistsElements,
    Light,
    Dark,
    Lightning,
    Ice,
    Fire,
    Death,
    Stone,
    Toad,
    Tiny,
    Piggy,
    Mute,
    Blind,
    Poison,
    Curse,
    Float,
    Paralyze,
    Sleep,
    Charm,
    Berserk,
    Petrify,
    D,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Weakness {
    Damage4x,
    Floating,
    SpearsArrow,
    Light,
    Dark,
    Lightning,
    Ice,
    Fire,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CreatureType {
    Undead,
    Mage,
    Slime,
    Giant,
    Spirit,
    Reptile,
    Machine,
    Dragon,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DropTable {
    common: u8,
    uncommon: u8,
    rare: u8,
    very_rare: u8,
}

#[derive(Debug, Default, PartialEq)]
pub struct Monster {
    name: String,
    xp: u16,
    gp: u16,
    is_boss: bool,
    level: u8,
    max_hp: u16,
    physical_attack: Stats,
    physical_defense: Stats,
    magical_defense: Stats,
    speed: Speed,
    drop_rate: u8,
    drop_table: DropTable,
    attack_seq_group: u8,
    attack_statuses: Vec<Status>,
    defense_statuses: Vec<Status>,
    weaknesses: Vec<Weakness>,
    spell_power: u8,
    creature_types: Vec<CreatureType>,
    reflex_attack_seq: u8,
}

#[derive(Debug, Default)]
pub struct Ff4 {
    monsters: Vec<Monster>,
}

pub fn parse_rom(data: &Vec<u8>) -> Result<Ff4, Box<Error>> {
    let mut name_table = Vec::new();
    let gp_table: Vec<u16>;
    let xp_table: Vec<u16>;
    let monster_offset_table: Vec<u16>;
    let mut stat_table = Vec::new();
    let mut speed_table = Vec::new();
    let mut drop_table = Vec::new();

    for addr in (rom_map::MONSTER_NAME_TABLE..rom_map::MONSTER_NAME_TABLE_END).step_by(8) {
        name_table.push(decode_string(&data[addr..addr + 8])?);
    }

    gp_table = parse_u16_table(
        &data[rom_map::MONSTER_GP_TABLE..],
        rom_map::MONSTER_GP_TABLE_ENTRIES,
    );
    xp_table = parse_u16_table(
        &data[rom_map::MONSTER_XP_TABLE..],
        rom_map::MONSTER_XP_TABLE_ENTRIES,
    );
    monster_offset_table = parse_u16_table(
        &data[rom_map::MONSTER_OFFSET_TABLE..],
        rom_map::MONSTER_OFFSET_TABLE_ENTRIES,
    );

    for addr in (rom_map::MONSTER_STAT_TABLE..rom_map::MONSTER_STAT_TABLE_END).step_by(3) {
        stat_table.push(parse_stats(&data[addr..]));
    }

    for addr in (rom_map::MONSTER_SPEED_TABLE..rom_map::MONSTER_SPEED_TABLE_END).step_by(2) {
        speed_table.push(Speed {
            min: data[addr],
            max: data[addr + 1],
        });
    }

    for addr in (rom_map::MONSTER_DROP_TABLE..rom_map::MONSTER_DROP_TABLE_END).step_by(4) {
        drop_table.push(parse_drop_table(&data[addr..]));
    }

    let mut ff4 = Ff4::default();
    for (index, &offset) in monster_offset_table.iter().enumerate() {
        let addr = rom_map::MONSTER_INFO_OFFSET + (offset as usize);
        let mut monster = parse_monster(&data[addr..], &stat_table, &speed_table, &drop_table);
        monster.name = name_table[index].clone();
        monster.gp = gp_table[index].clone();
        monster.xp = xp_table[index].clone();

        ff4.monsters.push(monster);
    }

    Ok(ff4)
}

fn parse_u16(data: &[u8]) -> u16 {
    (data[0] as u16) + ((data[1] as u16) << 8)
}

fn parse_u16_table(data: &[u8], entries: usize) -> Vec<u16> {
    let mut values = Vec::new();
    for i in 0..entries {
        values.push(parse_u16(&data[i * 2..]))
    }

    values
}

fn parse_stats(data: &[u8]) -> Stats {
    Stats {
        mult: data[0],
        rate: data[1],
        base: data[2],
    }
}

fn is_bit_set(data: u8, bit: u8) -> bool {
    assert!(bit < 8);
    (data >> bit) & 0x1 == 0x1
}

fn parse_status(data: &[u8], statuses: &mut Vec<Status>) {
    if is_bit_set(data[0], 7) {
        statuses.push(Status::ImmuneToElements);
    }
    if is_bit_set(data[0], 6) {
        statuses.push(Status::AbsorbsElements);
    }
    if is_bit_set(data[0], 5) {
        statuses.push(Status::ResistsElements);
    }
    if is_bit_set(data[0], 4) {
        statuses.push(Status::Light);
    }
    if is_bit_set(data[0], 3) {
        statuses.push(Status::Dark);
    }
    if is_bit_set(data[0], 2) {
        statuses.push(Status::Lightning);
    }
    if is_bit_set(data[0], 1) {
        statuses.push(Status::Ice);
    }
    if is_bit_set(data[0], 0) {
        statuses.push(Status::Fire);
    }

    if is_bit_set(data[1], 7) {
        statuses.push(Status::Death);
    }
    if is_bit_set(data[1], 6) {
        statuses.push(Status::Stone);
    }
    if is_bit_set(data[1], 5) {
        statuses.push(Status::Toad);
    }
    if is_bit_set(data[1], 4) {
        statuses.push(Status::Tiny);
    }
    if is_bit_set(data[1], 3) {
        statuses.push(Status::Piggy);
    }
    if is_bit_set(data[1], 2) {
        statuses.push(Status::Mute);
    }
    if is_bit_set(data[1], 1) {
        statuses.push(Status::Blind);
    }
    if is_bit_set(data[1], 0) {
        statuses.push(Status::Poison);
    }

    if is_bit_set(data[2], 7) {
        statuses.push(Status::Curse);
    }
    if is_bit_set(data[2], 6) {
        statuses.push(Status::Float);
    }
    if is_bit_set(data[2], 5) {
        statuses.push(Status::Paralyze);
    }
    if is_bit_set(data[2], 4) {
        statuses.push(Status::Sleep);
    }
    if is_bit_set(data[2], 3) {
        statuses.push(Status::Charm);
    }
    if is_bit_set(data[2], 2) {
        statuses.push(Status::Berserk);
    }
    if is_bit_set(data[2], 1) {
        statuses.push(Status::Petrify);
    }
    if is_bit_set(data[2], 0) {
        statuses.push(Status::D);
    }
}

fn parse_weakness(data: &[u8], weaknesses: &mut Vec<Weakness>) {
    if is_bit_set(data[0], 7) {
        weaknesses.push(Weakness::Damage4x);
    }
    if is_bit_set(data[0], 6) {
        weaknesses.push(Weakness::Floating);
    }
    if is_bit_set(data[0], 5) {
        weaknesses.push(Weakness::SpearsArrow);
    }
    if is_bit_set(data[0], 4) {
        weaknesses.push(Weakness::Light);
    }
    if is_bit_set(data[0], 3) {
        weaknesses.push(Weakness::Dark);
    }
    if is_bit_set(data[0], 2) {
        weaknesses.push(Weakness::Lightning);
    }
    if is_bit_set(data[0], 1) {
        weaknesses.push(Weakness::Ice);
    }
    if is_bit_set(data[0], 0) {
        weaknesses.push(Weakness::Fire);
    }
}

fn parse_creature_types(data: &[u8], types: &mut Vec<CreatureType>) {
    if is_bit_set(data[0], 7) {
        types.push(CreatureType::Undead);
    }
    if is_bit_set(data[0], 6) {
        types.push(CreatureType::Mage);
    }
    if is_bit_set(data[0], 5) {
        types.push(CreatureType::Slime);
    }
    if is_bit_set(data[0], 4) {
        types.push(CreatureType::Giant);
    }
    if is_bit_set(data[0], 3) {
        types.push(CreatureType::Spirit);
    }
    if is_bit_set(data[0], 2) {
        types.push(CreatureType::Reptile);
    }
    if is_bit_set(data[0], 1) {
        types.push(CreatureType::Machine);
    }
    if is_bit_set(data[0], 0) {
        types.push(CreatureType::Dragon);
    }
}

fn parse_drop_table(data: &[u8]) -> DropTable {
    DropTable {
        common: data[0],
        uncommon: data[1],
        rare: data[2],
        very_rare: data[3],
    }
}

fn parse_monster(
    data: &[u8],
    attack_table: &Vec<Stats>,
    speed_table: &Vec<Speed>,
    drop_table: &Vec<DropTable>,
) -> Monster {
    let mut monster = Monster::default();

    if (data[0] & 0x80) == 0x80 {
        monster.is_boss = true;
    }
    monster.level = data[0] & 0x7f;
    monster.max_hp = parse_u16(&data[1..]);

    monster.physical_attack = attack_table[data[3] as usize].clone();
    monster.physical_defense = attack_table[data[4] as usize].clone();
    monster.magical_defense = attack_table[data[5] as usize].clone();
    monster.speed = speed_table[(data[6] & 0x3f) as usize].clone();

    monster.drop_rate = match data[7] >> 6 {
        0b00 => 0,
        0b01 => 5,
        0b10 => 25,
        _ => 100,
    };
    if monster.drop_rate > 0 {
        monster.drop_table = drop_table[(data[7] & 0x3f) as usize].clone();
    }

    monster.attack_seq_group = data[8];
    let ext_byte_flags = data[9];
    let mut index = 10;

    if is_bit_set(ext_byte_flags, 7) {
        parse_status(&data[index..], &mut monster.attack_statuses);
        index += 3;
    }

    if is_bit_set(ext_byte_flags, 6) {
        parse_status(&data[index..], &mut monster.defense_statuses);
        index += 3;
    }

    if is_bit_set(ext_byte_flags, 5) {
        parse_weakness(&data[index..], &mut monster.weaknesses);
        index += 1;
    }

    if is_bit_set(ext_byte_flags, 4) {
        monster.spell_power = data[index];
        index += 1;
    }

    if is_bit_set(ext_byte_flags, 3) {
        parse_creature_types(&data[index..], &mut monster.creature_types);
        index += 1;
    }

    if is_bit_set(ext_byte_flags, 2) {
        monster.reflex_attack_seq = data[index];
    }

    monster
}

lazy_static! {
    static ref SPECIAL_CHARS: HashMap<u8, &'static str> = {
        let mut map = HashMap::new();

        map.insert(0x1c, "<1c>");
        map.insert(0x25, "<mute>");
        map.insert(0x3a, "<whip>");

        map.insert(0x76, "<flat m>");
        map.insert(0x77, "<flat h>");
        map.insert(0x78, "<flat p>");

        map.insert(0x8a, " t");
        map.insert(0x8b, "th");
        map.insert(0x8c, "he");
        map.insert(0x8d, "t ");
        map.insert(0x8e, "ou");

        map.insert(0x90, " a");
        map.insert(0x91, "s ");
        map.insert(0x92, "er");
        map.insert(0x93, "in");
        map.insert(0x94, "re");
        map.insert(0x95, "d ");
        map.insert(0x96, "an");
        map.insert(0x97, " o");
        map.insert(0x98, "on");
        map.insert(0x99, "st");
        map.insert(0x9a, " w");
        map.insert(0x9b, "o ");
        map.insert(0x9c, " m");
        map.insert(0x9d, "ha");
        map.insert(0x9e, "to");
        map.insert(0x9f, "is");

        map.insert(0xa0, "yo");
        map.insert(0xa1, " y");
        map.insert(0xa2, " i");
        map.insert(0xa3, "al");
        map.insert(0xa4, "ar");
        map.insert(0xa5, " h");
        map.insert(0xa6, "r ");
        map.insert(0xa7, " s");
        map.insert(0xa8, "at");
        map.insert(0xa9, "n ");
        map.insert(0xaa, " c");
        map.insert(0xab, "ng");
        map.insert(0xac, "ce");
        map.insert(0xad, "ll");
        map.insert(0xae, "y ");
        map.insert(0xaf, "nd");

        map.insert(0xb0, "en");
        map.insert(0xb1, "ed");
        map.insert(0xb2, "hi");
        map.insert(0xb3, "or");
        map.insert(0xb4, ", ");
        map.insert(0xb5, "I ");
        map.insert(0xb6, "u ");
        map.insert(0xb7, "me");
        map.insert(0xb8, "ta");
        map.insert(0xb9, " b");
        map.insert(0xba, " I");
        map.insert(0xbb, "te");
        map.insert(0xbc, "of");
        map.insert(0xbd, "ea");
        map.insert(0xbe, "ur");
        map.insert(0xbf, "l ");

        map.insert(0xc0, "'");
        map.insert(0xc1, ".");
        map.insert(0xc2, "-");
        map.insert(0xc3, "…");
        map.insert(0xc4, "!");
        map.insert(0xc5, "?");
        map.insert(0xc6, "%");
        map.insert(0xc7, "/");
        map.insert(0xc8, ":");
        map.insert(0xc9, ",");

        map.insert(0xca, " f");
        map.insert(0xcb, " d");
        map.insert(0xcc, "ow");
        map.insert(0xcd, "se");
        map.insert(0xce, "  ");
        map.insert(0xcf, "it");

        map.insert(0xd0, "et");
        map.insert(0xd1, "le");
        map.insert(0xd2, "f ");
        map.insert(0xd3, " g");
        map.insert(0xd4, "es");
        map.insert(0xd5, "ro");
        map.insert(0xd6, "ne");
        map.insert(0xd7, "ry");
        map.insert(0xd8, " l");
        map.insert(0xd9, "us");
        map.insert(0xda, "no");
        map.insert(0xdb, "ut");
        map.insert(0xdc, "ca");
        map.insert(0xdd, "as");
        map.insert(0xde, "Th");
        map.insert(0xdf, "ai");

        map.insert(0xe0, "ot");
        map.insert(0xe1, "be");
        map.insert(0xe2, "el");
        map.insert(0xe3, "om");
        map.insert(0xe4, "'s");
        map.insert(0xe5, "il");
        map.insert(0xe6, "de");
        map.insert(0xe7, "gh");
        map.insert(0xe8, "ay");
        map.insert(0xe9, "nt");
        map.insert(0xea, "Wh");
        map.insert(0xeb, "Yo");
        map.insert(0xec, "wa");
        map.insert(0xed, "oo");
        map.insert(0xee, "We");
        map.insert(0xef, "g ");

        map.insert(0xf0, "ge");
        map.insert(0xf1, " n");
        map.insert(0xf2, "ee");
        map.insert(0xf3, "wi");
        map.insert(0xf4, " M");
        map.insert(0xf5, "ke");
        map.insert(0xf6, "we");
        map.insert(0xf7, " p");
        map.insert(0xf8, "ig");
        map.insert(0xf9, "ys");
        map.insert(0xfa, " B");
        map.insert(0xfb, "am");
        map.insert(0xfc, "ld");
        map.insert(0xfd, " W");
        map.insert(0xfe, "la");
        map.insert(0xff, " ");

        map
    };
}

fn decode_string(data: &[u8]) -> Result<String, Box<Error>> {
    let mut s = String::from("");

    for &b in data {
        if 0x42 <= b && b <= 0x5b {
            s.push((b - 0x42 + ('A' as u8)).into());
        } else if 0x5c <= b && b <= 0x75 {
            s.push((b - 0x5c + ('a' as u8)).into());
        } else if 0x80 <= b && b <= 0x89 {
            s.push((b - 0x80 + ('0' as u8)).into());
        } else if SPECIAL_CHARS.contains_key(&b) {
            s.push_str(SPECIAL_CHARS[&b]);
        } else {
            bail!("unknown char {:x}", b);
        }
    }

    Ok(s)
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Read;
    use std::path::PathBuf;

    use super::*;

    fn blank_stat_table() -> Vec<Stats> {
        let size = 0xe0;
        let mut table = Vec::with_capacity(0x40);

        for _ in 0..size {
            table.push(Stats::default());
        }

        table
    }

    fn blank_speed_table() -> Vec<Speed> {
        let size = 0x40;
        let mut table = Vec::with_capacity(0x40);

        for _ in 0..size {
            table.push(Speed::default());
        }

        table
    }

    fn blank_drop_table() -> Vec<DropTable> {
        let size = 0x40;
        let mut table = Vec::with_capacity(0x40);

        for _ in 0..size {
            table.push(DropTable::default());
        }

        table
    }

    #[test]
    fn decode_string_test() {
        assert_eq!(
            "Naga    ",
            decode_string(&[0x4f, 0x5c, 0x62, 0x5c, 0xff, 0xff, 0xff, 0xff]).unwrap()
        );
    }

    #[test]
    fn parse_u16_test() {
        assert_eq!(0xaa55, parse_u16(&[0x55, 0xaa]));
    }

    #[test]
    fn parse_u16_table_test() {
        assert_eq!(
            vec!(0xaa55, 0xff00),
            parse_u16_table(&[0x55, 0xaa, 0x00, 0xff], 2)
        );
    }

    #[test]
    fn parse_stats_test() {
        assert_eq!(
            Stats {
                base: 0x55,
                mult: 0xaa,
                rate: 0xff,
            },
            parse_stats(&[0xaa, 0xff, 0x55])
        );
    }

    #[test]
    fn parse_drop_table_test() {
        assert_eq!(
            DropTable {
                common: 0x00,
                uncommon: 0x55,
                rare: 0xaa,
                very_rare: 0xff,
            },
            parse_drop_table(&[0x00, 0x55, 0x0aa, 0xff])
        );
    }

    #[test]
    fn parse_monster_test() {
        let mut stat_table = blank_stat_table();
        stat_table[0x01] = Stats {
            mult: 0x1,
            rate: 0x4b,
            base: 0x13,
        };
        stat_table[0x16] = Stats {
            mult: 0x3,
            rate: 0x63,
            base: 0x2c,
        };
        stat_table[0x60] = Stats {
            mult: 0x0,
            rate: 0x0,
            base: 0x0,
        };
        stat_table[0x6B] = Stats {
            mult: 0x01,
            rate: 0x3c,
            base: 0x01,
        };
        stat_table[0xA0] = Stats {
            mult: 0x0,
            rate: 0x0,
            base: 0x0,
        };
        stat_table[0xC0] = Stats {
            mult: 0x06,
            rate: 0x28,
            base: 0x16,
        };

        let mut speed_table = blank_speed_table();
        speed_table[0x02] = Speed {
            min: 0x01,
            max: 0x02,
        };
        speed_table[0x32] = Speed {
            min: 0x09,
            max: 0x09,
        };

        let mut drop_table = blank_drop_table();
        drop_table[0x38] = DropTable {
            common: 0xce,
            uncommon: 0xe2,
            rare: 0xcf,
            very_rare: 0xe7,
        };

        assert_eq!(
            Monster {
                name: "".to_string(),
                xp: 0,
                gp: 0,
                is_boss: false,
                level: 3,
                max_hp: 6,
                physical_attack: Stats {
                    base: 19,
                    mult: 1,
                    rate: 75
                },
                physical_defense: Stats {
                    base: 0,
                    mult: 0,
                    rate: 0
                },
                magical_defense: Stats {
                    base: 0,
                    mult: 0,
                    rate: 0
                },
                speed: Speed { min: 1, max: 2 },
                drop_rate: 5,
                drop_table: DropTable {
                    common: 0xce,
                    uncommon: 0xe2,
                    rare: 0xcf,
                    very_rare: 0xe7,
                },
                attack_seq_group: 0,
                attack_statuses: vec!(),
                defense_statuses: vec!(),
                weaknesses: vec!(),
                spell_power: 0x0,
                creature_types: vec!(),
                reflex_attack_seq: 0x0,
            },
            parse_monster(
                &[0x03, 0x06, 0x00, 0x01, 0x60, 0xa0, 0x02, 0x78, 0x00, 0x00],
                &stat_table,
                &speed_table,
                &drop_table
            )
        );
        assert_eq!(
            Monster {
                name: "".to_string(),
                xp: 0,
                gp: 0,
                is_boss: true,
                level: 15,
                max_hp: 3000,
                physical_attack: Stats {
                    base: 44,
                    mult: 3,
                    rate: 99
                },
                physical_defense: Stats {
                    base: 1,
                    mult: 1,
                    rate: 60
                },
                magical_defense: Stats {
                    base: 22,
                    mult: 6,
                    rate: 40
                },
                speed: Speed { min: 9, max: 9 },
                drop_rate: 0,
                drop_table: DropTable::default(),
                attack_seq_group: 149,
                attack_statuses: vec!(Status::Poison),
                defense_statuses: vec!(Status::AbsorbsElements, Status::Ice),
                weaknesses: vec!(Weakness::SpearsArrow, Weakness::Light, Weakness::Fire),
                spell_power: 31,
                creature_types: vec!(CreatureType::Undead),
                reflex_attack_seq: 0,
            },
            parse_monster(
                &[
                    0x8F, 0xB8, 0x0B, 0x16, 0x6B, 0xC0, 0x32, 0x00, 0x95, 0xF8, 0x00, 0x01, 0x00,
                    0x42, 0x00, 0x00, 0x31, 0x1F, 0x80
                ],
                &stat_table,
                &speed_table,
                &drop_table
            )
        );
    }

    #[cfg_attr(feature = "ci_tests", ignore)]
    #[test]
    fn parse_rom_test() {
        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push("ff2us.smc");

        let mut f = File::open(p).unwrap();
        let mut data = Vec::new();
        f.read_to_end(&mut data).unwrap();

        let ff4 = parse_rom(&data).unwrap();

        assert_eq!(
            Monster {
                name: "Milon   ".to_string(),
                xp: 3200,
                gp: 2500,
                is_boss: true,
                level: 15,
                max_hp: 3100,
                physical_attack: Stats {
                    base: 19,
                    mult: 1,
                    rate: 75
                },
                physical_defense: Stats {
                    base: 2,
                    mult: 1,
                    rate: 35
                },
                magical_defense: Stats {
                    base: 0,
                    mult: 0,
                    rate: 0
                },
                speed: Speed { min: 8, max: 8 },
                drop_rate: 0,
                drop_table: DropTable {
                    common: 0,
                    uncommon: 0,
                    rare: 0,
                    very_rare: 0
                },
                attack_seq_group: 145,
                attack_statuses: vec!(),
                defense_statuses: vec!(),
                weaknesses: vec!(),
                spell_power: 14,
                creature_types: vec!(),
                reflex_attack_seq: 146
            },
            ff4.monsters[0xa5]
        );

        assert_eq!(
            Monster {
                name: "Milon Z.".to_string(),
                xp: 4000,
                gp: 3000,
                is_boss: true,
                level: 15,
                max_hp: 3000,
                physical_attack: Stats {
                    base: 44,
                    mult: 3,
                    rate: 99
                },
                physical_defense: Stats {
                    base: 1,
                    mult: 1,
                    rate: 60
                },
                magical_defense: Stats {
                    base: 22,
                    mult: 6,
                    rate: 40
                },
                speed: Speed { min: 9, max: 9 },
                drop_rate: 0,
                drop_table: DropTable::default(),
                attack_seq_group: 149,
                attack_statuses: vec!(Status::Poison),
                defense_statuses: vec!(Status::AbsorbsElements, Status::Ice),
                weaknesses: vec!(Weakness::SpearsArrow, Weakness::Light, Weakness::Fire),
                spell_power: 31,
                creature_types: vec!(CreatureType::Undead),
                reflex_attack_seq: 0,
            },
            ff4.monsters[0xa6]
        );
    }
}
