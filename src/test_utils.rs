use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

pub fn load_rom() -> Result<Vec<u8>, Box<Error>> {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("ff2us.smc");

    let mut f = File::open(p)?;
    let mut data = Vec::new();
    f.read_to_end(&mut data)?;

    Ok(data)
}
