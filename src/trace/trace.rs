use crate::packet::{PacketBuilder, PacketBuilderTrait};
use crate::protocol::{self, Protocol};
use crate::trace::{TraceActivity, TraceOptions, TraceResult};
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
use std::net::{IpAddr, Ipv4Addr};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};

/// Perform trace from a source to destination
#[derive(Debug)]
pub struct Trace {
    source: IpAddr,
    destination: IpAddr,
    options: TraceOptions,
    probes_sent: usize,
    activity_receiver: Receiver<TraceResult>,
}

impl PartialEq for Trace {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source
            && self.destination == other.destination
            && self.options == other.options
    }
}
impl Eq for Trace {}

impl Trace {
    pub fn new(
        options: TraceOptions,
        source: IpAddr,
        destination: IpAddr,
        packet_sender: Sender<ProbeRequest<'_>>,
    ) -> Result<Self, TracerouteError> {
        let (activity_sender, activity_receiver) = channel();
        let probes_sent = match (source, destination) {
            (IpAddr::V4(source), IpAddr::V4(destination)) => Self::ipv4_probe_request(
                options.clone(),
                packet_sender,
                activity_sender,
                source,
                destination,
            )?,
            (IpAddr::V6(_source), IpAddr::V6(_destination)) => Err(TracerouteError::NoIpv6)?,
            _ => Err(TracerouteError::IpProtocolMismatch)?,
        };

        Ok(Self {
            source,
            destination,
            options,
            probes_sent,
            activity_receiver,
        })
    }

    fn ipv4_probe_request(
        options: TraceOptions,
        packet_sender: Sender<ProbeRequest<'_>>,
        activity_sender: Sender<TraceResult>,
        source: Ipv4Addr,
        destination: Ipv4Addr,
    ) -> Result<usize, TracerouteError> {
        let TraceOptions { protocol, .. } = options;

        // Get a list of all distances we are trying to probe
        let range = options.get_ttl_range();

        // Send activity of masked ttls
        for ttl in options.get_masked() {
            let _ = activity_sender
                .send(Ok(TraceActivity::Masked(ttl)))
                .unwrap();
        }

        // Build packets and place them into probe bundles
        let bundles: Vec<ProbeBundle<Ipv4Packet<'_>>> = range
            .iter()
            .map(|ttl| PacketBuilder::build(protocol, source, destination, *ttl))
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

impl Trace {
    pub fn flowhash(&self) -> u16 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish() as u16
    }
}

impl Hash for Trace {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        // dscp
        0u8.hash(state);
        // ecn
        0u8.hash(state);

        // Port data may not be taken into account for flows depending on the network device vendor
        // and device configuration
        self.source.hash(state);
        self.destination.hash(state);
        self.options.protocol.hash(state);
        // min/max ttl and mask could be included here but I'm not sure that makes sense
    }
}

impl Stream for Trace {
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
