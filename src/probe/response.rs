use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::time::Instant;

/// Information received from the returned [`ProbeSent`](crate::probe::ProbeSent)
#[derive(Debug)]
pub enum ProbeResponse {
    /// IPv4 Response
    V4 {
        /// Ip of the server which responded
        source: Ipv4Addr,
        /// Id of the ip packet
        id: u16,
        /// Checksum of the embdedded udp packet
        checksum: u16,
        /// Time when the probe returned
        instant: Instant,
    },
    /// IPv6 Response
    V6 {
        /// Ip of the server which responded
        source: Ipv6Addr,
        /// Id of the ip packet
        id: u16,
        /// Checksum of the embdedded udp packet
        checksum: u16,
        /// Time when the probe returned
        instant: Instant,
    },
}

impl ProbeResponse {
    pub fn new(source: IpAddr, id: u16, checksum: u16, instant: Instant) -> Self {
        match source {
            IpAddr::V4(source) => Self::V4 {
                source,
                id,
                checksum,
                instant,
            },
            IpAddr::V6(source) => Self::V6 {
                source,
                id,
                checksum,
                instant,
            },
        }
    }

    pub fn get_source(&self) -> IpAddr {
        match self {
            Self::V4 { source, .. } => IpAddr::V4(*source),
            Self::V6 { source, .. } => IpAddr::V6(*source),
        }
    }

    pub fn get_id(&self) -> &u16 {
        match self {
            Self::V4 { id, .. } => id,
            Self::V6 { id, .. } => id,
        }
    }

    pub fn get_checksum(&self) -> &u16 {
        match self {
            Self::V4 { checksum, .. } => checksum,
            Self::V6 { checksum, .. } => checksum,
        }
    }

    pub fn get_instant(&self) -> &Instant {
        match self {
            Self::V4 { instant, .. } => instant,
            Self::V6 { instant, .. } => instant,
        }
    }
}

//impl fmt::Display for ProbeResponse {
//    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//        match self {
//            Self::V4 { id, source, .. } => write!(f, "{}, {}", id, source),
//            Self::V6 { id, source, .. } => write!(f, "{}, {}", id, source),
//        }
//    }
//}
