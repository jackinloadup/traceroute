use std::cmp::Ordering;
use std::net::IpAddr;
use std::time::Instant;

#[derive(Debug)]
pub struct Probe {
    /// Transmit time
    pub instant: Instant,
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
            instant: Instant::now(),
            ttl,
            id,
            checksum,
            flowhash,
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

pub struct ProbeResponse {
    /// Ip of the server which responded
    pub source: IpAddr,
    /// Id of the ip packet
    pub id: u16,
    /// Checksum of the embdedded udp packet
    pub checksum: u16,
    /// Time when the probe returned
    pub instant: Instant,
}

impl ProbeResponse {
    pub fn new(source: IpAddr, id: u16, checksum: u16) -> Self {
        Self {
            source,
            id,
            checksum,
            instant: Instant::now(),
        }
    }
}
