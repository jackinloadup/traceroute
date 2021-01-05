use std::net::IpAddr;
use std::fmt;
use std::cmp::Ordering;

#[derive(Copy, Clone, Hash)]
pub enum Node {
  Hop(IpAddr),
  Hidden(u8, u16),
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Hop(ip) => write!(f, "{}", ip),
            Self::Hidden(ttl, flowhash) => write!(f, "#{:x?} Hidden @ {}", flowhash, ttl),
        }
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Hop(ip) => match other {
                Self::Hop(ip2) => ip == ip2,
                Self::Hidden(_, _) => false,
            }
            Self::Hidden(ttl, _) => match other {
                Self::Hop(_) => false,
                Self::Hidden(ttl2, _) => ttl == ttl2,
            }
        }
    }
}
impl Eq for Node {}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            Self::Hop(ip) => match other {
                Self::Hop(ip2) => ip.cmp(ip2),
                Self::Hidden(_,_) => Ordering::Less,
            }
            Self::Hidden(ttl,_) => match other {
                Self::Hop(_) => Ordering::Greater,
                Self::Hidden(ttl2,_) => ttl.cmp(ttl2),
            }
        }
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
