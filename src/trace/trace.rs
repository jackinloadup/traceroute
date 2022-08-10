use crate::trace::{TraceActivity, TraceOptions, TraceResult};
use crate::utils::packet_builder::{PacketBuilder, PacketBuilderTrait};
use crate::utils::Protocol;
use crate::TracerouteError;
use crate::{ProbeBundle, ProbeRequest};

use async_std::{
    pin::Pin,
    stream::Stream,
    task::{Context, Poll},
};
use pnet::packet::ipv4::Ipv4Packet;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};

#[derive(Debug)]
pub enum TraceType {
    V4(Trace<Ipv4Addr>),
    V6(Trace<Ipv6Addr>),
}

impl TraceType {
    pub fn probes_sent(&self) -> usize {
        match self {
            Self::V4(trace) => trace.probes_sent(),
            Self::V6(_trace) => todo!(),
        }
    }
}

// couldn't figure this out yet, maybe wrong way to go about it
//impl Deref for TraceType {
//    type Target = Trace<I>;
//
//    fn deref(&self) -> &Self::Target {
//        match self {
//            Self::V4(trace) => trace,
//            Self::V6(trace) => trace,
//        }
//    }
//}

/// Perform trace from a source to destination
#[derive(Debug)]
pub struct Trace<I> {
    source: I,
    source_port: u16,
    destination: I,
    destination_port: u16,
    probes_sent: usize,
    activity_receiver: Receiver<TraceResult>,
}

impl PartialEq for Trace<Ipv4Addr> {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source
            && self.source_port == other.source_port
            && self.destination == other.destination
            && self.destination_port == other.destination_port
    }
}
impl Eq for Trace<Ipv4Addr> {}

impl Trace<Ipv4Addr> {
    pub fn new(
        options: TraceOptions,
        source: Ipv4Addr,
        destination: Ipv4Addr,
        packet_sender: Sender<ProbeRequest<'_>>,
    ) -> Result<TraceType, TracerouteError> {
        let source_port = options.src_port;
        let destination_port = options.dst_port;
        let (activity_sender, activity_receiver) = channel();
        let probes_sent =
            Self::probe_request(options, packet_sender, activity_sender, source, destination)?;

        Ok(TraceType::V4(Self {
            source,
            source_port,
            destination,
            destination_port,
            probes_sent,
            activity_receiver,
        }))
    }

    fn probe_request(
        options: TraceOptions,
        packet_sender: Sender<ProbeRequest<'_>>,
        activity_sender: Sender<TraceResult>,
        source: Ipv4Addr,
        destination: Ipv4Addr,
    ) -> Result<usize, TracerouteError> {
        let TraceOptions {
            min_ttl,
            max_ttl,
            mask,
            src_port,
            dst_port,
            ..
        } = options;

        // Make a list of all distances we are trying to probe
        let range: Vec<u8> = match mask {
            // Mask out any distances we want to ignore
            Some(vec) => (min_ttl..=max_ttl)
                .into_iter()
                .filter(|ttl| !vec.contains(ttl))
                .collect(),
            // No need to mask
            None => (min_ttl..=max_ttl).into_iter().collect(),
        };

        let bundles: Vec<ProbeBundle<Ipv4Packet<'_>>> = range
            .iter()
            .map(|ttl| {
                PacketBuilder::build(Protocol::UDP, source, src_port, destination, dst_port, *ttl)
            })
            .collect::<Result<_, TracerouteError>>()?;

        // Record how many probes we sent before we loose bundles
        let probes_sent = bundles.len();
        // Create a package for the packet sender
        let request = ProbeRequest::V4 {
            bundles,
            activity_sender,
        };
        packet_sender.send(request).unwrap();
        Ok(probes_sent)
    }

    pub fn probes_sent(&self) -> usize {
        self.probes_sent
    }
}

impl<I> Trace<I>
where
    I: Hash,
{
    pub fn flowhash(&self) -> u16 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish() as u16
    }
}

impl<I> Hash for Trace<I>
where
    I: Hash,
{
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        // dscp
        0u8.hash(state);
        // ecn
        0u8.hash(state);

        self.source.hash(state);
        self.source_port.hash(state);
        self.destination.hash(state);
        self.destination_port.hash(state);
        // min/max ttl and mask could be included here but I'm not sure that makes sense
    }
}

impl<I> Stream for Trace<I> {
    type Item = Result<TraceActivity, TracerouteError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.activity_receiver.try_recv() {
            Ok(activity) => {
                cx.waker().wake_by_ref();
                Poll::Ready(Some(activity))
            }
            Err(err) => match err {
                TryRecvError::Empty => {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
                TryRecvError::Disconnected => Poll::Ready(None),
            },
        }
    }
}
