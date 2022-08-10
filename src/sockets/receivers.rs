use crate::utils::handle_ipv4_packet;
use crate::{ProbeResponse, TraceActivity, TraceResponse, TraceResult, TracerouteError};
use core::sync::atomic::{AtomicBool, Ordering};
use log::*;
use pnet::transport::{ipv4_packet_iter, TransportReceiver};
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

pub struct SocketReceiver {
    rx: TransportReceiver,
}
impl SocketReceiver {
    pub fn new(rx: TransportReceiver) -> Self {
        Self { rx }
    }
}

pub enum SocketReceivers {
    V4(SocketReceiver),
    V6(SocketReceiver),
    Both {
        v4: SocketReceiver,
        v6: SocketReceiver,
    },
}

impl SocketReceivers {
    pub fn receive(
        &mut self,
        probe_receiver: Receiver<(u16, Sender<TraceResult>)>,
        runnable: Arc<AtomicBool>,
    ) -> Result<(), TracerouteError> {
        // Wait for new packets from the outside world
        let timeout = Duration::from_millis(100);

        let mut probes: HashMap<u16, Sender<TraceResult>> = HashMap::new();
        while runnable.load(Ordering::SeqCst) {
            if let Self::V4(socket) = self {
                let mut packet_iter = ipv4_packet_iter(&mut socket.rx);
                loop {
                    // Handle packets until timeout
                    let option = packet_iter
                        .next_with_timeout(timeout)
                        .map_err(TracerouteError::Io)?;

                    let packet = match option {
                        None => break, // We didn't see any probes
                        Some((payload, _ip)) => payload,
                    };

                    // The moment we awknoledge the packet is received
                    let instant = Instant::now();

                    // Check for new probes
                    while let Ok((id, sender)) = probe_receiver.try_recv() {
                        probes.insert(id, sender);
                    }

                    if let Ok((source, id, checksum)) = handle_ipv4_packet(packet) {
                        // Match packet and return
                        match probes.remove(&id) {
                            Some(activity_sender) => {
                                let activity = TraceActivity::Response(TraceResponse::Found(
                                    ProbeResponse::new(source, id, checksum, instant),
                                ));
                                // If sender is closed there isn't anything we can do about it here
                                let _ = activity_sender.send(Ok(activity));
                            }
                            None => {
                                debug!("Received packet not found in probes");
                            }
                        };
                    };
                }

                // Send unresponsive response for unseen probes
                for (id, activity_sender) in probes.drain().take(1) {
                    // If sender is closed there isn't anything we can do about it here
                    let _ = activity_sender
                        .send(Ok(TraceActivity::Response(TraceResponse::NotReceived(id))));
                }
            }
        }
        Ok(())
    }
}
