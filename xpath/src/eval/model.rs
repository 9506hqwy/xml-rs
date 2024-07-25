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
pub enum Value {
    Boolean(bool),
    Node(Vec<XmlNode>),
    Number(f64),
    Text(String),
}

impl Default for Value {
    fn default() -> Self {
        Value::Node(vec![])
    }
}

/// function: string
impl TryFrom<&Value> for String {
    type Error = super::error::Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::Boolean(v) => {
                if *v {
                    Ok("true".to_string())
                } else {
                    Ok("false".to_string())
                }
            }
            Value::Node(v) => {
                if let Some(f) = v.first() {
                    Ok(f.as_string_value()?)
                } else {
                    Ok("".to_string())
                }
            }
            Value::Number(v) => match *v {
                f64::INFINITY => Ok("Infinity".to_string()),
                f64::NEG_INFINITY => Ok("-Infinity".to_string()),
                _ => Ok(v.to_string()),
            },
            Value::Text(v) => Ok(v.to_string()),
        }
    }
}

/// function: boolean
impl TryFrom<&Value> for bool {
    type Error = super::error::Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::Boolean(v) => Ok(*v),
            Value::Node(v) => Ok(!v.is_empty()),
            Value::Number(_) => {
                let n = f64::try_from(value)?;
                Ok(!(n == 0f64 || n.is_nan()))
            }
            Value::Text(v) => Ok(!v.is_empty()),
        }
    }
}

/// function: number
impl TryFrom<&Value> for f64 {
    type Error = super::error::Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::Boolean(v) => {
                if *v {
                    Ok(1f64)
                } else {
                    Ok(0f64)
                }
            }
            Value::Node(_) => {
                let s = String::try_from(value)?;
                Ok(f64::try_from(&Value::Text(s))?)
            }
            Value::Number(v) => Ok(*v),
            Value::Text(v) => Ok(v.parse::<f64>().unwrap_or(f64::NAN)),
        }
    }
}

impl cmp::PartialEq<bool> for Value {
    fn eq(&self, other: &bool) -> bool {
        bool::try_from(self)
            .map(|v| v == *other)
            .unwrap_or_default()
    }
}

impl cmp::PartialEq<f64> for Value {
    fn eq(&self, other: &f64) -> bool {
        f64::try_from(self).map(|v| v == *other).unwrap_or_default()
    }
}

impl cmp::PartialEq<String> for Value {
    fn eq(&self, other: &String) -> bool {
        String::try_from(self)
            .map(|v| &v == other)
            .unwrap_or_default()
    }
}

impl cmp::PartialOrd<bool> for Value {
    fn partial_cmp(&self, other: &bool) -> Option<cmp::Ordering> {
        self.partial_cmp(&f64::try_from(&Value::Boolean(*other)).ok()?)
    }
}

impl cmp::PartialOrd<f64> for Value {
    fn partial_cmp(&self, other: &f64) -> Option<cmp::Ordering> {
        f64::try_from(self).ok()?.partial_cmp(other)
    }
}

impl cmp::PartialOrd<String> for Value {
    fn partial_cmp(&self, other: &String) -> Option<cmp::Ordering> {
        self.partial_cmp(&f64::try_from(&Value::Text(other.clone())).ok()?)
    }
}

impl fmt::Display for Value {
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

impl ops::Add for Value {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let a = f64::try_from(&self).unwrap();
        let b = f64::try_from(&rhs).unwrap();
        Value::Number(a + b)
    }
}

impl ops::Sub for Value {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let a = f64::try_from(&self).unwrap();
        let b = f64::try_from(&rhs).unwrap();
        Value::Number(a - b)
    }
}

impl ops::Mul for Value {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let a = f64::try_from(&self).unwrap();
        let b = f64::try_from(&rhs).unwrap();
        Value::Number(a * b)
    }
}

impl ops::Div for Value {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        let a = f64::try_from(&self).unwrap();
        let b = f64::try_from(&rhs).unwrap();
        Value::Number(a / b)
    }
}

impl ops::Rem for Value {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        let a = f64::try_from(&self).unwrap();
        let b = f64::try_from(&rhs).unwrap();
        Value::Number(a % b)
    }
}

impl ops::Neg for Value {
    type Output = Self;

