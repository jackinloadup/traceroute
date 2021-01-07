pub mod edge;
pub mod hop;
pub mod node;
pub mod options;
pub mod packet_builder;
pub mod probe;
pub mod traceroute_results;

pub use edge::Edge;
pub use hop::Hop;
pub use node::Node;
pub use options::Options;
pub use probe::{Probe, ProbeResponse};
pub use traceroute_results::TracerouteResults;

use crate::TracerouteError;

use pnet::datalink::{MacAddr, NetworkInterface};
use std::net::{IpAddr, Ipv4Addr};

use std::io;

//use std::collections::HashMap;

/// Protocol to be used for traceroute
#[derive(Debug)]
pub enum Protocol {
    /// UDP-based traceroute
    UDP,
    /// TCP-based traceroute
    TCP,
    /// ICMP-based traceroute
    ICMP,
    /// DCCP-based traceroute
    DCCP,
}

pub fn get_default_source_ip() -> Result<Ipv4Addr, TracerouteError> {
    let default_interface = get_available_interfaces()
        .get(0)
        .ok_or_else(|| {
            TracerouteError::Io(io::Error::new(
                io::ErrorKind::Other,
                "No interfaces available",
            ))
        })?
        .clone();

    let source_ip = default_interface
        .ips
        .iter()
        .find(|i| i.is_ipv4())
        .ok_or_else(|| {
            TracerouteError::Io(io::Error::new(
                io::ErrorKind::Other,
                "Couldn't get interface IPv4 address",
            ))
        })?
        .ip();

    match source_ip {
        IpAddr::V4(ip) => Ok(ip),
        _ => Err(TracerouteError::Impossible("Not possible to get here")),
    }
}

/// Returns the list of interfaces that are up, not loopback, not point-to-point,
/// and have an IPv4 address associated with them.
pub fn get_available_interfaces() -> Vec<NetworkInterface> {
    let all_interfaces = pnet::datalink::interfaces();

    if cfg!(target_family = "windows") {
        all_interfaces
            .into_iter()
            .filter(|e| {
                e.mac.is_some()
                    && e.mac.unwrap() != MacAddr::zero()
                    && e.ips.iter().any(|ip| ip.ip().to_string() != "0.0.0.0")
            })
            .collect()
    } else {
        all_interfaces
            .into_iter()
            .filter(|e| {
                e.is_up()
                    && !e.is_loopback()
                    && e.ips.iter().any(|ip| ip.is_ipv4())
                    && e.mac.is_some()
                    && e.mac.unwrap() != MacAddr::zero()
            })
            .collect()
    }
}
//pub struct UdpResponse {
//    /// Outer Ip header source
//    source: IpAddr,
//    /// Inner Ip header id
//    id: u16,
//    /// Udp checksum
//    checksum: u16,
//    /// Udp source port
//    src_port: u16,
//    /// Udp dest port
//    dst_port: u16,
//}
//
//impl UdpResponse {
//    pub fn new(
//        source: IpAddr,
//        id: u16,
//        checksum: u16,
//        src_port: u16,
//        dst_port: u16) -> Self {
//        Self { source, id, checksum, src_port, dst_port }
//    }
//}

//type Flows = HashMap<u16, Flow>;
//
//struct Flow {
//    id: u16,
//    hops: Vec<Hop>
//}

// /// A trace to a specific destination
// struct Trace {
//     destination: IpAddr,
//     flows: Vec<Flow>,
//
// }
//
// /// A trace to a specific destination on specific protocol
// struct Flow {
//     interface: NetworkInterface,
//     source_ip: IpAddr,
//     protocol: Protocol,
//     queries: Vec<Query>,
// }

//struct Query {
//    /// Time to Live or how far away is the hop we received the response from
//    ttl: u16,
//    /// The method to be used in probing the node
//    request_method: Protocol,
//    /// Response from the query
//    response: Option<QueryResult>,
//}
//
//struct QueryResult {
//    /// Round Trip Time, how long has it been since we sent the originating packet
//    //rtt: ,
//    /// Address recieved from the query
//    address: IpAddr,
//}
//
//enum PacketState {
//  Unsent,
//  Sent,
//  Recieved,
//}

//enum ProbeType {
//    Min,
//    Max,
//    /// UDP over IPv4
//    UDPv4,
//}
