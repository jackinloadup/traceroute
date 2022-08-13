use crate::packet::{PacketBuilder, PacketBuilderTrait};
use crate::probe::{Probe, ProbeBundle};
use crate::utils::Protocol;
use crate::TracerouteError;

use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::{self, Ipv4Packet, MutableIpv4Packet};
use pnet::packet::udp::{self, MutableUdpPacket};
use pnet::packet::MutablePacket;
use rand::Rng;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::net::Ipv4Addr;

// 24 is the size of the payload we attached to the udp packet;
const IPV4_BUFFER_SIZE: usize =
    MutableIpv4Packet::minimum_packet_size() + MutableUdpPacket::minimum_packet_size() + 24;

impl PacketBuilderTrait<Ipv4Addr, Ipv4Packet<'_>> for PacketBuilder {
    fn build(
        protocol: Protocol,
        source: Ipv4Addr,
        source_port: u16,
        dest: Ipv4Addr,
        dest_port: u16,
        ttl: u8,
    ) -> Result<ProbeBundle<Ipv4Packet<'static>>, TracerouteError> {
        // create buffer for packet to fill into
        let buf = vec![0u8; IPV4_BUFFER_SIZE];
        let mut ip_header =
            MutableIpv4Packet::owned(buf).ok_or(TracerouteError::MalformedPacket)?;

        // Generate random IPv4 packet id
        let ip_id = rand::thread_rng().gen();

        // Return is only for errors
        let _ = set_ip_header_values(&mut ip_header, ttl, protocol, source, dest, ip_id)?;

        let flowhash = flowhash(&ip_header, dest_port, source_port, source, dest);

        let checksum = match protocol {
            Protocol::UDP => {
                build_udp_packet(&source, &mut ip_header, &dest, dest_port, source_port)?
            }
            protocol => Err(TracerouteError::UnimplimentedProtocol(protocol))?,
        };

        let packet = ip_header.consume_to_immutable();
        let probe = Probe::new(ttl, ip_id, checksum, flowhash);

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
    match protocol {
        Protocol::UDP => ip_header.set_next_level_protocol(IpNextHeaderProtocols::Udp),
        protocol => return Err(TracerouteError::UnimplimentedProtocol(protocol)),
    }
    ip_header.set_source(source);
    ip_header.set_destination(dest);
    ip_header.set_identification(ip_id);
    ip_header.set_checksum(ipv4::checksum(&ip_header.to_immutable()));
    Ok(())
}

fn flowhash(
    ip_header: &MutableIpv4Packet,
    dest_port: u16,
    source_port: u16,
    source: Ipv4Addr,
    dest: Ipv4Addr,
) -> u16 {
    let mut hasher = DefaultHasher::new();
    hasher.write_u8(ip_header.get_dscp());
    hasher.write_u8(ip_header.get_ecn());
    hasher.write_u16(dest_port);
    hasher.write_u16(source_port);
    source.hash(&mut hasher);
    dest.hash(&mut hasher);

    // This is a u64 but in dublin_traceroute it's a u16
    // get the hash and cast it from a u64 to u16
    let flowhash = hasher.finish() as u16;
    flowhash
}

fn build_udp_packet(
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
    // Question: Does calulating the packet sizes here a performance impact? or is it all inlined?
    // 8 bytes for the udp header and 24 for the payload
    udp_header.set_length(32_u16);
    udp_header.set_payload(&[0_u8; 24]);

    let checksum = udp::ipv4_checksum(&udp_header.to_immutable(), source, destination_ip);
    udp_header.set_checksum(checksum);

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
