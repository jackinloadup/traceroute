use crate::probe::ProbeBundle;
use crate::trace::TraceResult;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::Ipv6Packet;
use std::sync::mpsc::Sender;

/// Artifact created by [`Trace`](crate::trace::Trace) with the packets it wants to have sent and a channel to receive
/// the [`TraceResult`](TraceResult)'s
pub enum ProbeRequest<'a> {
    V4 {
        bundles: Vec<ProbeBundle<Ipv4Packet<'a>>>,
        activity_sender: Sender<TraceResult>,
    },
    V6 {
        bundles: Vec<ProbeBundle<Ipv6Packet<'a>>>,
        activity_sender: Sender<TraceResult>,
    },
}
