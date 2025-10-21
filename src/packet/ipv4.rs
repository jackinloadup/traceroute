use crate::TracerouteError;
use crate::packet::{PacketBuilder, PacketBuilderTrait};
use crate::probe::{Probe, ProbeBundle};
use crate::protocol::{Protocol, UdpParams};

use pnet::packet::MutablePacket;
use pnet::packet::icmp::{self, IcmpCode, IcmpTypes, MutableIcmpPacket};
use pnet::packet::ipv4::{self, Ipv4Packet, MutableIpv4Packet};
use pnet::packet::udp::{self, MutableUdpPacket};
use rand::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::net::{IpAddr,Ipv4Addr};

// 24 is the size of the payload we attached to the udp packet;
const IPV4_BUFFER_SIZE: usize =
    MutableIpv4Packet::minimum_packet_size() + MutableUdpPacket::minimum_packet_size() + 24;

impl PacketBuilderTrait<Ipv4Addr, Ipv4Packet<'_>> for PacketBuilder {
    fn build(
        protocol: Protocol,
        source: Ipv4Addr,
        dest: Ipv4Addr,
        ttl: u8,
    ) -> Result<ProbeBundle<Ipv4Packet<'static>>, TracerouteError> {
        // Validate if protocol is supported
        let _ = match protocol {
            Protocol::ICMP | Protocol::UDP(_) => (),
            protocol => Err(TracerouteError::UnimplimentedProtocol(protocol))?,
        };

        // Create buffer for packet to fill into
        let buf = vec![0u8; IPV4_BUFFER_SIZE];
        let mut ip_header =
            MutableIpv4Packet::owned(buf).ok_or(TracerouteError::MalformedPacket)?;

        // get source of randomness
        let mut rng = rand::rng();

        // Generate random IPv4 packet id
        let ip_id = rng.random::<u16>();

        // Return is only for errors
        let _ = set_ip_header_values(&mut ip_header, ttl, protocol, source, dest, ip_id)?;

        let (flowhash, checksum) = match protocol {
            Protocol::UDP(params) => {
                let flowhash = flowhash(
                    &ip_header,
                    source,
                    dest,
                    Some(params.source_port),
                    Some(params.destination_port),
                );
                let checksum = build_udp_packet(&mut ip_header, &source, &dest, params)?;
                (flowhash, checksum)
            }
            //Protocol::ICMP => {
            //    let flowhash = flowhash(&ip_header, source, dest, None, None);
            //    let checksum = build_icmp_packet(&mut ip_header)?;
            //    (flowhash, checksum)
            //}
            protocol => Err(TracerouteError::UnimplimentedProtocol(protocol))?,
        };

        let packet = ip_header.consume_to_immutable();
        let probe = Probe::new(IpAddr::V4(source), ttl, ip_id, checksum, flowhash);

        Ok(ProbeBundle { packet, probe })
    }
}

fn set_ip_header_values(
    ip_header: &mut MutableIpv4Packet,
    ttl: u8,
    protocol: Protocol,
    source: Ipv4Addr,
    dest: Ipv4Addr,
    ip_id: u16,
) -> Result<(), TracerouteError> {
    ip_header.set_version(4);
    ip_header.set_header_length(5);
    ip_header.set_dscp(0);
    ip_header.set_ecn(0);
    ip_header.set_total_length(52);
    ip_header.set_ttl(ttl);
    ip_header.set_next_level_protocol(protocol.into());
    ip_header.set_source(source);
    ip_header.set_destination(dest);
    ip_header.set_identification(ip_id);
    ip_header.set_checksum(ipv4::checksum(&ip_header.to_immutable()));
    Ok(())
}

fn flowhash(
    ip_header: &MutableIpv4Packet,
    source: Ipv4Addr,
    dest: Ipv4Addr,
    dest_port: Option<u16>,
    source_port: Option<u16>,
) -> u16 {
    let mut hasher = DefaultHasher::new();
    hasher.write_u8(ip_header.get_dscp());
    hasher.write_u8(ip_header.get_ecn());

    if let Some(source_port) = source_port {
        hasher.write_u16(source_port);
    }
    if let Some(dest_port) = dest_port {
        hasher.write_u16(dest_port);
    }

    source.hash(&mut hasher);
    dest.hash(&mut hasher);

    // This is a u64 but in dublin_traceroute it's a u16
    // get the hash and cast it from a u64 to u16
    let flowhash = hasher.finish() as u16;
    flowhash
}

// Build UDP probe. Response is ICMP packet with UDP packet returned inside
fn build_udp_packet(
    ip_header: &mut MutableIpv4Packet,
    source: &Ipv4Addr,
    destination_ip: &Ipv4Addr,
    params: UdpParams,
) -> Result<u16, TracerouteError> {
    let mut udp_header =
        MutableUdpPacket::new(ip_header.payload_mut()).ok_or(TracerouteError::MalformedPacket)?;

    udp_header.set_source(params.source_port);
    udp_header.set_destination(params.destination_port);
    // 8 bytes for the udp header and 24 for the payload
    udp_header.set_length(32_u16);
    udp_header.set_payload(&[0_u8; 24]);

    let checksum = udp::ipv4_checksum(&udp_header.to_immutable(), source, destination_ip);
    udp_header.set_checksum(checksum);

    Ok(checksum)
}

fn build_icmp_packet(ip_header: &mut MutableIpv4Packet) -> Result<u16, TracerouteError> {
    let mut icmp_header =
        MutableIcmpPacket::new(ip_header.payload_mut()).ok_or(TracerouteError::MalformedPacket)?;

    icmp_header.set_icmp_type(IcmpTypes::EchoRequest);
    icmp_header.set_icmp_code(IcmpCode::new(0));
    icmp_header.set_payload(&[0_u8; 24]);

    let checksum = icmp::checksum(&icmp_header.to_immutable());
    icmp_header.set_checksum(checksum);

    Ok(checksum)
}

//#[cfg(test)]
//mod tests {
//    use super::*;
//
//    #[test]
//    fn test_build_ipv4_probe() -> Result<(), String> {
//        let result = build_ipv4_probe(
//            Protocol::UDP,
//            &Ipv4Addr::LOCALHOST,
//            Ipv4Addr::UNSPECIFIED,
//            1,
//            1,
//        );
//        assert!(result.is_ok());
//        let (packet, probe) = result.unwrap();
//        assert_eq!(probe.ttl, packet.get_ttl());
//
//        Ok(())
//    }
//}
