use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::net::IpAddr;
use std::time::Instant;

use crate::prelude::{Checksum, Flowhash, TTL, TcpId};

/// Created by [`Probe`](crate::probe::Probe) when a packet is passed to the network to mark the [`Instant`] it was
/// sent
#[derive(Clone,Debug)]
pub struct ProbeSent {
    /// Source IP Address
    pub source: IpAddr,
    /// TCP ttl value which will control how many hops until the packet is returned to sender
    pub ttl: TTL,
    /// TCP identification
    pub id: TcpId,
    /// Checksum of inner UDP probe
    pub checksum: Checksum,
    /// Flowhash
    pub flowhash: Flowhash,
    /// The instant the probe was sent
    pub instant: Instant,
}

impl PartialEq for ProbeSent {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for ProbeSent {}

impl Ord for ProbeSent {
    fn cmp(&self, other: &Self) -> Ordering {
        self.ttl.cmp(&other.ttl)
    }
}

impl PartialOrd for ProbeSent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for ProbeSent {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
