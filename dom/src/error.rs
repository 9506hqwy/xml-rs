use xml_parser::nom;

type ParseError<'a> = nom::Err<nom::error::Error<&'a str>>;

#[derive(Debug, PartialEq)]
pub enum Error {
    Dom(DomException),
    Info(xml_info::error::Error),
    Parse(String),
}

#[derive(Debug, PartialEq)]
pub enum DomException {
    IndexSizeErr,
    DomStringSizeErr,
    HierarchyRequestErr,
    WrongDocumentErr,
    InvalidCharacterErr,
    NoDataAllowedErr,
    NoModificationAllowedErr,
    NotFoundErr,
    NotSupportErr,
    InuseAttributeErr,
}

impl From<DomException> for Error {
    fn from(value: DomException) -> Self {
        Error::Dom(value)
    }
}

impl<'a> From<ParseError<'a>> for Error {
    fn from(value: ParseError<'a>) -> Self {
        Error::Parse(value.to_string())
    }
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
