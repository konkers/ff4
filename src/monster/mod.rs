pub mod ai;
pub mod script;

use super::rom_map;
use super::string;

use std::error::Error;

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Stats {
    pub base: u8,
    pub mult: u8,
    pub rate: u8,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Speed {
    pub min: u8,
    pub max: u8,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
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

#[derive(Clone, Debug, PartialEq, Serialize)]
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

#[derive(Clone, Debug, PartialEq, Serialize)]
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

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct DropTable {
    pub common: u8,
    pub uncommon: u8,
    pub rare: u8,
    pub very_rare: u8,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Monster {
    pub index: usize,
    pub is_boss: bool,
    pub level: u8,
    pub max_hp: u16,
    pub physical_attack_index: u8,
    pub physical_defense_index: u8,
    pub magical_defense_index: u8,
    pub speed_index: u8,
    pub drop_rate: u8,
    pub drop_table_index: u8,
    pub attack_seq_group: u8,
    pub attack_statuses: Vec<Status>,
    pub defense_statuses: Vec<Status>,
    pub weaknesses: Vec<Weakness>,
    pub spell_power: u8,
    pub creature_types: Vec<CreatureType>,
    pub reflex_attack_seq: u8,
}

#[derive(Debug, Serialize)]
pub struct MonsterData {
    pub monsters: Vec<Monster>,
    pub name_table: Vec<String>,
    pub gp_table: Vec<u16>,
    pub xp_table: Vec<u16>,
    pub stat_table: Vec<Stats>,
    pub speed_table: Vec<Speed>,
    pub drop_tables: Vec<DropTable>,
    pub ai: ai::Ai,
}

pub fn parse(data: &Vec<u8>) -> Result<MonsterData, Box<Error>> {
    let mut name_table = Vec::new();
    let gp_table: Vec<u16>;
    let xp_table: Vec<u16>;
    let monster_offset_table: Vec<u16>;
    let mut stat_table = Vec::new();
    let mut speed_table = Vec::new();
    let mut drop_table = Vec::new();

    for addr in (rom_map::MONSTER_NAME_TABLE..rom_map::MONSTER_NAME_TABLE_END).step_by(8) {
        name_table.push(string::decode(&data[addr..addr + 8])?);
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

    let mut monsters = Vec::new();
    for (index, &offset) in monster_offset_table.iter().enumerate() {
        let addr = rom_map::MONSTER_INFO_OFFSET + (offset as usize);
        let mut monster = parse_monster(&data[addr..]);
        monster.index = index;

        monsters.push(monster);
    }

    let ai = ai::parse(&data)?;

    Ok(MonsterData {
        monsters: monsters,
        name_table: name_table,
        gp_table: gp_table,
        xp_table: xp_table,
        stat_table: stat_table,
        speed_table: speed_table,
        drop_tables: drop_table,
        ai: ai,
    })
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

fn parse_monster(data: &[u8]) -> Monster {
    let mut monster = Monster::default();

    if (data[0] & 0x80) == 0x80 {
        monster.is_boss = true;
    }
    monster.level = data[0] & 0x7f;
    monster.max_hp = parse_u16(&data[1..]);

    monster.physical_attack_index = data[3];
    monster.physical_defense_index = data[4];
    monster.magical_defense_index = data[5];
    monster.speed_index = data[6] & 0x3f;

    monster.drop_rate = match data[7] >> 6 {
        0b00 => 0,
        0b01 => 5,
        0b10 => 25,
        _ => 100,
    };
    if monster.drop_rate > 0 {
        monster.drop_table_index = data[7] & 0x3f;
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

#[cfg(test)]
mod tests {
    use super::super::test_utils;
    use super::*;

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
        assert_eq!(
            Monster {
                index: 0,
                is_boss: false,
                level: 3,
                max_hp: 6,
                physical_attack_index: 0x01,
                physical_defense_index: 0x60,
                magical_defense_index: 0xa0,
                speed_index: 0x02,
                drop_rate: 5,
                drop_table_index: 0x38,
                attack_seq_group: 0,
                attack_statuses: vec!(),
                defense_statuses: vec!(),
                weaknesses: vec!(),
                spell_power: 0x0,
                creature_types: vec!(),
                reflex_attack_seq: 0x0,
            },
            parse_monster(&[0x03, 0x06, 0x00, 0x01, 0x60, 0xa0, 0x02, 0x78, 0x00, 0x00],)
        );
        assert_eq!(
            Monster {
                index: 0,
                is_boss: true,
                level: 15,
                max_hp: 3000,
                physical_attack_index: 0x16,
                physical_defense_index: 0x6b,
                magical_defense_index: 0xc0,
                speed_index: 0x32,
                drop_rate: 0,
                drop_table_index: 0,
                attack_seq_group: 149,
                attack_statuses: vec!(Status::Poison),
                defense_statuses: vec!(Status::AbsorbsElements, Status::Ice),
                weaknesses: vec!(Weakness::SpearsArrow, Weakness::Light, Weakness::Fire),
                spell_power: 31,
                creature_types: vec!(CreatureType::Undead),
                reflex_attack_seq: 0,
            },
            parse_monster(&[
                0x8F, 0xB8, 0x0B, 0x16, 0x6B, 0xC0, 0x32, 0x00, 0x95, 0xF8, 0x00, 0x01, 0x00, 0x42,
                0x00, 0x00, 0x31, 0x1F, 0x80
            ],)
        );
    }

    #[cfg_attr(feature = "ci_tests", ignore)]
    #[test]
    fn parse_rom_test() {
        let data = test_utils::load_rom().unwrap();
        let ff4 = parse(&data).unwrap();

        let milon = &ff4.monsters[0xa5];
        assert_eq!(
            Monster {
                index: 0xa5,
                is_boss: true,
                level: 15,
                max_hp: 3100,
                physical_attack_index: 0x1,
                physical_defense_index: 0x66,
                magical_defense_index: 0xa0,
                speed_index: 0x33,
                drop_rate: 0,
                drop_table_index: 0x0,
                attack_seq_group: 145,
                attack_statuses: vec!(),
                defense_statuses: vec!(),
                weaknesses: vec!(),
                spell_power: 14,
                creature_types: vec!(),
                reflex_attack_seq: 146
            },
            *milon
        );
        assert_eq!("Milon   ".to_string(), ff4.name_table[milon.index]);
        assert_eq!(3200, ff4.xp_table[milon.index]);
        assert_eq!(2500, ff4.gp_table[milon.index]);
        assert_eq!(
            Stats {
                base: 19,
                mult: 1,
                rate: 75,
            },
            ff4.stat_table[milon.physical_attack_index as usize]
        );
        assert_eq!(
            Stats {
                base: 2,
                mult: 1,
                rate: 35,
            },
            ff4.stat_table[milon.physical_defense_index as usize]
        );
        assert_eq!(
            Stats {
                base: 0,
                mult: 0,
                rate: 0,
            },
            ff4.stat_table[milon.magical_defense_index as usize]
        );
        assert_eq!(
            Speed { min: 8, max: 8 },
            ff4.speed_table[milon.speed_index as usize]
        );

        let milon_z = &ff4.monsters[0xa6];
        assert_eq!(
            Monster {
                index: 0xa6,
                is_boss: true,
                level: 15,
                max_hp: 3000,
                physical_attack_index: 0x16,
                physical_defense_index: 0x6b,
                magical_defense_index: 0xc0,
                speed_index: 0x32,
                drop_rate: 0,
                drop_table_index: 0x0,
                attack_seq_group: 149,
                attack_statuses: vec!(Status::Poison),
                defense_statuses: vec!(Status::AbsorbsElements, Status::Ice),
                weaknesses: vec!(Weakness::SpearsArrow, Weakness::Light, Weakness::Fire),
                spell_power: 31,
                creature_types: vec!(CreatureType::Undead),
                reflex_attack_seq: 0,
            },
            *milon_z
        );
        assert_eq!("Milon Z.".to_string(), ff4.name_table[milon_z.index]);
        assert_eq!(4000, ff4.xp_table[milon_z.index]);
        assert_eq!(3000, ff4.gp_table[milon_z.index]);
        assert_eq!(
            Stats {
                base: 44,
                mult: 3,
                rate: 99,
            },
            ff4.stat_table[milon_z.physical_attack_index as usize]
        );
        assert_eq!(
            Stats {
                base: 1,
                mult: 1,
                rate: 60,
            },
            ff4.stat_table[milon_z.physical_defense_index as usize]
        );
        assert_eq!(
            Stats {
                base: 22,
                mult: 6,
                rate: 40,
            },
            ff4.stat_table[milon_z.magical_defense_index as usize]
        );
        assert_eq!(
            Speed { min: 9, max: 9 },
            ff4.speed_table[milon_z.speed_index as usize]
        );
    }
}
