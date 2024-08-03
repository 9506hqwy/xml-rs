use super::error;
use super::model::{self, AsValue};
use std::ops::Range;
use xml_dom::{self as dom, AsExpandedName, Attr, Document, Element, Node};

pub type XPathFunc =
    dyn Fn(Vec<model::Value>, dom::XmlNode, &mut model::Context) -> error::Result<model::Value>;

pub fn table() -> Vec<Entry> {
    vec![
        Entry {
            local_part: "last".to_string(),
            namespace_uri: None,
            args: (0..0),
            call: Box::new(last),
        },
        Entry {
            local_part: "position".to_string(),
            namespace_uri: None,
            args: (0..0),
            call: Box::new(position),
        },
        Entry {
            local_part: "count".to_string(),
            namespace_uri: None,
            args: (1..1),
            call: Box::new(count),
        },
        Entry {
            local_part: "id".to_string(),
            namespace_uri: None,
            args: (1..1),
            call: Box::new(id),
        },
        Entry {
            local_part: "local-name".to_string(),
            namespace_uri: None,
            args: (0..1),
            call: Box::new(local_name),
        },
        Entry {
            local_part: "namespace-uri".to_string(),
            namespace_uri: None,
            args: (0..1),
            call: Box::new(namespace_uri),
        },
        Entry {
            local_part: "name".to_string(),
            namespace_uri: None,
            args: (0..1),
            call: Box::new(name),
        },
        Entry {
            local_part: "string".to_string(),
            namespace_uri: None,
            args: (0..1),
            call: Box::new(string),
        },
        Entry {
            local_part: "concat".to_string(),
            namespace_uri: None,
            args: (2..usize::MAX),
            call: Box::new(concat),
        },
        Entry {
            local_part: "starts-with".to_string(),
            namespace_uri: None,
            args: (2..2),
            call: Box::new(starts_with),
        },
        Entry {
            local_part: "contains".to_string(),
            namespace_uri: None,
            args: (2..2),
            call: Box::new(contains),
        },
        Entry {
            local_part: "substring-before".to_string(),
            namespace_uri: None,
            args: (2..2),
            call: Box::new(substring_before),
        },
        Entry {
            local_part: "substring-after".to_string(),
            namespace_uri: None,
            args: (2..2),
            call: Box::new(substring_after),
        },
        Entry {
            local_part: "substring".to_string(),
            namespace_uri: None,
            args: (2..3),
            call: Box::new(substring),
        },
        Entry {
            local_part: "string-length".to_string(),
            namespace_uri: None,
            args: (0..1),
            call: Box::new(string_length),
        },
        Entry {
            local_part: "normalize-space".to_string(),
            namespace_uri: None,
            args: (0..1),
            call: Box::new(normalize_space),
        },
        Entry {
            local_part: "translate".to_string(),
            namespace_uri: None,
            args: (3..3),
            call: Box::new(translate),
        },
        Entry {
            local_part: "boolean".to_string(),
            namespace_uri: None,
            args: (1..1),
            call: Box::new(boolean),
        },
        Entry {
            local_part: "not".to_string(),
            namespace_uri: None,
            args: (1..1),
            call: Box::new(not),
        },
        Entry {
            local_part: "true".to_string(),
            namespace_uri: None,
            args: (0..0),
            call: Box::new(ftrue),
        },
        Entry {
            local_part: "false".to_string(),
            namespace_uri: None,
            args: (0..0),
            call: Box::new(ffalse),
        },
        Entry {
            local_part: "lang".to_string(),
            namespace_uri: None,
            args: (1..1),
            call: Box::new(lang),
        },
        Entry {
            local_part: "number".to_string(),
            namespace_uri: None,
            args: (0..1),
            call: Box::new(number),
        },
        Entry {
            local_part: "sum".to_string(),
            namespace_uri: None,
            args: (1..1),
            call: Box::new(sum),
        },
        Entry {
            local_part: "floor".to_string(),
            namespace_uri: None,
            args: (1..1),
            call: Box::new(floor),
        },
        Entry {
            local_part: "ceiling".to_string(),
            namespace_uri: None,
            args: (1..1),
            call: Box::new(ceiling),
        },
        Entry {
            local_part: "round".to_string(),
            namespace_uri: None,
            args: (1..1),
            call: Box::new(round),
        },
    ]
}

