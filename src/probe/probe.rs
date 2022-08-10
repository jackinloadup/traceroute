use crate::probe::ProbeSent;
use std::cmp::Ordering;
use std::time::Instant;

/// Information to correlate a sent packet to it's response
#[derive(Debug)]
pub struct Probe {
    /// TCP ttl value which will control how many hops until the packet is returned to sender
    pub ttl: u8,
    /// TCP identification
    pub id: u16,
    /// Checksum of inner UDP probe
    pub checksum: u16,
    /// Flowhash
    pub flowhash: u16,
}

impl Probe {
    pub fn new(ttl: u8, id: u16, checksum: u16, flowhash: u16) -> Self {
        Self {
            ttl,
            id,
            checksum,
            flowhash,
        }
    }

    /// Mark the moment the Probe is sent
    pub fn sent(self) -> ProbeSent {
        let Self {
            ttl,
            id,
            checksum,
            flowhash,
        } = self;

        ProbeSent {
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
