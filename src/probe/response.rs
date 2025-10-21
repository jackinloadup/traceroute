use std::net::IpAddr;
use std::time::{Duration, Instant};

use super::ProbeSent;

use crate::prelude::TTL;

/// Information received from the returned [`ProbeSent`](crate::probe::ProbeSent)
///
/// The response doesn't currently take into account the checksum of the sent probe vs the
/// response. Also there is no attempt to inspect the payload of the probe sent. There is no
/// current payload being sent but it may be useful to take advantage of the payload to investigate
/// when routers are spliting packets or other funky stuff.
///
/// Upon creation the following values are being discarded for reference
/// TcpId
/// Checksum (sent and received)
#[derive(Clone,Debug)]
pub struct ProbeResponse {
    /// How many hops away the `destination` is
    pub ttl: TTL,
    /// IP of the machine which responded
    pub destination: IpAddr,
    /// Time when the probe returned
    pub ping: Duration,
    /// Probe that was sent
    pub sent: ProbeSent,
}

impl ProbeResponse {
    pub fn new(sent: ProbeSent, destination: IpAddr, moment_received: Instant) -> Self {
        let ping = moment_received.duration_since(sent.instant);

        Self {
            ttl: sent.ttl,
            destination,
            ping,
            sent,
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
