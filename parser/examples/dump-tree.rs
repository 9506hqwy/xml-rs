use std::env;
use std::error::Error;
use std::fs;

fn main() -> Result<(), Box<dyn Error>> {
    let file_path = env::args().nth(1).ok_or("Missing file path")?;
    let contents = fs::read_to_string(file_path)?;
    match xml_parser::document(&contents) {
        Ok((_, tokens)) => {
            dbg!(tokens);
        }
        Err(e) => {
            dbg!(e);
        }
    }
    Ok(())
}
