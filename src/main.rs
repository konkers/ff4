extern crate ff4;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::error::Error;
use std::fs::{create_dir_all, write};

use ff4::monster::{DropTable, Monster, Speed, Stats};
use ff4::test_utils;

// Aggregate all information about a given monster for easier printing.
#[derive(Debug, Default, PartialEq, Serialize)]
pub struct CombinedMonster {
    pub monster: Monster,
    pub xp: u16,
    pub gp: u16,
    pub physical_attack: Stats,
    pub physical_defense: Stats,
    pub magical_defense: Stats,
    pub speed: Speed,
    pub drop_table: DropTable,
}

fn dump_monsters(ff4: &ff4::monster::Ff4) {
    let dir = format!("out/monster");
    create_dir_all(&dir).unwrap();
    for i in 0..ff4.monsters.len() {
        let name = ff4.name_table[i].trim();

        let m = &ff4.monsters[i];
        let monster = CombinedMonster {
            monster: ff4.monsters[i].clone(),
            xp: ff4.xp_table[i],
            gp: ff4.gp_table[i],
            physical_attack: ff4.stat_table[m.physical_attack_index as usize].clone(),
            physical_defense: ff4.stat_table[m.physical_defense_index as usize].clone(),
            magical_defense: ff4.stat_table[m.magical_defense_index as usize].clone(),
            speed: ff4.speed_table[m.speed_index as usize].clone(),
            drop_table: ff4.drop_tables[m.drop_table_index as usize].clone(),
        };
        let j = serde_json::to_string_pretty(&monster).unwrap();
        write(format!("{}/{}.json", &dir, name), &j).unwrap();
    }
}

fn main() -> Result<(), Box<Error>> {
    let rom_data = test_utils::load_rom()?;
    let ff4 = ff4::monster::parse_rom(&rom_data)?;

    dump_monsters(&ff4);

    Ok(())
}
