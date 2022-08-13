pub mod hop;
mod protocol;

pub use hop::Hop;
pub use protocol::Protocol;

use crate::TracerouteError;

use pnet::datalink::{MacAddr, NetworkInterface};
use std::net::{IpAddr, Ipv4Addr};

use pnet::packet::icmp::{IcmpPacket, IcmpTypes};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;
use std::io;

//use std::collections::HashMap;

pub fn get_default_source_ip() -> Result<Ipv4Addr, TracerouteError> {
    let default_interface = get_available_interfaces()
        .first()
        .ok_or_else(|| {
            // io error may not be right for here maybe just a new traceroute error
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
        _ => unreachable!("ipv6 addresses have already been filtered out"),
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
                e.is_lower_up()
                    && !e.is_loopback()
                    && e.ips.iter().any(|ip| ip.is_ipv4())
                    && e.mac.is_some()
                    && e.mac.unwrap() != MacAddr::zero()
            })
            .collect()
    }
}

/// Unpack the incoming payload from an ICMP packet
/// This payload should be the payload we sent to the destination via the echo request
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

/// Process incoming ICMP packet and handle unexpected results
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

/// Processes incoming IPv4 packet and passes it on to transport layer packet handler.
pub fn handle_ipv4_packet(header: Ipv4Packet) -> Result<(IpAddr, u16, u16), TracerouteError> {
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
    Ok((source, id, checksum))
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
