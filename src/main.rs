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
    pub ai: String,
}

fn dump_monsters(ff4: &ff4::Ff4) {
    let dir = format!("out/monster");
    create_dir_all(&dir).unwrap();
    let monster_data = &ff4.monster_data;
    for i in 0..monster_data.monsters.len() {
        let name = monster_data.name_table[i].trim();
        println!("{}", name);

        let m = &monster_data.monsters[i];
        let stat_table = &monster_data.stat_table;

        let seq = m.attack_seq_group as usize;
        let ai = if monster_data.ai.groups.len() > seq {
           ff4.render_ai(&monster_data.ai.groups[seq])
        } else {
            "unknown".to_string()
        };

        let monster = CombinedMonster {
            monster: m.clone(),
            xp: monster_data.xp_table[i],
            gp: monster_data.gp_table[i],
            physical_attack: stat_table[m.physical_attack_index as usize].clone(),
            physical_defense: stat_table[m.physical_defense_index as usize].clone(),
            magical_defense: stat_table[m.magical_defense_index as usize].clone(),
            speed: monster_data.speed_table[m.speed_index as usize].clone(),
            drop_table: monster_data.drop_tables[m.drop_table_index as usize].clone(),
            ai: ai,
        };
        let j = serde_json::to_string_pretty(&monster).unwrap();
        write(format!("{}/{:02x}-{}.json", &dir, i, name), &j).unwrap();
    }
}

fn main() -> Result<(), Box<Error>> {
    let rom_data = test_utils::load_rom()?;
    let ff4 = ff4::parse_rom(&rom_data)?;

    dump_monsters(&ff4);

    Ok(())
}
