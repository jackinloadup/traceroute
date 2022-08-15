use crate::protocol::Protocol;
use pnet::packet::icmp::IcmpType;
use std::error::Error;
use std::fmt;
use std::io::{self, ErrorKind};
use std::sync::mpsc;

/// Wrapper for all errors that can occur in this library
#[derive(Debug)]
pub enum TracerouteError {
    /// Recieved an [`io::Error`]
    Io(io::Error),
    UnmatchedPacket(&'static str),
    ICMPTypeUnexpected(IcmpType),
    /// A malformed packet was encountered
    MalformedPacket,
    /// This library doesn't currently support Ipv6
    NoIpv6,
    /// Attempted to use a protocol not yet supported
    UnimplimentedProtocol(Protocol),
    /// A channel was unexptectedly closed
    ChannelUnexpectedlyClosed,
    /// An invalid trace was created which used an Ipv4 and Ipv6 address for the source and
    /// destination
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
            Self::MalformedPacket => write!(f, "a malformed packet was encounted"),
            Self::NoIpv6 => write!(f, "ipv6 is not yet supported"),
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

impl<T> From<mpsc::SendError<T>> for TracerouteError {
    fn from(_err: mpsc::SendError<T>) -> Self {
        Self::ChannelUnexpectedlyClosed
    }
}
