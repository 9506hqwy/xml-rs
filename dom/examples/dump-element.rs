use std::env;
use std::error::Error;
use std::fs;
use xml_dom::{Document, Node};

fn main() -> Result<(), Box<dyn Error>> {
    let file_path = env::args().nth(1).ok_or("Missing file path")?;
    let contents = fs::read_to_string(file_path)?;

    let (_, dom) = xml_dom::XmlDocument::from_raw(&contents)?;
    let elements = dom.get_elements_by_tag_name("*");

    for element in elements.iter() {
        if let xml_dom::XmlNode::Element(element) = element {
            dbg!(element.node_name());
        } else {
            unreachable!();
        }
    }
    Ok(())
}
