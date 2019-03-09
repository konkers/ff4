#[macro_use]
extern crate simple_error;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate nom;

#[macro_use]
extern crate serde_derive;

use std::error::Error;

pub mod monster;
pub mod rom_map;
pub mod string;
pub mod test_utils;

pub struct Ff4 {
    pub monster_data: monster::MonsterData,
}

impl Ff4 {
    pub fn render_ai_condition_set(&self, set: &monster::ai::ConditionSet) -> String {
        let mut conditions = Vec::new();
        for &i in set.condition_indexes.iter() {
            let c = &self.monster_data.ai.conditions[i as usize];
            conditions.push(format!("op_{:02x}(0x{:02x}, 0x{:02x}, 0x{:02x}",
             c.op, c.args[0], c.args[1], c.args[2]));
        }

        conditions.join(" && ")
    }
    pub fn render_ai_entry(&self, entry: &monster::ai::GroupEntry) -> String {
        let condition_set = &self.monster_data.ai.condition_sets[entry.condition_set_index as usize];
        let conditions = self.render_ai_condition_set(condition_set);

        let mut s = format!("if({}) {{\n", conditions);

        s += "}}\n";
        s
    }

    pub fn render_ai(&self, group: &monster::ai::Group) -> String {
        let mut s = String::new();

        for entry in group.entries.iter() {
            s += &self.render_ai_entry(entry);
        }

        s
    }
}

pub fn parse_rom(data: &[u8]) -> Result<Ff4, Box<Error>> {
    let monster_data = monster::parse(data)?;

    Ok(Ff4 {
        monster_data: monster_data,
    })
}
