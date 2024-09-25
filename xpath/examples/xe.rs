use std::borrow::{Borrow, BorrowMut};
use std::error::Error;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::{env, fs, io};
use xml_dom::{
    AsNode, Attr, AttrMut, CharacterData, Document, DocumentMut, Element, NamedNodeMapMut, Node,
    PrettyPrint,
};

struct Argument {
    file: Option<PathBuf>,
    expr: String,
    value: String,
    ns: Vec<(Option<String>, String)>,
    no_indent: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let arg = args()?;

    let mut context = xml_xpath::eval::model::Context::default();
    for n in arg.ns.as_slice() {
        context.add_ns(n.0.as_deref(), n.1.as_str());
    }

    let dom = document(arg.file.as_deref())?;

    let value = xml_xpath::query(dom.clone(), arg.expr.as_str(), &mut context)
        .map_err(|v| v.to_string())?;

    match value {
        xml_xpath::eval::model::Value::Boolean(_) => {
            return Err("Specify XML element not value using XPATH.".into());
        }
        xml_xpath::eval::model::Value::Node(nodes) => {
            for node in nodes {
                replace(node, arg.value.as_str())?;
            }
        }
        xml_xpath::eval::model::Value::Number(_) => {
            return Err("Specify XML element not value using XPATH.".into());
        }
        xml_xpath::eval::model::Value::Text(_) => {
            return Err("Specify XML element not value using XPATH.".into());
        }
    }

    if arg.no_indent {
        println!("{}", dom);
    } else {
        dom.pretty_print()?;
    }
    Ok(())
}

fn args() -> Result<Argument, Box<dyn Error>> {
    let mut file = None;
    let mut expr = None;
    let mut value = None;
    let mut ns = vec![];
    let mut no_indent = false;

    let mut args = env::args();
    args.next(); // skip exe.
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--xpath" => {
                if expr.is_some() {
                    return Err("Specify `--xpath` only once.".into());
                }

                expr = Some(args.next().ok_or("Specify value of `--xpath`.")?);
            }
            "--value" => {
                if value.is_some() {
                    return Err("Specify `--value` only once.".into());
                }

                let node = args.next().ok_or("Specify value of `--value`.")?;
                parse_node(node.as_str())?;
                value = Some(node);
            }
            "--setns" => {
                let n = args.next().ok_or("Specify value of `--setns`.")?;

                let (prefix, uri) = n
                    .split_once('=')
                    .ok_or("Specify `xmlns:prefix=uri` format of `--setns`.")?;
                let (prefix, local_part) = prefix.split_once(':').unwrap_or(("", prefix));

                if prefix.is_empty() && local_part != "xmlns" {
                    return Err("Specify `xmlns` of `--setns` for default namespace.".into());
                } else if !prefix.is_empty() && prefix != "xmlns" {
                    return Err("Specify value of `--setns` with `xmlns` prefix.".into());
                }

                let name = if local_part == "xmlns" {
                    None
                } else {
                    Some(local_part.to_string())
                };

                ns.push((name, uri.to_string()));
            }
            "--no-indent" => {
                no_indent = true;
            }
            _ => {
                if file.is_some() {
                    return Err("Specify `file path` only once.".into());
                }

                file = Some(PathBuf::from(arg));
            }
        }
    }

    if expr.is_none() {
        return Err("Specify `--xpath`".into());
    }

    if value.is_none() {
        return Err("Specify `--value`".into());
    }

    Ok(Argument {
        file,
        expr: expr.unwrap(),
        value: value.unwrap(),
        ns,
        no_indent,
    })
}

fn document(path: Option<&Path>) -> Result<xml_dom::XmlDocument, Box<dyn Error>> {
    let contents = match path {
        Some(path) => fs::read_to_string(path)?,
        _ => {
            let mut contents = vec![];
            io::stdin().read_to_end(&mut contents)?;
            String::from_utf8(contents)?
        }
    };

    let context = xml_dom::Context::from_text_expanded(true);
    let (rest, dom) = xml_dom::XmlDocument::from_raw_with_context(contents.as_str(), context)?;
    if !rest.is_empty() {
        return Err("invalid format XML".into());
    }

    Ok(dom)
}

