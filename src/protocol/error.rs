use std::{error::Error, fmt};

/// Represents an error which occurred whilst parsing a Protocol.
#[derive(Copy, Debug, PartialEq, Eq, Clone)]
pub enum ParseProtocolErr {
    UnknownProtocol,
}

impl Error for ParseProtocolErr {}

impl fmt::Display for ParseProtocolErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseProtocolErr::UnknownProtocol => write!(f, "Unknown Protocol"),
        }
    }
}
