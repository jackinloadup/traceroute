use crate::utils::Protocol;
use pnet::packet::icmp::IcmpType;
use std::error::Error;
use std::fmt;
use std::io::{self, ErrorKind};

/// Wrapper for all errors that can occur in this library
#[derive(Debug)]
pub enum TracerouteError {
    Io(io::Error),
    UnmatchedPacket(&'static str),
    ICMPTypeUnexpected(IcmpType),
    MalformedPacket,
    NoIpv6,
    UnimplimentedProtocol(Protocol),
    ChannelUnexpectedlyClosed,
    IpProtocolMismatch,
}

impl Error for TracerouteError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl fmt::Display for TracerouteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Io(ref err) => match err.kind() {
                ErrorKind::PermissionDenied => write!(
                    f,
                    "Couldn't open network: {:?}. Try again with sudo or boost this executable privilages with setcap cap_net_admin+ep",
                    err.kind()
                ),
                _ => write!(f, "failed with unhandled error: {}", err),
            },
            Self::UnmatchedPacket(ref err) => write!(f, "unmatched packet: {}", err),
            Self::MalformedPacket => write!(f, "malformed packet"),
            Self::NoIpv6 => write!(f, "ipv6 not yet supported"),
            Self::UnimplimentedProtocol(proto) => {
                write!(f, "{} probe is unimplimented", proto)
            },
            Self::ICMPTypeUnexpected(_icmp_type) => {
                write!(f, "ran into an unimplimented ICMP type")
            },
            Self::ChannelUnexpectedlyClosed => write!(f, "channel unexptectedly closed"),
            Self::IpProtocolMismatch => write!(f, "attempted to use ipv4 with ipv6"),
        }
    }
}

impl From<io::Error> for TracerouteError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}
