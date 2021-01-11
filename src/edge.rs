use std::fmt;
use std::time::Duration;

/// Edge in the graph
#[derive(Clone)]
pub enum Edge {
    TTL(u8),
    RTT(Duration),
    Results(Duration, u8),
    Connected,
}

impl fmt::Display for Edge {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::TTL(ttl) => write!(f, "TTL {}", ttl),
            Self::RTT(duration) => write!(f, "Latency {:?}", duration),
            Self::Results(duration, ttl) => write!(f, "Latency {:?} @ {}", duration, ttl),
            Self::Connected => write!(f, ""),
        }
    }
}