fn parse_node(node: &str) -> Result<xml_dom::XmlElement, Box<dyn Error>> {
    let doc = format!("<e>{}</e>", node);
    let (rest, dom) = xml_dom::XmlDocument::from_raw(doc.as_str())?;
    if !rest.is_empty() {
        return Err("invalid format XML".into());
    }

    Ok(dom.borrow().document_element()?)
}

fn replace(node: xml_dom::XmlNode, value: &str) -> Result<(), Box<dyn Error>> {
    match node {
        xml_dom::XmlNode::Document(v) => {
            clear_child(v.clone())?;
            append_child(v, value)?;
        }
        xml_dom::XmlNode::Attribute(v) => {
            clear_child(v.clone())?;
            append_child(v, value)?;
        }
        xml_dom::XmlNode::Element(v) => {
            clear_child(v.clone())?;
            append_child(v, value)?;
        }
        _ => {
            return Err("Specify XML element not value using XPATH.".into());
        }
    }

    Ok(())
}

fn clear_child<T>(node: T) -> Result<(), Box<dyn Error>>
where
    T: xml_dom::Node + xml_dom::NodeMut,
{
    for child in node.child_nodes().iter() {
        node.remove_child(&child)?;
    }

    Ok(())
}

fn append_child<T>(node: T, value: &str) -> Result<(), Box<dyn Error>>
where
    T: Clone + xml_dom::Node + xml_dom::NodeMut,
{
    let new_value = parse_node(value)?;

    for child in new_value.child_nodes().iter() {
        append_child_to_tree(node.clone(), child)?;
    }

    Ok(())
}

fn append_child_to_tree<T>(node: T, child: xml_dom::XmlNode) -> Result<(), Box<dyn Error>>
where
    T: Clone + xml_dom::Node + xml_dom::NodeMut,
{
    match child {
        xml_dom::XmlNode::Attribute(v) => {
            let mut n = node
                .owner_document()
                .unwrap()
                .create_attribute(v.name().as_str())?;
            n.borrow_mut().set_value(v.value()?.as_str())?;

            if let Some(mut attr) = node.attributes() {
                attr.borrow_mut().set_named_item(n)?;
            } else {
                return Err("Not supported XML node type.".into());
            }
        }
        xml_dom::XmlNode::CData(v) => {
            let n = node
                .owner_document()
                .unwrap()
                .create_cdata_section(v.data()?.as_str());
            node.append_child(n.as_node())?;
        }
        xml_dom::XmlNode::Comment(v) => {
            let n = node
                .owner_document()
                .unwrap()
                .create_comment(v.data()?.as_str());
            node.append_child(n.as_node())?;
        }
        xml_dom::XmlNode::Element(v) => {
            let n = node
                .owner_document()
                .unwrap()
                .create_element(v.tag_name().as_str())?;
            node.append_child(n.as_node())?;

            if let Some(attributes) = v.attributes() {
                for descendant in attributes.iter() {
                    append_child_to_tree(n.clone(), descendant.as_node())?;
                }
            }

            for descendant in v.child_nodes().iter() {
                append_child_to_tree(n.clone(), descendant)?;
            }
        }
        xml_dom::XmlNode::EntityReference(v) => {
            let n = node
                .owner_document()
                .unwrap()
                .create_entity_reference(v.node_name().as_str())?;
            node.append_child(n.as_node())?;
        }
        xml_dom::XmlNode::Text(v) => {
            let n = node
                .owner_document()
                .unwrap()
                .create_text_node(v.data()?.as_str());
            node.append_child(n.as_node())?;
        }
        _ => {
            return Err("Not supported XML node type.".into());
        }
    }

    Ok(())
}
