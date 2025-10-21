use std::fmt;
use std::net::IpAddr;
use std::time::Duration;

/// Represents a response from a node at a certain distance/ttl in a Trace
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Hop {
    /// The hop responded
    Received(IpAddr, Duration),
    /// The sent probe was ignored or didn't make it back
    TimedOut,
    /// Masked
    Masked,
}

impl fmt::Display for Hop {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Received(ip, duration) => write!(f, "{:?}  {:#?}", ip, duration),
            Self::TimedOut => write!(f, "Timed out"),
            Self::Masked => write!(f, "Masked"),
        }
    }
}
