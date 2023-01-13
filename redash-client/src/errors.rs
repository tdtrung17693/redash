use core::fmt;
use std::error::Error;

#[derive(Debug)]
pub struct OperationError(pub(crate) String);
impl fmt::Display for OperationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "There is an error: {}", self.0)
    }
}

impl Error for OperationError {}
