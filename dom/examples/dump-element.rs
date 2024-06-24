use std::env;
use std::error::Error;
use std::fs;
use xml_dom::{Document, Node, NodeList};

fn main() -> Result<(), Box<dyn Error>> {
    let file_path = env::args().nth(1).ok_or("Missing file path")?;
    let contents = fs::read_to_string(file_path)?;
    let (_, tree) = xml_parser::document(&contents).unwrap();
    let dom = xml_dom::XmlDocument::from(&tree);
    let elements = dom.get_elements_by_tag_name("*");
    for i in 0..elements.length() {
        if let xml_dom::XmlNode::Element(element) = elements.item(i).unwrap() {
            dbg!(element.node_name());
        } else {
            unreachable!();
        }
    }
    Ok(())
}
