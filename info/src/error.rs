use xml_parser::nom;

type ParseError<'a> = nom::Err<nom::error::Error<&'a str>>;

#[derive(Debug, PartialEq)]
pub enum Error {
    IsolatedNode,
    InvalidData(String),
    InvalidHierarchy,
    InvalidType,
    NotFoundDoumentElement,
    NotFoundReference(String),
    OufOfIndex(usize),
    Parse(String),
}

impl<'a> From<ParseError<'a>> for Error {
    fn from(value: ParseError<'a>) -> Self {
        Error::Parse(value.to_string())
    }
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