// -----------------------------------------------------------------------------------------------

pub struct Entry {
    local_part: String,
    namespace_uri: Option<String>,
    args: Range<usize>,
    call: Box<XPathFunc>,
}

impl Entry {
    pub fn local_part(&self) -> &str {
        self.local_part.as_str()
    }

    pub fn namespace_uri(&self) -> Option<&str> {
        self.namespace_uri.as_deref()
    }

    pub fn min_args(&self) -> usize {
        self.args.start
    }

    pub fn max_args(&self) -> usize {
        self.args.end
    }

    pub fn exec(
        &self,
        args: Vec<model::Value>,
        node: dom::XmlNode,
        context: &mut model::Context,
    ) -> error::Result<model::Value> {
        (self.call)(args, node, context)
    }
}

// -----------------------------------------------------------------------------------------------

fn last(
    _: Vec<model::Value>,
    _: dom::XmlNode,
    context: &mut model::Context,
) -> error::Result<model::Value> {
    Ok(context.get_size().as_value())
}

fn position(
    _: Vec<model::Value>,
    _: dom::XmlNode,
    context: &mut model::Context,
) -> error::Result<model::Value> {
    Ok(context.get_position().as_value())
}

fn count(
    args: Vec<model::Value>,
    _: dom::XmlNode,
    _: &mut model::Context,
) -> error::Result<model::Value> {
    let arg = args.first().unwrap();
    if let model::Value::Node(n) = arg {
        Ok(n.len().as_value())
    } else {
        Err(error::Error::InvalidType)
    }
}

fn id(
    _: Vec<model::Value>,
    node: dom::XmlNode,
    _: &mut model::Context,
) -> error::Result<model::Value> {
    if node.owner_document().map(|v| v.doc_type()).is_some() {
        unimplemented!()
    } else {
        Ok(model::Value::Node(vec![]))
    }
}

fn local_name(
    args: Vec<model::Value>,
    node: dom::XmlNode,
    _: &mut model::Context,
) -> error::Result<model::Value> {
    let arg = if let Some(arg) = args.first() {
        if let model::Value::Node(n) = arg {
            n
        } else {
            return Err(error::Error::InvalidType);
        }
    } else {
        &vec![node]
    };

    if let Some(node) = arg.first() {
        if let Some((local_name, _, _)) = node.as_expanded_name()? {
            Ok(model::Value::Text(local_name))
        } else {
            Ok(model::Value::Text(String::new()))
        }
    } else {
        Ok(model::Value::Text(String::new()))
    }
}

fn namespace_uri(
    args: Vec<model::Value>,
    node: dom::XmlNode,
    _: &mut model::Context,
) -> error::Result<model::Value> {
    let arg = if let Some(arg) = args.first() {
        if let model::Value::Node(n) = arg {
            n
        } else {
            return Err(error::Error::InvalidType);
        }
    } else {
        &vec![node]
    };

    if let Some(node) = arg.first() {
        if let Some((_, _, uri)) = node.as_expanded_name()? {
            Ok(model::Value::Text(uri.unwrap_or_default()))
        } else {
            Ok(model::Value::Text(String::new()))
        }
    } else {
        Ok(model::Value::Text(String::new()))
    }
}

fn name(
    args: Vec<model::Value>,
    node: dom::XmlNode,
    _: &mut model::Context,
) -> error::Result<model::Value> {
    let arg = if let Some(arg) = args.first() {
        if let model::Value::Node(n) = arg {
            n
        } else {
            return Err(error::Error::InvalidType);
        }
    } else {
        &vec![node]
    };

    if let Some(node) = arg.first() {
        match node.as_expanded_name()? {
            Some((local_name, Some(prefix), _)) => {
                if prefix == "xmlns" {
                    Ok(model::Value::Text(local_name))
                } else {
                    Ok(model::Value::Text(format!("{}:{}", prefix, local_name)))
                }
            }
            Some((local_name, _, _)) => Ok(model::Value::Text(local_name)),
            _ => Ok(model::Value::Text(String::new())),
        }
    } else {
        Ok(model::Value::Text(String::new()))
    }
}

