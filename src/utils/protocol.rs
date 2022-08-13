use std::fmt;

/// Protocol to be used for traceroute
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Protocol {
    /// UDP-based traceroute
    UDP,
    /// TCP-based traceroute
    TCP,
    /// ICMP-based traceroute
    ICMP,
    /// DCCP-based traceroute
    DCCP,
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} protocol", self)
    }
}
