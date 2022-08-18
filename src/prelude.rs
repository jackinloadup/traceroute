//! The traceroute prelude.
//!
//! The prelude re-exports most commonly used traits and macros from this crate.
//!
//! # Examples
//!
//! Import the prelude with:
//!
//! ```
//! # #[allow(unused_imports)]
//! use traceroute::prelude::*;
//! ```
//!
pub type TTL = u8;
pub type TcpId = u16;
pub type Checksum = u16;
pub type Flowhash = u16;

pub use crate::protocol::Protocol;
pub use crate::sockets::SocketJoinResult;
pub use crate::trace::{Trace, TraceOptions};
pub use crate::traceroute::{Traceroute, TracerouteError};
pub use crate::utils::Hop;
