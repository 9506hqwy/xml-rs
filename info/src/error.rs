#[derive(Debug)]
pub enum Error {
    IsolatedNode,
    NotFoundDoumentElement,
    NotFoundReference(String),
}

pub type Result<T> = std::result::Result<T, Error>;
