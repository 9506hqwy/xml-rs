use crate::eval;

#[derive(Debug)]
pub enum Error<'a> {
    ExprRemain(&'a str),
    ExprSyntax(String),
    Eval(eval::error::Error),
}

impl<'a> From<eval::error::Error> for Error<'a> {
    fn from(value: eval::error::Error) -> Self {
        Error::Eval(value)
    }
}

impl<'a> std::error::Error for Error<'a> {}

impl<'a> std::fmt::Display for Error<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}

pub type Result<'a, T> = std::result::Result<T, Error<'a>>;
