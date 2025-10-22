use crate::sockets::{SocketJoinResult, Sockets};
use crate::trace::{Trace, TraceOptions};
use crate::traceroute::TracerouteError;
use log::*;
use std::net::IpAddr;
use std::time::Duration;

/// Interface to init the network and create [`Trace`s](Trace)
///
/// # Panics
/// The [`close()`](Traceroute::close) fn must be run before drop to capture any panics that came
/// from the socket threads. This shouldn't happen but in the case it does there is a way to handle
/// it from within the application logic and not here.

// Provides management interface for traceroute
// TODO bind/unbind interface?
pub struct Traceroute {
    sockets: Sockets,
    //probes: HashMap<u16, Probe>,
    //streams: HashMap<u16, TraceActivity>,
}

impl Traceroute {
    /// Create a new traceroute engine
    pub fn new(packet_delay: u16) -> Result<Self, TracerouteError> {
        let packet_delay = Duration::from_millis(packet_delay as u64);
        let sockets = Sockets::new(packet_delay)?;

        Ok(Self { sockets })
    }

    /// Get available system addresses which can be used as the source for a trace
    pub fn addresses(&self) -> &Vec<IpAddr> {
        self.sockets.addresses()
    }

    /// Close network connections
    ///
    /// This must be run before drop to capture any panics that came from the socket threads.
    /// This shouldn't happen but in the case it does there is a way to handle it from within
    /// the application logic and not here.
    pub fn close(&mut self) -> SocketJoinResult {
        self.sockets.close()
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
