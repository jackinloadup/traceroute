use crate::sockets::Sockets;
use crate::trace::{Trace, TraceOptions};
use crate::traceroute::TracerouteError;
use log::*;
use std::net::IpAddr;

/// Interface to init the network and create [`Trace`s](Trace)
///
/// Drop to close network sockets

// Provides management interface for traceroute
// TODO bind/unbind interface?
pub struct Traceroute {
    sockets: Sockets,
    //queue: HashMap<u16, Probe>,
    //probes: HashMap<u16, Probe>,
    //streams: HashMap<u16, TraceActivity>,
}

impl Traceroute {
    /// Create a new traceroute engine
    pub fn new() -> Result<Self, TracerouteError> {
        let sockets = Sockets::new()?;

        Ok(Self { sockets })
    }

    /// Get available system addresses which can be used as the source for a trace
    pub fn addresses(&self) -> &Vec<IpAddr> {
        self.sockets.addresses()
    }

    /// Run a trace against a single target.
    pub fn trace(
        &self,
        source: IpAddr,
        destination: IpAddr,
        options: TraceOptions,
    ) -> Result<Trace, TracerouteError> {
        let packet_sender = self.sockets.packet_sender();
        info!("Start trace for {}", destination);

        Trace::new(options, source, destination, packet_sender)
    }
}
