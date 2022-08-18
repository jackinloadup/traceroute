use super::{SocketReceiver, SocketReceivers, SocketSender, SocketSenders};
use crate::trace::TraceRequest;
use crate::utils::get_default_source_ip;
use crate::TracerouteError;
use core::sync::atomic::{AtomicBool, Ordering};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::transport::transport_channel;
use pnet::transport::TransportChannelType::Layer3;
use std::any::Any;
use std::net::{IpAddr, Ipv6Addr};
use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

pub type SocketJoinResult = Vec<Result<Result<(), TracerouteError>, Box<dyn Any + Send>>>;

/// Creates network sockets to handle egress and ingress packets
pub struct Sockets {
    addresses: Vec<IpAddr>,
    send_handle: Option<JoinHandle<Result<(), TracerouteError>>>,
    receive_handle: Option<JoinHandle<Result<(), TracerouteError>>>,
    runnable: Arc<AtomicBool>,
    packet_sender: Sender<TraceRequest<'static>>,
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

    pub fn packet_sender(&self) -> Sender<TraceRequest<'static>> {
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

    /// Close the network connection
    ///
    /// This must be run before drop to capture any panics that came from the socket threads.
    /// This shouldn't happen but in the case it does there is a way to handle it from within
    /// the application logic and not here.
    pub fn close(&mut self) -> SocketJoinResult {
        // Tell network loop to stop
        self.runnable.store(false, Ordering::SeqCst);

        [&mut self.send_handle, &mut self.receive_handle]
            .iter_mut()
            .filter(|option| option.is_some()) // Only give us threads
            // Take the value leaving None behind
            // Unwrap the value as we know it is Some because we filtered above
            // join the thread
            .map(|option| option.take().unwrap().join())
            .collect()
    }
}
