//!  Network path discovery library
//!
//!  Determine path used by packets sent from self and destination.
//!
//!  It's important to note that we are only able to measure the path in a outgoing direction. Some
//!  sections of the path may be the same in either direction but other sections may come back on a
//!  completely different path without visability. The return path for our probe may not be
//!  along the same path it came causing oddities in the probe latency. We don't yet detect this.
//!
//! # Example
//!
//! ```
//! use std::net::{IpAddr, Ipv4Addr};
//! use std::panic;
//! use traceroute::{Traceroute, TraceOptions, TraceActivity};
//! use async_std::stream::StreamExt;
//! use async_std::task::block_on;
//!
//! let mut traceroute = Traceroute::new()?;
//! let source = traceroute.addresses().first().unwrap().clone();
//! let destination = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
//! let options = TraceOptions::default();
//! let mut trace = traceroute.trace(source, destination, options)?;
//!
//! // First activity received
//! let activity = block_on(trace.next()).unwrap()?;
//!
//! // Close the network sockets
//! for thread_result in traceroute.close() {
//!     match thread_result {
//!        Ok(result) => result?,
//!        // We should never panic
//!        Err(e) => panic::resume_unwind(e),
//!     }
//! }
//! # Ok::<(), traceroute::TracerouteError>(())
//! ```
extern crate log;
extern crate petgraph;
extern crate pnet;

mod edge;
//mod node;
mod packet;
pub mod prelude;
mod probe;
mod protocol;
mod sockets;
mod trace;
mod traceroute;
mod utils;

pub use edge::Edge;
pub use prelude::*;
//pub use node::Node;
