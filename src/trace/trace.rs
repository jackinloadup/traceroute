use super::{TraceOptions, TraceRequest, TraceResponse, TraceResult};

use crate::packet::{PacketBuilder, PacketBuilderTrait};
use crate::prelude::*;
use crate::probe::ProbeBundle;
use crate::utils::Hop;
use crate::TracerouteError;

use async_std::{
    pin::Pin,
    stream::Stream,
    task::{Context, Poll},
};
use log::debug;
use pnet::packet::ipv4::Ipv4Packet;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::iter::Iterator;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::time::Duration;

/// Perform trace from a source to destination
#[derive(Debug)]
pub struct Trace {
    source: IpAddr,
    destination: IpAddr,
    options: TraceOptions,
    probes_sent: usize,
    packet_sender: Sender<TraceRequest<'static>>,
    activity_receiver: Receiver<TraceResult>,
    queue: Vec<Option<Hop>>,
    // each time we start a trace increment and use invocations with packet building to know which
    // packets were with which round of sending. Is this an issue?
    round: usize,
    completed: usize,
}

impl Trace {
    pub fn new(
        options: TraceOptions,
        source: IpAddr,
        destination: IpAddr,
        packet_sender: Sender<TraceRequest<'static>>,
    ) -> Result<Self, TracerouteError> {
        match (source, destination) {
            (IpAddr::V4(_), IpAddr::V4(_)) => {}
            (IpAddr::V6(_), IpAddr::V6(_)) => Err(TracerouteError::NoIpv6)?,
            _ => Err(TracerouteError::IpProtocolMismatch)?,
        };

        // Create channel to have something to put into the new Trace
        // Drop the sender. We will create a new channel on the start of each trace
        // thus this receiver is really just filling space for on new
        let (_sender, activity_receiver) = channel();

        // convert ttl to usize
        let max_ttl: usize = options.max_ttl.into();

        // init the queue vec
        let queue = vec![None; max_ttl];

        Ok(Self {
            source,
            destination,
            options,
            probes_sent: 0,
            packet_sender,
            activity_receiver,
            queue,
            round: 0,
            completed: 0,
        })
    }

    fn ipv4_probe_request(
        &mut self,
        activity_sender: Sender<TraceResult>,
        source: Ipv4Addr,
        destination: Ipv4Addr,
    ) -> Result<usize, TracerouteError> {
        // Send activity of masked ttls
        for ttl in self.options.get_masked() {
            self.insert_response(ttl, Hop::Masked);
        }

        let Self {
            packet_sender,
            options,
            ..
        } = self;

        // Get a list of all distances we are trying to probe
        let range = options.get_ttl_range();
        let TraceOptions { protocol, .. } = options;

        // Build packets and place them into probe bundles
        let bundles: Vec<ProbeBundle<Ipv4Packet<'static>>> = range
            .iter()
            .map(|ttl| PacketBuilder::build(*protocol, source, destination, *ttl))
            .collect::<Result<_, TracerouteError>>()?;

        // Record how many probes we sent before we loose bundles
        let probes_sent = bundles.len();
        // Create a package for the packet sender
        let request = TraceRequest::V4 {
            bundles,
            timeout: Duration::from_millis(options.timeout.into()),
            activity_sender,
        };
        packet_sender.send(request)?;
        Ok(probes_sent)
    }

    pub fn probes_sent(&self) -> usize {
        self.probes_sent
    }

    fn insert_response(&mut self, ttl: TTL, hop: Hop) {
        let index: usize = (ttl - 1).into();
        debug!("len {:?} index {:?}", self.queue.len(), index);
        let _ = match self.queue.get_mut(index) {
            Some(option) => option.insert(hop),
            None => todo!("queue isn't big enough"),
        };
    }

    fn collect_results(&mut self) -> Vec<Hop> {
        let hops: Vec<Hop> = self
            .queue
            .iter_mut()
            .map(|option| option.take())
            .filter(|option| option.is_some())
            .map(|option| option.unwrap())
            .collect();

        hops
    }

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
    }
}

impl PartialEq for Trace {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source
            && self.destination == other.destination
            && self.options == other.options
    }
}
impl Eq for Trace {}

impl Iterator for Trace {
    type Item = Result<Vec<Hop>, TracerouteError>;

    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            round,
            completed,
            source,
            destination,
            ..
        } = self;

        // start new round
        if round == completed {
            *round += 1;

            let (activity_sender, activity_receiver) = channel();
            self.activity_receiver = activity_receiver;

            let source = source.clone();
            let destination = destination.clone();

            let sending_probes_result = match (source, destination) {
                (IpAddr::V4(source), IpAddr::V4(destination)) => {
                    self.ipv4_probe_request(activity_sender, source, destination)
                }
                (IpAddr::V6(_source), IpAddr::V6(_destination)) => Err(TracerouteError::NoIpv6),
                _ => Err(TracerouteError::IpProtocolMismatch),
            };
            match sending_probes_result {
                Ok(probes_sent) => {
                    self.probes_sent += probes_sent;
                    return None;
                }
                Err(err) => return Some(Err(err)),
            }
        }

        // handle all activity in the channel
        loop {
            let trace_result = match self.activity_receiver.try_recv() {
                Ok(result) => result,
                Err(err) => match err {
                    TryRecvError::Empty => {
                        return None;
                    }
                    TryRecvError::Disconnected => {
                        self.completed += 1;
                        return Some(Ok(self.collect_results()));
                    }
                },
            };
            let response = match trace_result {
                Ok(response) => response,
                Err(err) => return Some(Err(err)),
            };

            match response {
                TraceResponse::Received(response) => {
                    let hop = Hop::Received(response.destination, response.ping);
                    self.insert_response(response.ttl, hop);
                }
                TraceResponse::TimedOut(probe_sent) => {
                    self.insert_response(probe_sent.ttl, Hop::TimedOut);
                }
            };
        }
    }
}

impl Stream for Trace {
    type Item = Result<Vec<Hop>, TracerouteError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let result = match self.get_mut().next() {
            Some(result) => result,
            None => {
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
        };
        match result {
            Ok(hops) => {
                cx.waker().wake_by_ref();
                Poll::Ready(Some(Ok(hops)))
            }
            Err(err) => Poll::Ready(Some(Err(err))),
        }
    }
}
