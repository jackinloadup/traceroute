use std::fmt;

/// Protocol to be used for traceroute
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Protocol {
    /// User Datagram Protocol
    UDP,
    /// Transmission Control Protocol
    TCP,
    /// Internet Control Message Protocol
    ICMP,
    /// Datagram Congestion Control Protocol
    DCCP,
    /// Stream Control Transmission Protocol
    SCTP,
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} protocol", self)
    }
}
