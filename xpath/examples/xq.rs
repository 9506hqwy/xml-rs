use std::error::Error;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::{env, fs, io};
use xml_dom::PrettyPrint;

struct Argument {
    file: Option<PathBuf>,
    expr: String,
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

    let value =
        xml_xpath::query(dom, arg.expr.as_str(), &mut context).map_err(|v| v.to_string())?;

    match value {
        xml_xpath::eval::model::Value::Boolean(v) => {
            println!("{}", v);
        }
        xml_xpath::eval::model::Value::Node(nodes) => {
            for node in nodes {
                if arg.no_indent {
                    println!("{}", node);
                } else {
                    node.pretty_print()?;
                    println!();
                }
            }
        }
        xml_xpath::eval::model::Value::Number(v) => {
            println!("{}", v);
        }
        xml_xpath::eval::model::Value::Text(v) => {
            println!("{}", v);
        }
    }

    Ok(())
}

fn args() -> Result<Argument, Box<dyn Error>> {
    let mut file = None;
    let mut expr = None;
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

    Ok(Argument {
        file,
        expr: expr.unwrap(),
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
