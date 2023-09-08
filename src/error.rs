use roxmltree::Error as XMLError;
use std::error::Error;
use std::fmt;
use std::io::Error as IoError;
use std::num::ParseFloatError;

#[derive(Debug)]
/// The error type of the library, which gets
/// returned on parsing issues.
pub struct Metadata {
    details: String,
}

impl Metadata {
    pub fn new(msg: &str) -> Metadata {
        Metadata {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for Metadata {
    fn description(&self) -> &str {
        &self.details
    }
}

impl From<ParseFloatError> for Metadata {
    fn from(_: ParseFloatError) -> Metadata {
        Metadata::new("Cannot convert string to float")
    }
}

impl From<IoError> for Metadata {
    fn from(e: IoError) -> Metadata {
        Metadata::new(&e.to_string())
    }
}

impl From<XMLError> for Metadata {
    fn from(e: XMLError) -> Metadata {
        Metadata::new(&e.to_string())
    }
}
