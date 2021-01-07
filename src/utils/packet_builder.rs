use crate::utils::Probe;
use crate::utils::Protocol;
use crate::TracerouteError;

use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::{Ipv4Packet, MutableIpv4Packet};
use pnet::packet::udp::MutableUdpPacket;
use pnet::packet::MutablePacket;
use rand::Rng;
use std::net::Ipv4Addr;

use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

pub fn build_ipv4_probe(
    protocol: Protocol,
    source: &Ipv4Addr,
    destination_ip: Ipv4Addr,
    ttl: u8,
    port: u16,
) -> Result<(Ipv4Packet, Probe), TracerouteError> {
    let buf = vec![0u8; 66]; // FIXME length of 66 is from libtraceroute

    // Generate random IPv4 packet id
    let ip_id = rand::thread_rng().gen();

    let mut ip_header = MutableIpv4Packet::owned(buf).ok_or(TracerouteError::MalformedPacket)?;

    ip_header.set_version(4);
    ip_header.set_header_length(5);
    ip_header.set_total_length(52);
    ip_header.set_ttl(ttl);
    ip_header.set_next_level_protocol(IpNextHeaderProtocols::Udp);
    ip_header.set_source(*source);
    ip_header.set_destination(destination_ip);
    ip_header.set_identification(ip_id);
    ip_header.set_checksum(pnet::packet::ipv4::checksum(&ip_header.to_immutable()));

    //let source_port = rand::thread_rng().gen_range(49152, 65535);
    let source_port = 49153;

    let mut hasher = DefaultHasher::new();
    hasher.write_u8(ip_header.get_dscp());
    hasher.write_u8(ip_header.get_ecn());
    for octet in source.octets().to_vec() {
        hasher.write_u8(octet);
    }
    for octet in destination_ip.octets().to_vec() {
        hasher.write_u8(octet);
    }
    hasher.write_u16(port);
    hasher.write_u16(source_port);

    // This is a u64 but in dublin_traceroute it's a u16
    // get the hash and cast it from a u64 to u8
    let flowhash = hasher.finish() as u16;

    let checksum = match protocol {
        Protocol::UDP => {
            build_ipv4_udp_packet(source, &mut ip_header, &destination_ip, port, source_port)?
        }
        protocol => return Err(TracerouteError::UnimplimentedProtocol(protocol)),
    };

    Ok((
        ip_header.consume_to_immutable(),
        Probe::new(ttl, ip_id, checksum, flowhash),
    ))
}

fn build_ipv4_udp_packet(
    source: &Ipv4Addr,
    ip_header: &mut MutableIpv4Packet,
    destination_ip: &Ipv4Addr,
    port: u16,
    source_port: u16,
) -> Result<u16, TracerouteError> {
    let mut udp_header =
        MutableUdpPacket::new(ip_header.payload_mut()).ok_or(TracerouteError::MalformedPacket)?;

    udp_header.set_source(source_port);
    udp_header.set_destination(port);
    udp_header.set_length(32 as u16);
    udp_header.set_payload(&[0; 24]);

    let checksum =
        pnet::packet::udp::ipv4_checksum(&udp_header.to_immutable(), source, destination_ip);
    udp_header.set_checksum(checksum);

    Ok(checksum)
}
