//! Traceroute
//!
//!  Determine latency between self and destination
extern crate pnet;
extern crate resolve;
extern crate petgraph;

pub mod utils;

use std::time::Instant;
use std::time::Duration;

use pnet::packet::icmp::{IcmpPacket, IcmpTypes};
use pnet::packet::udp::UdpPacket;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::Packet;

use pnet::transport::{ipv4_packet_iter, transport_channel, TransportSender, TransportReceiver};
use pnet::transport::TransportChannelType::Layer3;
use std::collections::HashMap;

use std::net::IpAddr;
use std::net::Ipv4Addr;

pub use utils::TracerouteResults;
pub use utils::Options;
pub use utils::Protocol;
pub use utils::{Probe, ProbeResponse};
use utils::get_default_source_ip;
use utils::packet_builder::build_ipv4_probe;

use petgraph::graphmap::DiGraphMap;
use utils::{Node, Edge};

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
    pub fn run(&mut self) -> Result<TracerouteResults, &'static str> {
        // Lock to ensure traceroute isn't running at the same time as another

        // Set the protocol we are looking to recieve
        let protocol = Layer3(IpNextHeaderProtocols::Icmp);
        let (mut tx, mut rx) = transport_channel(4096, protocol)
                                    .expect("Can't get channel");

        let targets = match self.options.target_ips() {
            Ok(ips) => ips,
            Err(e) => {
                eprintln!("{}", e);
                return Err("Failed to get target ips");
            }
        };

        let results = targets.iter()
                         .map(|target| self.trace(&mut tx, &mut rx, *target))
                         // TODO look into fold_first when it stablizes
                         .fold(None, |prev: Option<TracerouteResults>, cur| {
                             match prev {
                                 Some(mut graph) => {
                                     if let Ok(trace) = cur {
                                        graph.extend(trace.all_edges());
                                     }
                                     Some(graph)
                                 }
                                 None => cur.ok(),
                             }
                         });


        results.ok_or("Idk, it failed")
    }

    /// Run a trace against a specific single target
    pub fn trace(&self, tx: &mut TransportSender, rx: &mut TransportReceiver, target: Ipv4Addr) -> Result<TracerouteResults, &'static str> {
        eprintln!("Start trace for {}", target);

        let source = get_default_source_ip();

        let probes = match self.sweep(&source, tx, target) {
            Ok(probes) => probes,
            Err(e) => return Err(e),
        };

        let timeout = Duration::new(1, 0);
        let responses = Self::listen(rx, timeout);

        let masked = self.options.get_masked();

        Ok(TracerouteResults::new(probes, responses, IpAddr::V4(source), IpAddr::V4(target), masked))
    }


    /// Send all targets one packet
    fn sweep(&self, source: &Ipv4Addr, tx: &mut TransportSender, target: Ipv4Addr) -> Result<HashMap<u16, Probe>, &'static str> {
        let Options { delay, min_ttl, max_ttl, .. } = self.options;

        (min_ttl..=max_ttl)
            .into_iter()
            .filter(|i| !self.options.get_masked().contains(i))
            .map(|ttl| build_ipv4_probe(Protocol::UDP, source, target, ttl, 33440))
            .map(|(packet, probe)| {
                std::thread::sleep(Duration::new(0, delay as u32));

                tx.send_to(packet, IpAddr::V4(target))
                    .and(Ok(probe))
            })
            .fold(Ok(HashMap::new()), |hashmap, probe: Result<Probe, _>| {
               hashmap.map(|mut hash| {
                   let probe = probe.unwrap();
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


fn unpack_icmp_payload(payload: &[u8]) -> Result<(u16, u16), &'static str> {
    let packet = Ipv4Packet::new(&payload[4..]).expect("malformed Ipv4 packet");
    let id = packet.get_identification();

    let checksum = match packet.get_next_level_protocol() {
        IpNextHeaderProtocols::Udp => {
            UdpPacket::new(packet.payload())
                .unwrap()
                .get_checksum()
        }
        _ => return Err("unknown inner packet type"),
    };
    Ok((id, checksum))
}

fn handle_icmp_packet(packet: &[u8]) -> Result<(u16, u16), &'static str> {
    let icmp_packet = IcmpPacket::new(packet).expect("malformed ICMP packet");
    let payload = icmp_packet.payload();

    match icmp_packet.get_icmp_type() {
        IcmpTypes::TimeExceeded |
        IcmpTypes::EchoReply |
        IcmpTypes::DestinationUnreachable => {
            unpack_icmp_payload(payload)
        },
        _ => Err("wrong packet icmp")
    }

}

/// Processes IPv4 packet and passes it on to transport layer packet handler.
fn handle_ipv4_packet(packet: Ipv4Packet) -> Result<ProbeResponse, &'static str> {
    //let header = Ipv4Packet::new(packet).expect("malformed IPv4 packet");
    let header = packet;

    let source = IpAddr::V4(header.get_source());
    let payload = header.payload();

    let (id, checksum) = match header.get_next_level_protocol() {
        IpNextHeaderProtocols::Icmp => handle_icmp_packet(payload).unwrap(),
        // Any packets hitting here are actually for another application
        _ => return Err("wrong packet ipv4"),
    };
    Ok(ProbeResponse::new(source, id, checksum))
}

