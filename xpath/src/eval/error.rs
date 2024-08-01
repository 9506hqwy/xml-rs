#[derive(Debug)]
pub enum Error {
    Dom(xml_dom::error::Error),
    InvalidType,
    InvalidArgumentCount(String),
    NotFoundFunction(String),
}

impl From<xml_dom::error::Error> for Error {
    fn from(value: xml_dom::error::Error) -> Self {
        Error::Dom(value)
    }
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
