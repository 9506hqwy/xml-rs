// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PrefixedName<'a> {
    pub prefix: &'a str,
    pub local_part: &'a str,
}

impl<'a> From<(&'a str, &'a str)> for PrefixedName<'a> {
    fn from(value: (&'a str, &'a str)) -> Self {
        let (prefix, local_part) = value;
        PrefixedName { prefix, local_part }
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum QName<'a> {
    Prefixed(PrefixedName<'a>),
    Unprefixed(&'a str),
}

impl<'a> Default for QName<'a> {
    fn default() -> Self {
        QName::Unprefixed("")
    }
}

impl<'a> From<PrefixedName<'a>> for QName<'a> {
    fn from(value: PrefixedName<'a>) -> Self {
        QName::Prefixed(value)
    }
}

impl<'a> From<&'a str> for QName<'a> {
    fn from(value: &'a str) -> Self {
        QName::Unprefixed(value)
    }
}

// -----------------------------------------------------------------------------------------------
