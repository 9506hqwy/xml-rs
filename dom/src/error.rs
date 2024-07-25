#[derive(Debug)]
pub enum Error {
    Info(xml_info::error::Error),
}

impl From<xml_info::error::Error> for Error {
    fn from(value: xml_info::error::Error) -> Self {
        Error::Info(value)
    }
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
