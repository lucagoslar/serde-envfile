use std;
use std::fmt::{self, Display};

use serde::{de, ser};

pub type Result<T> = std::result::Result<T, Error>;

/// Possible errors.
#[derive(Debug)]
pub enum Error {
    Message(String),

    Eof,
    Syntax,
    ExpectedBoolean,
    ExpectedInteger,
    UnsupportedTupleStruct,
    UnsupportedStructureInSeq,
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl Error {
    pub fn new<E>(e: E) -> Self
    where
        E: std::error::Error,
    {
        Self::Message(e.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Message(msg) => formatter.write_str(msg),
            Error::Eof => formatter.write_str("unexpected end of input"),
            Error::Syntax => formatter.write_str("synatx error"),
            Error::ExpectedBoolean => formatter.write_str("expected boolean"),
            Error::ExpectedInteger => formatter.write_str("expected integer"),
            Error::UnsupportedTupleStruct => formatter.write_str("tuple structs are not supported"),
            Error::UnsupportedStructureInSeq => {
                formatter.write_str("unsupported structure in sequence")
            }
        }
    }
}

impl std::error::Error for Error {}
