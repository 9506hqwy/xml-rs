use std::cmp;
use std::fmt;
use std::ops;
use xml_dom::AsStringValue;
use xml_dom::XmlNode;

// -----------------------------------------------------------------------------------------------

#[derive(Default)]
pub struct Context {
    size: Vec<usize>,
    position: Vec<usize>,
}

impl Context {
    pub fn get_position(&self) -> usize {
        *self.position.last().unwrap_or(&0)
    }

    pub fn pop_position(&mut self) -> usize {
        self.position.pop().unwrap_or_default()
    }

    pub fn push_position(&mut self, position: usize) {
        self.position.push(position);
    }

    pub fn get_size(&self) -> usize {
        *self.size.last().unwrap_or(&0)
    }

    pub fn pop_size(&mut self) -> usize {
        self.size.pop().unwrap_or_default()
    }

    pub fn push_size(&mut self, size: usize) {
        self.size.push(size);
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Debug)]
pub enum Value<'a> {
    Boolean(bool),
    Node(Vec<XmlNode<'a>>),
    Number(f64),
    Text(String),
}

impl<'a> Default for Value<'a> {
    fn default() -> Self {
        Value::Node(vec![])
    }
}

/// function: string
impl<'a> From<&Value<'a>> for String {
    fn from(value: &Value<'a>) -> Self {
        match value {
            Value::Boolean(v) => {
                if *v {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            }
            Value::Node(v) => {
                if let Some(f) = v.first() {
                    f.as_string_value()
                } else {
                    "".to_string()
                }
            }
            Value::Number(v) => match *v {
                f64::INFINITY => "Infinity".to_string(),
                f64::NEG_INFINITY => "-Infinity".to_string(),
                _ => v.to_string(),
            },
            Value::Text(v) => v.to_string(),
        }
    }
}

/// function: boolean
impl<'a> From<&Value<'a>> for bool {
    fn from(value: &Value<'a>) -> Self {
        match value {
            Value::Boolean(v) => *v,
            Value::Node(v) => !v.is_empty(),
            Value::Number(_) => {
                let n = f64::from(value);
                !(n == 0f64 || n.is_nan())
            }
            Value::Text(v) => !v.is_empty(),
        }
    }
}

/// function: number
impl<'a> From<&Value<'a>> for f64 {
    fn from(value: &Value<'a>) -> Self {
        match value {
            Value::Boolean(v) => {
                if *v {
                    1f64
                } else {
                    0f64
                }
            }
            Value::Node(_) => {
                let s = String::from(value);
                f64::from(&Value::Text(s))
            }
            Value::Number(v) => *v,
            Value::Text(v) => v.parse::<f64>().unwrap_or(f64::NAN),
        }
    }
}

impl<'a> cmp::PartialEq<bool> for Value<'a> {
    fn eq(&self, other: &bool) -> bool {
        bool::from(self) == *other
    }
}

impl<'a> cmp::PartialEq<f64> for Value<'a> {
    fn eq(&self, other: &f64) -> bool {
        f64::from(self) == *other
    }
}

impl<'a> cmp::PartialEq<String> for Value<'a> {
    fn eq(&self, other: &String) -> bool {
        &String::from(self) == other
    }
}

impl<'a> cmp::PartialOrd<bool> for Value<'a> {
    fn partial_cmp(&self, other: &bool) -> Option<cmp::Ordering> {
        self.partial_cmp(&f64::from(&Value::Boolean(*other)))
    }
}

impl<'a> cmp::PartialOrd<f64> for Value<'a> {
    fn partial_cmp(&self, other: &f64) -> Option<cmp::Ordering> {
        f64::from(self).partial_cmp(other)
    }
}

impl<'a> cmp::PartialOrd<String> for Value<'a> {
    fn partial_cmp(&self, other: &String) -> Option<cmp::Ordering> {
        self.partial_cmp(&f64::from(&Value::Text(other.clone())))
    }
}

impl<'a> fmt::Display for Value<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Value::Boolean(v) => v.fmt(f),
            Value::Node(v) => {
                for n in v {
                    n.fmt(f)?;
                }
                Ok(())
            }
            Value::Number(v) => v.fmt(f),
            Value::Text(v) => v.fmt(f),
        }
    }
}

impl<'a> ops::Add for Value<'a> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let a = f64::from(&self);
        let b = f64::from(&rhs);
        Value::Number(a + b)
    }
}

impl<'a> ops::Sub for Value<'a> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let a = f64::from(&self);
        let b = f64::from(&rhs);
        Value::Number(a - b)
    }
}

