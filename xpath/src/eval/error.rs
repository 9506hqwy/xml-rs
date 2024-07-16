#[derive(Debug)]
pub enum Error {
    InvalidType,
}

pub type Result<T> = std::result::Result<T, Error>;
