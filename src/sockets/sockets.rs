use crate::sockets::{SocketReceiver, SocketReceivers, SocketSender, SocketSenders};
use crate::utils::get_default_source_ip;
use crate::{ProbeRequest, TracerouteError};
use core::sync::atomic::{AtomicBool, Ordering};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::transport::transport_channel;
use pnet::transport::TransportChannelType::Layer3;
use std::net::{IpAddr, Ipv6Addr};
use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;

/// Creates network sockets to handle egress and ingress packets
pub struct Sockets {
    addresses: Vec<IpAddr>,
    send_handle: Option<JoinHandle<Result<(), TracerouteError>>>,
    receive_handle: Option<JoinHandle<Result<(), TracerouteError>>>,
    runnable: Arc<AtomicBool>,
    packet_sender: Sender<ProbeRequest<'static>>,
}

impl Sockets {
    pub fn new() -> Result<Self, TracerouteError> {
        let runnable = Arc::new(AtomicBool::new(true));

        let (mut tx, mut rx) = Self::setup_sockets()?;
        let addresses = tx.addresses();
        let (packet_sender, packet_receiver) = channel();
        let (probe_sender, probe_receiver) = channel();

        let run = runnable.clone();
        let send_handle = thread::Builder::new()
            .name("send".to_string())
            .spawn(move || tx.send(packet_receiver, probe_sender, run))
            .map_err(TracerouteError::Io)?;

        let run = runnable.clone();
        let receive_handle = thread::Builder::new()
            .name("receive".to_string())
            .spawn(move || rx.receive(probe_receiver, run))
            .map_err(TracerouteError::Io)?;

        Ok(Self {
            addresses,
            send_handle: Some(send_handle),
            receive_handle: Some(receive_handle),
            runnable,
            packet_sender,
        })
    }

    pub fn addresses(&self) -> &Vec<IpAddr> {
        &self.addresses
    }

    pub fn packet_sender(&self) -> Sender<ProbeRequest<'static>> {
        self.packet_sender.clone()
    }

    fn setup_sockets() -> Result<(SocketSenders, SocketReceivers), TracerouteError> {
        let ipv4_source = get_default_source_ip()?;

        // Set the protocol we are looking to recieve
        let protocol = Layer3(IpNextHeaderProtocols::Icmp);
        let mb_v4socket = transport_channel(4096, protocol).map(|(tx, rx)| {
            (
                SocketSender::new(vec![ipv4_source], tx),
                SocketReceiver::new(rx),
            )
        });

        let mb_v6socket: Result<(SocketSender<Ipv6Addr>, SocketReceiver), TracerouteError> =
            Err(TracerouteError::NoIpv6);

        match (mb_v4socket, mb_v6socket) {
            (Ok(_v4_socket), Ok(_v6_socket)) => Err(TracerouteError::NoIpv6),
            //Ok((
            //    SocketSenders::Both {
            //        v4: v4_socket.0,
            //        v6: v6_socket.0,
            //    },
            //    SocketReceivers::Both {
            //        v4: v4_socket.1,
            //        v6: v6_socket.1,
            //    },
            //)),
            (Ok(v4_socket), Err(_)) => Ok((
                SocketSenders::V4(v4_socket.0),
                SocketReceivers::V4(v4_socket.1),
            )),
            (Err(_), Ok(_v6_socket)) => Err(TracerouteError::NoIpv6),
            //Ok((
            //        SocketSenders::V6(v6_socket.0),
            //        SocketReceivers::V6(v6_socket.1),
            //        )),
            (Err(err), Err(_)) => Err(err)?,
        }
    }
}

impl Drop for Sockets {
    /// Close the network loop and hand out the spawn handle
    fn drop(&mut self) {
        // Tell network loop to stop
        self.runnable.store(false, Ordering::SeqCst);

        // Take the handles and cancel them. The Option<T> is needed so we can pull the handles out
        // here. This is needed as we are borrowing self.
        // It shouldn't be possible for the take to return None
        if let Some(handle) = self.receive_handle.take() {
            handle.join().unwrap();
        }

        if let Some(handle) = self.send_handle.take() {
            handle.join().unwrap();
        }
    }
}
