use std::{fmt, str::FromStr};

use pnet::packet::ip::{IpNextHeaderProtocol, IpNextHeaderProtocols};

use super::{error::ParseProtocolErr, udp::UdpParams};

/// Protocol to be used for traceroute
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Protocol {
    /// Datagram Congestion Control Protocol
    DCCP,
    /// Internet Control Message Protocol
    ICMP,
    /// Stream Control Transmission Protocol
    SCTP,
    /// Transmission Control Protocol
    TCP,
    /// User Datagram Protocol
    UDP(UdpParams),
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Default for Protocol {
    fn default() -> Self {
        Self::UDP(UdpParams::default())
    }
}

impl FromStr for Protocol {
    type Err = ParseProtocolErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let protocol = match s {
            "dccp" | "DCCP" => Self::DCCP,
            "icmp" | "ICMP" => Self::ICMP,
            "sctp" | "SCTP" => Self::SCTP,
            "tcp" | "TCP" => Self::TCP,
            "udp" | "UDP" => Self::UDP(UdpParams::default()),
            _ => Err(ParseProtocolErr::UnknownProtocol)?,
        };

        Ok(protocol)
    }
}

impl Into<IpNextHeaderProtocol> for Protocol {
    fn into(self) -> IpNextHeaderProtocol {
        match self {
            Self::DCCP => IpNextHeaderProtocols::Dccp,
            Self::ICMP => IpNextHeaderProtocols::Icmp,
            Self::SCTP => IpNextHeaderProtocols::Sctp,
            Self::TCP => IpNextHeaderProtocols::Tcp,
            Self::UDP(_) => IpNextHeaderProtocols::Udp,
        }
    }
}
