use crate::probe::TcpId;
use crate::{ProbeBundle, ProbeRequest, TraceActivity, TraceResult, TracerouteError};
use core::sync::atomic::{AtomicBool, Ordering};
use log::*;
use pnet::packet::ipv4::Ipv4Packet;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};

use pnet::transport::TransportSender;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

trait SocketSenderTrait<I, P> {
    fn send_packet(&mut self, packet: P, destination: I) -> Result<usize, TracerouteError>;
}

pub struct SocketSender<I> {
    pub addresses: Vec<I>,
    tx: TransportSender,
}

impl SocketSender<Ipv4Addr> {
    pub fn new(addresses: Vec<Ipv4Addr>, tx: TransportSender) -> Self {
        Self { addresses, tx }
    }
}

pub enum SocketSenders {
    V4(SocketSender<Ipv4Addr>),
    V6(SocketSender<Ipv6Addr>),
    Both {
        v4: SocketSender<Ipv4Addr>,
        v6: SocketSender<Ipv6Addr>,
    },
}

impl SocketSenders {
    pub fn addresses(&self) -> Vec<IpAddr> {
        match *self {
            Self::V4(ref socket) => socket
                .addresses
                .clone()
                .iter()
                .map(|ip| IpAddr::V4(*ip))
                .collect(),
            Self::Both { ref v4, ref v6 } => {
                let mut v4 = v4
                    .addresses
                    .clone()
                    .iter()
                    .map(|ip| IpAddr::V4(*ip))
                    .collect::<Vec<IpAddr>>();
                let mut v6 = v6
                    .addresses
                    .clone()
                    .iter()
                    .map(|ip| IpAddr::V6(*ip))
                    .collect::<Vec<IpAddr>>();
                v4.append(&mut v6);
                v4
            }
            Self::V6(ref socket) => socket
                .addresses
                .clone()
                .iter()
                .map(|ip| IpAddr::V6(*ip))
                .collect(),
        }
    }

    pub fn send(
        &mut self,
        packet_receiver: Receiver<ProbeRequest<'_>>,
        probe_sender: Sender<(TcpId, Sender<TraceResult>)>,
        runnable: Arc<AtomicBool>,
    ) -> Result<(), TracerouteError> {
        while runnable.load(Ordering::SeqCst) {
            let packet_delay = Duration::from_millis(5);
            let probe_request = match packet_receiver.try_recv() {
                Ok(request) => request,
                Err(TryRecvError::Empty) => {
                    thread::sleep(Duration::from_millis(10));
                    continue;
                }
                Err(TryRecvError::Disconnected) => break,
            };

            match probe_request {
                ProbeRequest::V4 {
                    bundles,
                    activity_sender,
                } => {
                    debug!(
                        "Sender has received ProbeRequest with {} packets",
                        bundles.len()
                    );
                    for bundle in bundles {
                        let ProbeBundle { probe, packet } = bundle;

                        probe_sender
                            .send((probe.id, activity_sender.clone()))
                            .map_err(|_| TracerouteError::ChannelUnexpectedlyClosed)?;

                        let dest = packet.get_destination();

                        thread::sleep(packet_delay);

                        self.send_packet(packet, dest)?;

                        let activity = TraceActivity::Sent(probe.sent());
                        // If sender is closed there isn't anything we can do about it here
                        let _ = activity_sender.send(Ok(activity));
                    }
                }
                ProbeRequest::V6 {
                    bundles: _,
                    activity_sender: _,
                } => todo!(),
            }
        }
        Ok(())
    }
}

impl SocketSenderTrait<Ipv4Addr, Ipv4Packet<'_>> for SocketSenders {
    fn send_packet(
        &mut self,
        packet: Ipv4Packet,
        destination: Ipv4Addr,
    ) -> Result<usize, TracerouteError> {
        match self {
            Self::V4(socket) => socket
                .tx
                .send_to(packet, IpAddr::V4(destination))
                .map_err(TracerouteError::Io),
            Self::Both { v4: socket, .. } => socket
                .tx
                .send_to(packet, IpAddr::V4(destination))
                .map_err(TracerouteError::Io),
            Self::V6(_) => unimplemented!(),
        }
    }
}
