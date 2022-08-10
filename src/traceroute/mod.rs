use std::sync::Arc;

mod error;
mod traceroute;

// Contemplating if this is needed. I'm thinking the application should choose if this is needed
pub type TraceAgent = Arc<Traceroute>;

pub use self::traceroute::Traceroute;
pub use error::TracerouteError;
