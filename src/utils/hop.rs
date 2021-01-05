use std::fmt;
use std::net::IpAddr;
use std::time::{Duration, Instant};

/// Represents every single hop in a traceroute
pub struct Hop {
    ttl: u8,
    sent: IpAddr,
    sent_time: Instant,
    received: IpAddr,
    received_time: Instant,
    flowhash: u16,
    //name: String,
    //last_hop: bool,
}

impl Hop {
    pub fn new(ttl: u8, sent: IpAddr, sent_time: Instant, received: IpAddr, received_time: Instant, flowhash: u16) -> Hop {
        Hop {
            ttl,
            sent,
            sent_time,
            received,
            received_time,
            flowhash,
        }
    }

    /// Distance away
    pub fn ttl(&self) -> u8 {
        self.ttl
    }

    /// The address we sent the Probe from
    pub fn sent(&self) -> IpAddr {
        self.sent
    }

    /// The address which responded to our query
    pub fn received(&self) -> IpAddr {
        self.received
    }

    /// Round trip time to Hop and back
    pub fn elapsed(&self) -> Duration {
        self.received_time.duration_since(self.sent_time)
    }

    /// Hash of properties which usually follow the same path
    pub fn flowhash(&self) -> u16 {
        self.flowhash
    }

    //pub fn name(&self) -> String {
    //    self.name
    //}
}

impl fmt::Display for Hop {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:>2}  {:<15}  {:#?}", self.ttl, self.received, self.elapsed())
    }
}
