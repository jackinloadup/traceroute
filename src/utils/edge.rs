use std::time::Duration;
use std::fmt;

#[derive(Clone)]
pub enum Edge {
    TTL(u8),
    RTT(Duration),
    Connected,
}

impl fmt::Display for Edge {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::TTL(ttl) => write!(f, "TTL {}", ttl),
            Self::RTT(duration) => write!(f, "Latency {:?}", duration),
            Self::Connected => write!(f, ""),
        }
    }
}
