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
    Impossible(&'static str),
    UnimplimentedProtocol(Protocol),
}

//impl TracerouteError {
//    fn as_str(&self) -> &str {
//        match error {
//            TracerouteError::Io(err) => match err.kind() {
//                ErrorKind::PermissionDenied => format!(
//                    "Couldn't open network: {:?}. Try again with sudo",
//                    err.kind()
//                ),
//                ErrorKind::Other => match err.into_inner() {
//                    Some(err) => format!("Failed: {}", err),
//                    None => format!("Failed with unhandled error"),
//                },
//                err_kind => format!("Failed with unhandled error: {:?}", err_kind),
//            },
//            TracerouteError::Impossible(err) | TracerouteError::UnmatchedPacket(err) => {
//                format!("Failed: {:?}", err)
//            }
//            TracerouteError::ICMPTypeUnexpected(_)
//            | TracerouteError::PacketDecode
//            | TracerouteError::MalformedPacket => format!("Failed decoding received packets"),
//            TracerouteError::NoIpv6 => format!("No support for ipv6 yet"),
//            TracerouteError::UnimplimentedProtocol(proto) => {
//                format!("No support for {:?} probes yet", proto)
//            }
//        }
//    }
//}

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
                _ => write!(f, "Failed with unhandled error: {}", err),
            },
            Self::Impossible(ref err) => write!(f, "impossible error: {}", err),
            Self::UnmatchedPacket(ref err) => write!(f, "unmatched packet: {}", err),
            Self::MalformedPacket => write!(f, "Malformed packet"),
            Self::NoIpv6 => write!(f, "Ipv6 not yet supported"),
            Self::UnimplimentedProtocol(proto) => {
                write!(f, "{} probe is unimplimented", proto)
            }
            Self::ICMPTypeUnexpected(_icmp_type) => {
                write!(f, "ran into an unimplimented ICMP type")
            }
        }
    }
}

impl From<io::Error> for TracerouteError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}
