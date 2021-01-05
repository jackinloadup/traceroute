//! Traceroute
//!
//!  Determine latency between self and destination
extern crate pnet;
extern crate resolve;
extern crate petgraph;

pub mod utils;

use url::Host;
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
pub use utils::Hop;
use utils::packet_builder::PacketBuilderIpv4;

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

        let mut graph = DiGraphMap::<Node, Edge>::new();

        let targets = match self.options.target_ips() {
            Ok(ips) => ips,
            Err(e) => {
                eprintln!("{}", e);
                return Err("Failed to get target ips");
            }
        };

        for target in targets {
            let trace_graph = self.trace(&mut tx, &mut rx, target);
            graph.extend(trace_graph.all_edges());
        }

        Ok(TracerouteResults::new(graph))
    }

    /// Run a trace against a specific single target
    pub fn trace(&self, tx: &mut TransportSender, rx: &mut TransportReceiver, target: Ipv4Addr) -> DiGraphMap<Node, Edge> {
        println!("Start trace for {}", target);

        let timeout = Duration::new(1, 0);

        let mut packet_builder = PacketBuilderIpv4::new();

        let mut sent_packets = HashMap::new();

        // Don't bother the host with more probes than are required. We want to be good
        // neighbors
        //let mut target_ttl = None;
        //if Some(ttl) = target_ttl {
        //    max_ttl = ttl;
        //}

        let mut probes = self.sweep(&mut packet_builder, tx, target);
        while let Some(probe) = probes.pop() {
            sent_packets.insert(probe.id, probe);
        }

        let responses = Self::listen(rx, timeout);

        Self::match_packets(sent_packets, responses, packet_builder.ip(), IpAddr::V4(target))
    }

    // TODO what to do when target isn't found
    /// Correlate sent and received packets
    fn match_packets(mut sent: HashMap::<u16, Probe>, recv: Vec<ProbeResponse>, source: IpAddr, target: IpAddr) -> DiGraphMap<Node, Edge> {
        let mut target_ttl = None;
        let mut results = vec![];
        for response in recv {
            if let Some(probe) = sent.remove(&response.id) {
                let hop = Hop::new(probe.ttl, source, probe.instant, response.source, response.instant, probe.flowhash);
                if None == target_ttl && response.source == target {
                    target_ttl = Some(probe.ttl);
                }
                results.push(hop);
            }
        }

        match target_ttl {
            Some(ttl) => println!("Target TTL is {}", ttl),
            None => println!("Target wasn't found"),
        }

        // Loop through unmatch probes
        //for (_, probe) in sent {
        //    if let Some(ttl) = target_ttl {
        //        if probe.ttl > ttl {
        //            break;
        //        }
        //    }

        //    println!("{:?}", probe);
        //}



        let mut graph = DiGraphMap::<Node, Edge>::new();
        let source = graph.add_node(Node::Hop(source));

        results.sort_by(|a, b| a.ttl().cmp(&b.ttl()));
        let mut prev_node = source;
        let mut prev_ttl = 1;

        // for each matched hop
        for hop in results.iter() {
            let ttl = hop.ttl();

            // find any missing hops between this one and the last seen
            let hidden = ttl - prev_ttl;
            for i in 1..hidden {
                let new_node = graph.add_node(Node::Hidden(prev_ttl + i, hop.flowhash()));
                graph.add_edge(prev_node, new_node, Edge::Connected);
                prev_node = new_node;
            }

            let new_node = graph.add_node(Node::Hop(hop.received()));

            // if the last hop was the same distance make don't add an edge
            if new_node == prev_node {
                prev_ttl = ttl;
                continue;
            }
            //graph.add_edge(source, index, Edge::RTT(hop.elapsed()));
            graph.add_edge(prev_node, new_node, Edge::Connected);
            //graph.add_edge(prev_node, new_node, Edge::TTL(hop.ttl()));

            prev_node = new_node;
            prev_ttl = ttl;
        }

        graph
    }

    /// Send all targets one packet
    fn sweep(&self, packet_builder: &mut PacketBuilderIpv4, tx: &mut TransportSender, target: Ipv4Addr) -> Vec<Probe> {
        let Options { min_ttl, max_ttl, delay, .. } = self.options;

        (min_ttl..=max_ttl)
            .into_iter()
            .map(|ttl| {
                let (packet, probe) = packet_builder.build_packet(Protocol::UDP, target, ttl, 33440); 
                tx.send_to(packet, IpAddr::V4(target));
                std::thread::sleep(Duration::new(0, delay as u32));
                probe
            })
            .collect()
    }

    /// Listen for packet responses
    fn listen(rx: &mut TransportReceiver, timeout: Duration) -> Vec<ProbeResponse> {
        let mut results = vec![];
        // Listen for return packets
        let listen_start = Instant::now();
        while let Ok(Some((packet, ip))) = ipv4_packet_iter(rx).next_with_timeout(timeout) {
            // check if we have been listening too long
            if Instant::now().duration_since(listen_start) > timeout {
                break;
            }

            if let Ok(response) = handle_ipv4_packet(packet) {
                results.push(response);
            }

            // Check if we found the target
            //if ip == target {
            //    target_ttl = Some(depth);
            //}
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
        _ => unimplemented!("unknown inner packet type"),
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
        _ => Result::Err("wrong packet icmp")
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

