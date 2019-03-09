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

pub fn parse_rom(data: &[u8]) -> Result<Ff4, Box<Error>> {
    let monster_data = monster::parse(data)?;

    Ok(Ff4 {
        monster_data: monster_data,
    })
}
