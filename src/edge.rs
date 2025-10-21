use std::cmp::Ordering;
use std::fmt;
use std::time::Duration;

/// Edge in the graph
#[derive(Clone, Debug)]
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

impl PartialEq for Edge {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::TTL(ttl) => match other {
                Self::TTL(ttl2) => ttl == ttl2,
                Self::RTT(_) | Self::Connected => false,
            },
            Self::RTT(duration) => match other {
                Self::RTT(duration2) => duration == duration2,
                Self::TTL(_) | Self::Connected => false,
            },
            Self::Connected => match other {
                Self::Connected => true, // Might be false
                Self::TTL(_) | Self::RTT(_)  => false,
            },
        }
    }
}
impl Eq for Edge {}

impl Ord for Edge {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            Self::TTL(ttl) => match other {
                Self::TTL(ttl2) => ttl.cmp(ttl2),
                Self::RTT(_) | Self::Connected => Ordering::Less,
            },
            Self::RTT(duration) => match other {
                Self::TTL(_) => Ordering::Greater,
                Self::RTT(duration2) => duration.cmp(duration2),
                Self::Connected  => Ordering::Less,
            },
            Self::Connected => match other {
                Self::TTL(_) | Self::RTT(_) => Ordering::Greater,
                Self::Connected => Ordering::Equal
            },
        }
    }
}

impl PartialOrd for Edge {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
