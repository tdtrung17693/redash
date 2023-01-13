use core::fmt;
use std::{error::Error, io};

#[derive(Debug)]
pub enum RedashError {
    DataError(String),
    IOError(io::Error),
    UnknownError(Box<dyn Error>),
    ServerError(String, u8),
    OperationError(String),
}

impl fmt::Display for RedashError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            RedashError::DataError(err) => write!(f, "{err}"),
            RedashError::IOError(err) => write!(f, "{err}"),
            RedashError::UnknownError(err) => write!(f, "{err}"),
            RedashError::ServerError(err, u) => {
                write!(f, "{err} - Server response: {}", *u as char)
            }
            RedashError::OperationError(err) => write!(f, "{err}"),
        }
    }
}
impl Error for RedashError {}