// -----------------------------------------------------------------------------------------------

fn string(
    args: Vec<model::Value>,
    node: dom::XmlNode,
    _: &mut model::Context,
) -> error::Result<model::Value> {
    let arg = if let Some(arg) = args.first() {
        arg
    } else {
        &model::Value::Node(vec![node])
    };
    Ok(model::Value::Text(String::try_from(arg)?))
}

fn concat(
    args: Vec<model::Value>,
    _: dom::XmlNode,
    _: &mut model::Context,
) -> error::Result<model::Value> {
    let mut s = String::new();
    for arg in args {
        s.push_str(&String::try_from(&arg)?);
    }
    Ok(model::Value::Text(s))
}

fn starts_with(
    args: Vec<model::Value>,
    _: dom::XmlNode,
    _: &mut model::Context,
) -> error::Result<model::Value> {
    let mut args = args.iter();
    let s1 = String::try_from(args.next().unwrap())?;
    let s2 = String::try_from(args.next().unwrap())?;
    Ok(model::Value::Boolean(s1.starts_with(&s2)))
}

fn contains(
    args: Vec<model::Value>,
    _: dom::XmlNode,
    _: &mut model::Context,
) -> error::Result<model::Value> {
    let mut args = args.iter();
    let s1 = String::try_from(args.next().unwrap())?;
    let s2 = String::try_from(args.next().unwrap())?;
    Ok(model::Value::Boolean(s1.contains(&s2)))
}

fn substring_before(
    args: Vec<model::Value>,
    _: dom::XmlNode,
    _: &mut model::Context,
) -> error::Result<model::Value> {
    let mut args = args.iter();
    let s1 = String::try_from(args.next().unwrap())?;
    let s2 = String::try_from(args.next().unwrap())?;
    let r = s1.split_once(&s2).map(|v| v.0).unwrap_or_default();
    Ok(model::Value::Text(r.to_string()))
}

fn substring_after(
    args: Vec<model::Value>,
    _: dom::XmlNode,
    _: &mut model::Context,
) -> error::Result<model::Value> {
    let mut args = args.iter();
    let s1 = String::try_from(args.next().unwrap())?;
    let s2 = String::try_from(args.next().unwrap())?;
    let r = s1.split_once(&s2).map(|v| v.1).unwrap_or_default();
    Ok(model::Value::Text(r.to_string()))
}

fn substring(
    args: Vec<model::Value>,
    _: dom::XmlNode,
    _: &mut model::Context,
) -> error::Result<model::Value> {
    let mut args = args.iter();
    let v = String::try_from(args.next().unwrap())?;
    let s = f64::try_from(args.next().unwrap())?.round() as usize - 1;
    let c = if let Some(v) = args.next() {
        Some(f64::try_from(v)?.round() as usize)
    } else {
        None
    };
    let (_, mut r) = v.split_at(s);
    if let Some(c) = c {
        (r, _) = r.split_at(c);
    }
    Ok(model::Value::Text(r.to_string()))
}

fn string_length(
    args: Vec<model::Value>,
    node: dom::XmlNode,
    _: &mut model::Context,
) -> error::Result<model::Value> {
    let arg = if let Some(arg) = args.first() {
        arg
    } else {
        &model::Value::Node(vec![node])
    };
    Ok(model::Value::Number(String::try_from(arg)?.len() as f64))
}

fn normalize_space(
    args: Vec<model::Value>,
    node: dom::XmlNode,
    _: &mut model::Context,
) -> error::Result<model::Value> {
    let arg = if let Some(arg) = args.first() {
        arg
    } else {
        &model::Value::Node(vec![node])
    };
    let r = String::try_from(arg)?;
    let w = r.split_whitespace().collect::<Vec<&str>>();
    Ok(model::Value::Text(w.join(" ")))
}

