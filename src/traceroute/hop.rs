use crate::traceroute::HostAgent;
use std::collections::HashMap;
use std::fmt;
use std::net::IpAddr;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Hop {
    distance: u8,
    // Map keyed by Destination IP
    hosts: HashMap<IpAddr, HostAgent>,
}

impl Hop {
    pub fn new(distance: u8) -> Self {
        let hosts = HashMap::new();
        Self { distance, hosts }
    }

    pub fn get_host_mut(&mut self, source: IpAddr) -> &mut HostAgent {
        if !self.hosts.contains_key(&source) {
            let host = HostAgent::new(source);
            self.hosts.insert(source, host);
        }

        // unwrap okay because we just created the object above us. We know it exists.
        self.hosts.get_mut(&source).unwrap()
    }

    //pub fn tick(&self) {
    //    self.hosts.iter().for_each(|(_, host)| host.tick());
    //}

    pub fn is_empty(&self) -> bool {
        self.hosts.len() == 0
    }

    pub fn distance(&self) -> u8 {
        self.distance
    }
}

impl fmt::Display for Hop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.hosts
            .values()
            .try_fold((), |_, host| write!(f, "{:>2} {}", self.distance, host))
    }
}