impl<'a> ops::Mul for Value<'a> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let a = f64::from(&self);
        let b = f64::from(&rhs);
        Value::Number(a * b)
    }
}

impl<'a> ops::Div for Value<'a> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        let a = f64::from(&self);
        let b = f64::from(&rhs);
        Value::Number(a / b)
    }
}

impl<'a> ops::Rem for Value<'a> {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        let a = f64::from(&self);
        let b = f64::from(&rhs);
        Value::Number(a % b)
    }
}

impl<'a> ops::Neg for Value<'a> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        let a = f64::from(&self);
        Value::Number(0f64 - a)
    }
}

impl<'a> Value<'a> {
    pub fn is_bool(&self) -> bool {
        matches!(self, Value::Boolean(_))
    }

    pub fn is_node(&self) -> bool {
        matches!(self, Value::Node(_))
    }

    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }

    pub fn is_text(&self) -> bool {
        matches!(self, Value::Text(_))
    }
}

// -----------------------------------------------------------------------------------------------

pub trait AsValue<'a> {
    fn as_value(&self) -> Value<'a>;
}

impl<'a> AsValue<'a> for bool {
    fn as_value(&self) -> Value<'a> {
        Value::Boolean(*self)
    }
}

impl<'a> AsValue<'a> for XmlNode<'a> {
    fn as_value(&self) -> Value<'a> {
        Value::Node(vec![self.clone()])
    }
}

impl<'a> AsValue<'a> for Vec<XmlNode<'a>> {
    fn as_value(&self) -> Value<'a> {
        Value::Node(self.clone())
    }
}

impl<'a> AsValue<'a> for f64 {
    fn as_value(&self) -> Value<'a> {
        Value::Number(*self)
    }
}

impl<'a> AsValue<'a> for usize {
    fn as_value(&self) -> Value<'a> {
        Value::Number(*self as f64)
    }
}

impl<'a> AsValue<'a> for String {
    fn as_value(&self) -> Value<'a> {
        Value::Text(self.clone())
    }
}

impl<'a> AsValue<'a> for &str {
    fn as_value(&self) -> Value<'a> {
        Value::Text(self.to_string())
    }
}