fn translate(
    args: Vec<model::Value>,
    _: dom::XmlNode,
    _: &mut model::Context,
) -> error::Result<model::Value> {
    let mut args = args.iter();
    let s1 = String::try_from(args.next().unwrap())?;
    let s2 = String::try_from(args.next().unwrap())?;
    let s3 = String::try_from(args.next().unwrap())?;
    let mut r = String::new();
    for ch in s1.chars() {
        if let Some(index) = s2.chars().position(|v| v == ch) {
            if let Some(ch) = s3.chars().nth(index) {
                r.push(ch);
            }
        } else {
            r.push(ch)
        }
    }
    Ok(model::Value::Text(r))
}

// -----------------------------------------------------------------------------------------------

fn boolean(
    args: Vec<model::Value>,
    _: dom::XmlNode,
    _: &mut model::Context,
) -> error::Result<model::Value> {
    let arg = args.first().unwrap();
    Ok(model::Value::Boolean(bool::try_from(arg)?))
}

fn not(
    args: Vec<model::Value>,
    _: dom::XmlNode,
    _: &mut model::Context,
) -> error::Result<model::Value> {
    let arg = args.first().unwrap();
    Ok(model::Value::Boolean(!bool::try_from(arg)?))
}

fn ftrue(
    _: Vec<model::Value>,
    _: dom::XmlNode,
    _: &mut model::Context,
) -> error::Result<model::Value> {
    Ok(model::Value::Boolean(true))
}

fn ffalse(
    _: Vec<model::Value>,
    _: dom::XmlNode,
    _: &mut model::Context,
) -> error::Result<model::Value> {
    Ok(model::Value::Boolean(false))
}

fn lang(
    args: Vec<model::Value>,
    node: dom::XmlNode,
    _: &mut model::Context,
) -> error::Result<model::Value> {
    let name = String::try_from(args.first().unwrap())?;

    let mut n = Some(node);
    while let Some(dom::XmlNode::Element(element)) = n {
        // FIXME: namespace
        if let Some(attr) = element.get_attribute_node("lang") {
            if attr.value()? == name {
                return Ok(model::Value::Boolean(true));
            }
        }

        n = element.parent_node();
    }

    Ok(model::Value::Boolean(false))
}

// -----------------------------------------------------------------------------------------------

fn number(
    args: Vec<model::Value>,
    node: dom::XmlNode,
    _: &mut model::Context,
) -> error::Result<model::Value> {
    let arg = if let Some(arg) = args.first() {
        arg
    } else {
        &model::Value::Node(vec![node])
    };
    Ok(model::Value::Number(f64::try_from(arg)?))
}

fn sum(
    args: Vec<model::Value>,
    _: dom::XmlNode,
    _: &mut model::Context,
) -> error::Result<model::Value> {
    let arg = args.first().unwrap();
    if let model::Value::Node(nodes) = arg {
        let mut s = 0f64;
        for node in nodes {
            s += f64::try_from(&model::Value::Node(vec![node.clone()]))?
        }
        Ok(model::Value::Number(s))
    } else {
        Err(error::Error::InvalidType)
    }
}

fn floor(
    args: Vec<model::Value>,
    _: dom::XmlNode,
    _: &mut model::Context,
) -> error::Result<model::Value> {
    let arg = f64::try_from(args.first().unwrap())?;
    Ok(model::Value::Number(arg.floor()))
}

fn ceiling(
    args: Vec<model::Value>,
    _: dom::XmlNode,
    _: &mut model::Context,
) -> error::Result<model::Value> {
    let arg = f64::try_from(args.first().unwrap())?;
    Ok(model::Value::Number(arg.ceil()))
}

fn round(
    args: Vec<model::Value>,
    _: dom::XmlNode,
    _: &mut model::Context,
) -> error::Result<model::Value> {
    let arg = f64::try_from(args.first().unwrap())?;
    Ok(model::Value::Number(arg.round()))
}
