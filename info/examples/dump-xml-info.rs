use std::env;
use std::error::Error;
use std::fs;

fn main() -> Result<(), Box<dyn Error>> {
    let file_path = env::args().nth(1).ok_or("Missing file path")?;
    let contents = fs::read_to_string(file_path)?;
    let (rest, tree) = xml_parser::document(&contents).map_err(|e| e.to_string())?;
    if !rest.is_empty() {
        Err("Exist tailing string.".into())
    } else {
        let doc = xml_info::XmlDocument::new(&tree).unwrap();
        dbg!(doc);
        Ok(())
    }
}
