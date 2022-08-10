use std::cmp::Ordering;
use std::time::Instant;

/// Created by [`Probe`](crate::probe::Probe) when a packet is passed to the network to mark the [`Instant`] it was
/// sent
#[derive(Debug, Clone, Copy)]
pub struct ProbeSent {
    /// TCP ttl value which will control how many hops until the packet is returned to sender
    pub ttl: u8,
    /// TCP identification
    pub id: u16,
    /// Checksum of inner UDP probe
    pub checksum: u16,
    /// Flowhash
    pub flowhash: u16,
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
