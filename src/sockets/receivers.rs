use crate::prelude::*;
use crate::probe::{ProbeResponse, ProbeSent};
use crate::trace::{TraceResponse, TraceResult, TraceSent};
use crate::utils::handle_ipv4_packet;
use core::sync::atomic::{AtomicBool, Ordering};
use log::*;
use pnet::transport::{TransportReceiver, ipv4_packet_iter};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;
use std::time::Instant;

// How long to wait for new packets from the outside world before breaking the loop and
// allowing other things to happen
const RECEIVE_TIMEOUT: Duration = Duration::from_micros(100);

// How long we will wait for an unmatched packet to stay around before dropping them
const UNMATCHED_PACKETS_TIMEOUT: Duration = Duration::from_secs(10);

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
type FlowMap = HashMap<Flowhash, (Duration, Sender<TraceResult>)>;
type ProbeMap = HashMap<TcpId, ProbeSent>;
type PacketMap = HashMap<TcpId, (IpAddr, Instant)>;

impl SocketReceivers {
    pub fn receive(
        &mut self,
        probe_receiver: Receiver<TraceSent>,
        runnable: Arc<AtomicBool>,
    ) -> Result<(), TracerouteError> {
        // Flows and their connection back to the requester
        let mut flows: FlowMap = HashMap::new();
        // Probes awaiting responses from the network
        let mut probes: ProbeMap = HashMap::new();
        // Packets received without a matching probe
        let mut unmatched_packets: PacketMap = HashMap::new();

        while runnable.load(Ordering::SeqCst) {
            //debug!("num flows {}; num probes {};", flows.len(), probes.len());
            if let Self::V4(socket) = self {
                let mut packet_iter = ipv4_packet_iter(&mut socket.rx);
                loop {
                    // Aggressively handle new probes sent
                    while let Ok(trace_sent) = probe_receiver.try_recv() {
                        let TraceSent {
                            probes: sent_probes,
                            timeout,
                            activity_sender,
                        } = trace_sent;

                        debug!(
                            "Receiver has received TraceSent with {} probes",
                            sent_probes.len()
                        );

                        let flowhash = sent_probes.first().unwrap().flowhash;

                        for sent in sent_probes {
                            // Was this packet seen before the TraceSent package got here
                            // IRL packets from immediate router could respond faster
                            //
                            // source from the unmatched packet is the would be a destination
                            // from this machine perspective
                            if let Some((source, instant)) = unmatched_packets.remove(&sent.id) {
                                let activity = TraceResponse::Received(ProbeResponse::new(
                                    sent, source, instant,
                                ));

                                // If sender is closed there isn't anything we can do about it here
                                let _ = activity_sender.send(Ok(activity));
                            }
                            // watch for probe in the future
                            else {
                                let _ = probes.insert(sent.id, sent);
                            }
                        }

                        let _ = flows.insert(flowhash, (timeout, activity_sender));
                    }

                    // Grab packets until timeout
                    let option = packet_iter
                        .next_with_timeout(RECEIVE_TIMEOUT)
                        .map_err(TracerouteError::Io)?;

                    // Did we time out
                    let packet = match option {
                        None => break, // We didn't see any probes
                        Some((payload, _ip)) => payload,
                    };

                    // The moment we acknowledge the packet is received
                    let instant = Instant::now();

                    //
                    let (source, id, _checksum) = match handle_ipv4_packet(packet) {
                        Ok(data) => data,
                        Err(err) => todo!("traceroute error relating to packet parsing, {}", err),
                    };

                    // Match packet and return
                    match probes.remove(&id) {
                        Some(sent) => {
                            let (_timeout, sender) = flows.get(&sent.flowhash).unwrap();

                            let activity =
                                TraceResponse::Received(ProbeResponse::new(sent, source, instant));

                            // If sender is closed there isn't anything we can do about it here
                            let _ = sender.send(Ok(activity));
                        }
                        None => {
                            debug!("Received packet not found in probes from {}", source);
                            // store packet to see if a TraceSent comes to claim it
                            let _ = unmatched_packets.insert(id, (source, instant));
                        }
                    };
                }

                let now = Instant::now();

                remove_expired_unmatched_packets(&now, &mut unmatched_packets);

                remove_timed_out_probes(&now, &mut probes, &flows);

                remove_empty_flows(&probes, &mut flows);
            }
        }
        Ok(())
    }
}

fn remove_empty_flows(probes: &ProbeMap, flows: &mut FlowMap) {
    // identify flows there are no probes for
    let flows_to_remove: Vec<Flowhash> = flows
        .keys()
        .filter(|flowhash| {
            // count probes with a specific flowhash
            let count = probes
                .values()
                .filter(|sent| sent.flowhash == **flowhash)
                .count();
            count == 0
        })
        // take copy of flowhash to eliminate borrowing
        .map(|flowhash| *flowhash)
        .collect();
    // close flows we don't have probes for
    for flowhash in flows_to_remove {
        let _ = flows.remove(&flowhash);
    }
}

fn remove_timed_out_probes(now: &Instant, probes: &mut ProbeMap, flows: &FlowMap) {
    // Check for probes which have timed out and collect their ids
    let probes_to_remove: Option<Vec<TcpId>> = probes
        .iter()
        .map(|(id, sent)| {
            let (timeout, _sender) = flows.get(&sent.flowhash).unwrap();

            if now.duration_since(sent.instant) > *timeout {
                return Some(*id);
            }
            None
        })
        .collect();
    // remove probes that have expired
    if let Some(probes_to_remove) = probes_to_remove {
        for id in probes_to_remove {
            // remove probe and grab the owned value
            let sent = probes
                .remove(&id)
                .expect("couldn't find probe seen moments ago");

            // get the sender
            let (_timeout, sender) = flows.get(&sent.flowhash).unwrap();
            // Send unresponsive response for unseen probes
            // If sender is closed there isn't anything we can do about it here
            let _ = sender.send(Ok(TraceResponse::TimedOut(sent)));
        }
    }
}

fn remove_expired_unmatched_packets(now: &Instant, unmatched_packets: &mut PacketMap) {
    // remove unmatched packets that have lingered around too long
    let packets_to_remove: Option<Vec<TcpId>> = unmatched_packets
        .iter()
        .map(|(id, (_source, received))| {
            if now.duration_since(*received) > UNMATCHED_PACKETS_TIMEOUT {
                return Some(*id);
            }
            None
        })
        .collect();
    // remove packets that have expired
    if let Some(packets_to_remove) = packets_to_remove {
        for id in packets_to_remove {
            // remove probe and grab the owned value
            let _ = unmatched_packets
                .remove(&id)
                .expect("couldn't find packet seen moments ago");
        }
    }
}
