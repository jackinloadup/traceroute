use std::cmp::Ordering;
use std::net::IpAddr;
use std::time::Instant;

use crate::prelude::{Checksum, Flowhash, TTL, TcpId};
use crate::probe::ProbeSent;

/// Information to correlate a sent packet to it's response
#[derive(Debug)]
pub struct Probe {
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
}

impl Probe {
    pub fn new(source: IpAddr, ttl: TTL, id: TcpId, checksum: Checksum, flowhash: Flowhash) -> Self {
        Self {
            source,
            ttl,
            id,
            checksum,
            flowhash,
        }
    }

    /// Mark the moment the Probe is sent
    pub fn sent(self) -> ProbeSent {
        let Self {
            source,
            ttl,
            id,
            checksum,
            flowhash,
        } = self;

        ProbeSent {
            source,
            ttl,
            id,
            checksum,
            flowhash,
            instant: Instant::now(),
        }
    }
}

impl PartialEq for Probe {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for Probe {}

impl Ord for Probe {
    fn cmp(&self, other: &Self) -> Ordering {
        self.ttl.cmp(&other.ttl)
    }
}

impl PartialOrd for Probe {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