    fn neg(self) -> Self::Output {
        let a = f64::try_from(&self).unwrap();
        Value::Number(0f64 - a)
    }
}

impl Value {
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

pub trait AsValue {
    fn as_value(&self) -> Value;
}

impl AsValue for bool {
    fn as_value(&self) -> Value {
        Value::Boolean(*self)
    }
}

impl AsValue for XmlNode {
    fn as_value(&self) -> Value {
        Value::Node(vec![self.clone()])
    }
}

impl AsValue for Vec<XmlNode> {
    fn as_value(&self) -> Value {
        Value::Node(self.clone())
    }
}

impl AsValue for f64 {
    fn as_value(&self) -> Value {
        Value::Number(*self)
    }
}

impl AsValue for usize {
    fn as_value(&self) -> Value {
        Value::Number(*self as f64)
    }
}

impl AsValue for String {
    fn as_value(&self) -> Value {
        Value::Text(self.clone())
    }
}

impl AsValue for &str {
    fn as_value(&self) -> Value {
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
        assert_eq!("false", String::try_from(&v).unwrap());

        let v = Value::Boolean(true);
        assert_eq!("true", String::try_from(&v).unwrap());
    }

    #[test]
    fn test_value_to_string_node() {
        let v = Value::Node(vec![]);
        assert_eq!("", String::try_from(&v).unwrap());

        let (_, tree) = xml_parser::document("<root>a</root>").unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let v = Value::Node(vec![doc.as_node()]);
        assert_eq!("a", String::try_from(&v).unwrap());
    }

    #[test]
    fn test_value_to_string_number() {
        let v = Value::Number(0f64);
        assert_eq!("0", String::try_from(&v).unwrap());

        let v = Value::Number(-0f64);
        assert_eq!("-0", String::try_from(&v).unwrap());

        let v = Value::Number(f64::NAN);
        assert_eq!("NaN", String::try_from(&v).unwrap());

        let v = Value::Number(f64::INFINITY);
        assert_eq!("Infinity", String::try_from(&v).unwrap());

        let v = Value::Number(f64::NEG_INFINITY);
        assert_eq!("-Infinity", String::try_from(&v).unwrap());

        let v = Value::Number(1f64);
        assert_eq!("1", String::try_from(&v).unwrap());

        let v = Value::Number(1.1f64);
        assert_eq!("1.1", String::try_from(&v).unwrap());
    }

    #[test]
    fn test_value_to_string_text() {
        let v = Value::Text("a".to_string());
        assert_eq!("a", String::try_from(&v).unwrap());
    }

    #[test]
    fn test_value_to_bool_boolean() {
        let v = Value::Boolean(false);
        assert!(!bool::try_from(&v).unwrap());

        let v = Value::Boolean(true);
        assert!(bool::try_from(&v).unwrap());
    }

    #[test]
    fn test_value_to_bool_node() {
        let v = Value::Node(vec![]);
        assert!(!bool::try_from(&v).unwrap());

        let (_, tree) = xml_parser::document("<root>a</root>").unwrap();
        let doc = xml_dom::XmlDocument::from(xml_info::XmlDocument::new(&tree).unwrap());
        let v = Value::Node(vec![doc.as_node()]);
        assert!(bool::try_from(&v).unwrap());
    }

    #[test]
    fn test_value_to_bool_number() {
        let v = Value::Number(0f64);
        assert!(!bool::try_from(&v).unwrap());

        let v = Value::Number(-0f64);
        assert!(!bool::try_from(&v).unwrap());

        let v = Value::Number(f64::NAN);
        assert!(!bool::try_from(&v).unwrap());

        let v = Value::Number(f64::INFINITY);
        assert!(bool::try_from(&v).unwrap());

        let v = Value::Number(f64::NEG_INFINITY);
        assert!(bool::try_from(&v).unwrap());

        let v = Value::Number(1f64);
        assert!(bool::try_from(&v).unwrap());

        let v = Value::Number(1.1f64);
        assert!(bool::try_from(&v).unwrap());
    }

    #[test]
    fn test_value_to_bool_text() {
        let v = Value::Text("".to_string());
        assert!(!bool::try_from(&v).unwrap());

        let v = Value::Text("a".to_string());
        assert!(bool::try_from(&v).unwrap());
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
