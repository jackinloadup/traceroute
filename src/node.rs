use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::net::IpAddr;

use crate::prelude::{Checksum, Flowhash, TTL, TcpId};

/// Node in the graph
#[derive(Copy, Clone, Debug)]
pub enum Node {
    Flow(Flowhash),
    Hop(IpAddr),
    // A node that didn't respond, aka timed out
    Hidden(TTL),
    Masked(TTL),
}

impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Flow(flowhash) => flowhash.hash(state),
            Self::Hop(ip) => ip.hash(state),
            Self::Hidden(ttl) => ttl.hash(state),
            Self::Masked(ttl) => ttl.hash(state),
        }
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Flow(flowhash) => write!(f, "Flow #{:x?}", flowhash),
            Self::Hop(ip) => write!(f, "{}", ip),
            Self::Hidden(_ttl) => write!(f, "Hidden"),
            Self::Masked(_ttl) => write!(f, "Masked"),
        }
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Flow(flow) => match other {
                Self::Flow(flow2) => flow == flow2,
                Self::Hop(_) | Self::Hidden(_) | Self::Masked(_) => false,
            },
            Self::Hop(ip) => match other {
                Self::Hop(ip2) => ip == ip2,
                Self::Flow(_) | Self::Hidden(_) | Self::Masked(_) => false,
            },
            Self::Hidden(ttl) => match other {
                Self::Hidden(ttl2) => ttl == ttl2,
                Self::Flow(_) | Self::Hop(_) | Self::Masked(_) => false,
            },
            Self::Masked(ttl) => match other {
                Self::Masked(ttl2) => ttl == ttl2,
                Self::Flow(_) | Self::Hidden(_) | Self::Hop(_) => false,
            },
        }
    }
}
impl Eq for Node {}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            Self::Flow(flow) => match other {
                Self::Flow(flow2) => flow.cmp(flow2),
                Self::Hop(_) | Self::Hidden(_) | Self::Masked(_) => Ordering::Less,
            },
            Self::Hop(ip) => match other {
                Self::Flow(_) => Ordering::Greater,
                Self::Hop(ip2) => ip.cmp(ip2),
                Self::Hidden(_) | Self::Masked(_) => Ordering::Less,
            },
            Self::Hidden(ttl) => match other {
                Self::Flow(_) | Self::Hop(_) => Ordering::Greater,
                Self::Hidden(ttl2) => ttl.cmp(ttl2),
                Self::Masked(_) => Ordering::Less,
            },
            Self::Masked(ttl) => match other {
                Self::Flow(_) | Self::Hop(_) | Self::Hidden(_) => Ordering::Greater,
                Self::Masked(ttl2) => ttl.cmp(ttl2),
            },
        }
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
