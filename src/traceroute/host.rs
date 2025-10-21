use std::convert::TryInto;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::net::IpAddr;
use std::time::Duration;

// A long term view of a host with ping data
#[derive(Clone, Debug, Ord, PartialOrd)]
pub struct HostAgent {
    source: IpAddr,
    probes: Vec<Duration>,
}

impl HostAgent {
    pub fn new(source: IpAddr) -> Self {
        // maybe reserve space based on number of probes to send per host if known
        let probes = vec![];

        Self {
            source,
            probes,
        }
    }

    pub fn add_ping(&mut self, ping: Duration) {
        self.probes.push(ping);
    }

    pub fn average_ping(&self) -> Duration {
        let length = self.probes.len();
        if length == 1 {
            return self.probes[0];
        }
        let sum = self
            .probes
            .iter()
            .fold(Duration::new(0, 0), |acc, x| acc + *x);
        sum.checked_div(length.try_into().unwrap()).unwrap()
    }
}

impl PartialEq for HostAgent {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source
    }
}
impl Eq for HostAgent {}

impl Hash for HostAgent {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.source.hash(state);
    }
}

impl fmt::Display for HostAgent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:<15} in {:.3?}", self.source, self.average_ping())
    }
}