// -----------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use cmp::Ordering;
    use xml_dom::AsNode;

    #[test]
    fn test_context_position() {
        let mut ctx = Context::default();
        assert_eq!(0, ctx.get_position());

        ctx.push_position(1);
        assert_eq!(1, ctx.get_position());

        ctx.pop_position();
        assert_eq!(0, ctx.get_position());

        ctx.pop_position();
    }

    #[test]
    fn test_context_size() {
        let mut ctx = Context::default();
        assert_eq!(0, ctx.get_size());

        ctx.push_size(1);
        assert_eq!(1, ctx.get_size());

        ctx.pop_size();
        assert_eq!(0, ctx.get_size());

        ctx.pop_size();
    }

    #[test]
    fn test_value_to_string_boolean() {
        let v = Value::Boolean(false);
        assert_eq!("false", String::from(&v));

        let v = Value::Boolean(true);
        assert_eq!("true", String::from(&v));
    }

    #[test]
    fn test_value_to_string_node() {
        let v = Value::Node(vec![]);
        assert_eq!("", String::from(&v));

        let (_, tree) = xml_parser::document("<root>a</root>").unwrap();
        let doc = xml_dom::XmlDocument::from(&tree);
        let v = Value::Node(vec![doc.as_node()]);
        assert_eq!("a", String::from(&v));
    }

    #[test]
    fn test_value_to_string_number() {
        let v = Value::Number(0f64);
        assert_eq!("0", String::from(&v));

        let v = Value::Number(-0f64);
        assert_eq!("-0", String::from(&v));

        let v = Value::Number(f64::NAN);
        assert_eq!("NaN", String::from(&v));

        let v = Value::Number(f64::INFINITY);
        assert_eq!("Infinity", String::from(&v));

        let v = Value::Number(f64::NEG_INFINITY);
        assert_eq!("-Infinity", String::from(&v));

        let v = Value::Number(1f64);
        assert_eq!("1", String::from(&v));

        let v = Value::Number(1.1f64);
        assert_eq!("1.1", String::from(&v));
    }

    #[test]
    fn test_value_to_string_text() {
        let v = Value::Text("a".to_string());
        assert_eq!("a", String::from(&v));
    }

    #[test]
    fn test_value_to_bool_boolean() {
        let v = Value::Boolean(false);
        assert!(!bool::from(&v));

        let v = Value::Boolean(true);
        assert!(bool::from(&v));
    }

    #[test]
    fn test_value_to_bool_node() {
        let v = Value::Node(vec![]);
        assert!(!bool::from(&v));

        let (_, tree) = xml_parser::document("<root>a</root>").unwrap();
        let doc = xml_dom::XmlDocument::from(&tree);
        let v = Value::Node(vec![doc.as_node()]);
        assert!(bool::from(&v));
    }

    #[test]
    fn test_value_to_bool_number() {
        let v = Value::Number(0f64);
        assert!(!bool::from(&v));

        let v = Value::Number(-0f64);
        assert!(!bool::from(&v));

        let v = Value::Number(f64::NAN);
        assert!(!bool::from(&v));

        let v = Value::Number(f64::INFINITY);
        assert!(bool::from(&v));

        let v = Value::Number(f64::NEG_INFINITY);
        assert!(bool::from(&v));

        let v = Value::Number(1f64);
        assert!(bool::from(&v));

        let v = Value::Number(1.1f64);
        assert!(bool::from(&v));
    }

    #[test]
    fn test_value_to_bool_text() {
        let v = Value::Text("".to_string());
        assert!(!bool::from(&v));

        let v = Value::Text("a".to_string());
        assert!(bool::from(&v));
    }

    #[test]
    fn test_value_eq_boolean() {
        let v = Value::Boolean(true);
        assert!(!v.eq(&false));

        let v = Value::Boolean(true);
        assert!(v.eq(&true));
    }

    #[test]
    fn test_value_eq_number() {
        let v = Value::Number(0f64);
        assert!(!v.eq(&1f64));

        let v = Value::Number(0f64);
        assert!(v.eq(&0f64));
    }

    #[test]
    fn test_value_eq_text() {
        let v = Value::Text("a".to_string());
        assert!(!v.eq(&"b".to_string()));

        let v = Value::Text("a".to_string());
        assert!(v.eq(&"a".to_string()));
    }

    #[test]
    fn test_value_ord_boolean() {
        let v = Value::Boolean(false);
        assert_eq!(Some(Ordering::Less), v.partial_cmp(&true));

        let v = Value::Boolean(true);
        assert_eq!(Some(Ordering::Equal), v.partial_cmp(&true));

        let v = Value::Boolean(true);
        assert_eq!(Some(Ordering::Greater), v.partial_cmp(&false));
    }

    #[test]
    fn test_value_ord_number() {
        let v = Value::Number(0f64);
        assert_eq!(Some(Ordering::Less), v.partial_cmp(&1f64));

        let v = Value::Number(0f64);
        assert_eq!(Some(Ordering::Equal), v.partial_cmp(&0f64));

        let v = Value::Number(0f64);
        assert_eq!(Some(Ordering::Greater), v.partial_cmp(&-1f64));
    }

    #[test]
    fn test_value_ord_text() {
        let v = Value::Text("0".to_string());
        assert_eq!(Some(Ordering::Less), v.partial_cmp(&"1".to_string()));

        let v = Value::Text("0".to_string());
        assert_eq!(Some(Ordering::Equal), v.partial_cmp(&"0".to_string()));

        let v = Value::Text("0".to_string());
        assert_eq!(Some(Ordering::Greater), v.partial_cmp(&"-1".to_string()));
    }

    #[test]
    fn test_value_add() {
        let a = Value::Number(2f64);
        let b = Value::Number(1f64);
        let c = a + b;
        assert!(c.eq(&3f64));
    }

    #[test]
    fn test_value_sub() {
        let a = Value::Number(2f64);
        let b = Value::Number(1f64);
        let c = a - b;
        assert!(c.eq(&1f64));
    }

    #[test]
    fn test_value_mul() {
        let a = Value::Number(2f64);
        let b = Value::Number(1f64);
        let c = a * b;
        assert!(c.eq(&2f64));
    }

    #[test]
    fn test_value_div() {
        let a = Value::Number(2f64);
        let b = Value::Number(1f64);
        let c = a / b;
        assert!(c.eq(&2f64));
    }

    #[test]
    fn test_value_mod() {
        let a = Value::Number(2f64);
        let b = Value::Number(1f64);
        let c = a % b;
        assert!(c.eq(&0f64));
    }

    #[test]
    fn test_value_neg() {
        let a = Value::Number(2f64);
        let b = -a;
        assert!(b.eq(&-2f64));
    }
}
