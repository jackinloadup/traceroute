use std::fmt;
use std::net::IpAddr;
use std::time::Duration;

/// Represents every single hop in a traceroute
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Hop {
    /// The hop responded
    Received(IpAddr, Duration),
    /// The sent probe was ignored or didn't make it back
    TimedOut,
    /// Masked
    Masked,
}

impl fmt::Display for Hop {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Received(ip, duration) => write!(f, "{:?}  {:#?}", ip, duration),
            Self::TimedOut => write!(f, "Timed out"),
            Self::Masked => write!(f, "Masked"),
        }
    }
}

//impl Hop {
//    pub fn new(
//        ttl: u8,
//        sent: IpAddr,
//        sent_time: Instant,
//        received: IpAddr,
//        received_time: Instant,
//        flowhash: u16,
//    ) -> Hop {
//        Hop {
//            ttl,
//            sent,
//            sent_time,
//            received,
//            received_time,
//            flowhash,
//        }
//    }
//
//    /// Distance away
//    pub fn ttl(&self) -> u8 {
//        self.ttl
//    }
//
//    /// The address we sent the Probe from
//    pub fn sent(&self) -> IpAddr {
//        self.sent
//    }
//
//    /// The address which responded to our query
//    pub fn received(&self) -> IpAddr {
//        self.received
//    }
//
//    /// Round trip time to Hop and back
//    pub fn elapsed(&self) -> Duration {
//        self.received_time.duration_since(self.sent_time)
//    }
//
//    /// Hash of properties which usually follow the same path
//    pub fn flowhash(&self) -> u16 {
//        self.flowhash
//    }
//
//    //pub fn name(&self) -> String {
//    //    self.name
//    //}
//}
