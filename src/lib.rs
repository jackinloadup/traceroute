//! Traceroute
//!
//!  Determine latency between self and destination
extern crate petgraph;
extern crate pnet;
extern crate resolve;

pub mod utils;

use std::time::Duration;
use std::time::Instant;

use pnet::packet::icmp::{IcmpPacket, IcmpType, IcmpTypes};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;

use pnet::transport::TransportChannelType::Layer3;
use pnet::transport::{ipv4_packet_iter, transport_channel, TransportReceiver, TransportSender};
use std::collections::HashMap;

use std::net::IpAddr;
use std::net::Ipv4Addr;

use std::io;

use std::thread::sleep;

use utils::get_default_source_ip;
use utils::packet_builder::build_ipv4_probe;
pub use utils::Options;
pub use utils::Protocol;
pub use utils::TracerouteResults;
pub use utils::{Probe, ProbeResponse};

#[derive(Debug)]
pub enum TracerouteError {
    Io(io::Error),
    UnmatchedPacket(&'static str),
    ICMPTypeUnexpected(IcmpType),
    PacketDecode,
    MalformedPacket,
    NoIpv6,
    Impossible(&'static str),
    UnimplimentedProtocol(Protocol),
}

/// Provides management interface for traceroute
pub struct Traceroute {
    /// Configuration options which dictate highlevel behavior
    options: Options,
}

impl Traceroute {
    pub fn new(options: Options) -> Self {
        Self { options }
    }

    /// Run a contained traceroute against the target/s specified in options
    pub fn run(&mut self) -> Result<TracerouteResults, TracerouteError> {
        // Lock to ensure traceroute isn't running at the same time as another

        // Set the protocol we are looking to recieve
        let protocol = Layer3(IpNextHeaderProtocols::Icmp);
        let (mut tx, mut rx) = transport_channel(4096, protocol).map_err(TracerouteError::Io)?;

        let targets = self.options.target_ips()?;
        let source = get_default_source_ip()?;

        let default_trace = TracerouteResults::default(IpAddr::V4(source));
        // Run trace against each target and merge results
        targets
            .iter()
            .map(|target| self.trace(&mut tx, &mut rx, source, *target))
            // TODO look into fold_first when it stablizes to eliminate the need for
            // Option<T>
            .fold(Ok(default_trace), |prev, cur| {
                let mut traces = prev?;
                if let Ok(trace) = cur {
                    traces.extend(trace.all_edges());
                }
                Ok(traces)
            })
    }

    /// Run a trace against a specific single target
    pub fn trace(
        &self,
        tx: &mut TransportSender,
        rx: &mut TransportReceiver,
        source: Ipv4Addr,
        target: Ipv4Addr,
    ) -> Result<TracerouteResults, TracerouteError> {
        eprintln!("Start trace for {}", target);

        let probes = self.sweep(&source, tx, target)?;

        let timeout = Duration::new(1, 0);
        let responses = Self::listen(rx, timeout);

        let masked = self.options.get_masked();

        Ok(TracerouteResults::new(
            probes,
            responses,
            IpAddr::V4(source),
            IpAddr::V4(target),
            masked,
        ))
    }

    /// Send all targets one packet
    fn sweep(
        &self,
        source: &Ipv4Addr,
        tx: &mut TransportSender,
        target: Ipv4Addr,
    ) -> Result<HashMap<u16, Probe>, TracerouteError> {
        let Options {
            delay,
            min_ttl,
            max_ttl,
            ..
        } = self.options;

        (min_ttl..=max_ttl)
            .into_iter()
            .filter(|i| !self.options.get_masked().contains(i))
            .map(|ttl| build_ipv4_probe(Protocol::UDP, source, target, ttl, 33440))
            .map(|results| {
                let (packet, probe) = results?;
                sleep(Duration::from_millis(delay as u64));

                tx.send_to(packet, IpAddr::V4(target))
                    .map_err(TracerouteError::Io)
                    .and(Ok(probe))
            })
            .fold(Ok(HashMap::new()), |hashmap, probe: Result<Probe, _>| {
                let probe = probe?;
                hashmap.map(|mut hash| {
                    hash.insert(probe.id, probe);
                    hash
                })
            })
    }

    /// Listen for packet responses
    fn listen(rx: &mut TransportReceiver, timeout: Duration) -> Vec<ProbeResponse> {
        let mut results = vec![];
        // Listen for return packets
        let listen_start = Instant::now();
        while let Ok(Some((packet, _ip))) = ipv4_packet_iter(rx).next_with_timeout(timeout) {
            // check if we have been listening too long
            if Instant::now().duration_since(listen_start) > timeout {
                break;
            }

            if let Ok(response) = handle_ipv4_packet(packet) {
                results.push(response);
            }
        }

        results
    }

    /// Return source port
    pub fn srcport(&self) -> u16 {
        self.options.src_port
    }

    /// Return destination port
    pub fn dstport(&self) -> u16 {
        self.options.dst_port
    }

    /// Number of times we will probe each distance
    pub fn npaths(&self) -> u8 {
        self.options.npaths
    }

    /// Minimum distance we are quering
    pub fn min_ttl(&self) -> u8 {
        self.options.min_ttl
    }

    /// Maximum distance we are quering
    pub fn max_ttl(&self) -> u8 {
        self.options.max_ttl
    }
}

fn unpack_icmp_payload(payload: &[u8]) -> Result<(u16, u16), TracerouteError> {
    let packet = Ipv4Packet::new(&payload[4..]).ok_or(TracerouteError::MalformedPacket)?;
    let id = packet.get_identification();

    let checksum = match packet.get_next_level_protocol() {
        IpNextHeaderProtocols::Udp => UdpPacket::new(packet.payload())
            .ok_or(TracerouteError::MalformedPacket)?
            .get_checksum(),
        _ => {
            return Err(TracerouteError::UnmatchedPacket(
                "icmp response was of an unknown type",
            ))
        }
    };
    Ok((id, checksum))
}

fn handle_icmp_packet(packet: &[u8]) -> Result<(u16, u16), TracerouteError> {
    let icmp_packet = IcmpPacket::new(packet).ok_or(TracerouteError::MalformedPacket)?;

    let payload = icmp_packet.payload();

    match icmp_packet.get_icmp_type() {
        IcmpTypes::TimeExceeded | IcmpTypes::EchoReply | IcmpTypes::DestinationUnreachable => {
            unpack_icmp_payload(payload)
        }
        icmp_type => Err(TracerouteError::ICMPTypeUnexpected(icmp_type)),
    }
}

/// Processes IPv4 packet and passes it on to transport layer packet handler.
fn handle_ipv4_packet(header: Ipv4Packet) -> Result<ProbeResponse, TracerouteError> {
    let source = IpAddr::V4(header.get_source());
    let payload = header.payload();

    let (id, checksum) = match header.get_next_level_protocol() {
        IpNextHeaderProtocols::Icmp => handle_icmp_packet(payload)?,
        // Any packets hitting here are actually for another application
        _ => {
            return Err(TracerouteError::UnmatchedPacket(
                "pnet is sending us packets we didn't request",
            ))
        }
    };
    Ok(ProbeResponse::new(source, id, checksum))
}
