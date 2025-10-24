//!  Network path discovery library
//!
//!  Determine path used by packets sent from self and destination.
//!
//!  <div class="warning">
//!
//!  It's important to note that the path we are measuring is only in the outgoing direction. The
//!  packets from target back to the source may take a different path we can't measure from our
//!  viewpoint. This can cause oddities in the probe latency.
//!
//!  </div>
//!
//! # Example
//!
//! ```
//! use std::net::{IpAddr, Ipv4Addr};
//! use std::panic;
//! use traceroute::{Traceroute, TraceOptions};
//! use async_std::stream::StreamExt;
//! use async_std::task::block_on;
//!
//! let mut traceroute = Traceroute::new(5)?;
//! let source = traceroute.addresses().first().unwrap().clone();
//! let destination = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
//! let options = TraceOptions::default();
//! let mut trace = traceroute.trace(source, destination, options)?;
//!
//! // First activity received
//! let activity = block_on(StreamExt::next(&mut trace)).unwrap()?;
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
mod node;
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
pub use node::Node;
