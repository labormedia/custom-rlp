use std::error;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    WrongType,
    InvalidCharLength,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::WrongType => write!(f, "Wrong type"),
            Self::InvalidCharLength => write!(f, "Invalid char length"),
        }
    }
}

impl error::Error for Error {}