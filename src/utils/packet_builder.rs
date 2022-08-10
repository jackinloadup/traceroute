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

pub trait PacketBuilderTrait<A, P> {
    fn build(
        protocol: Protocol,
        source: A,
        source_port: u16,
        dest: A,
        dest_port: u16,
        ttl: u8,
    ) -> Result<ProbeBundle<P>, TracerouteError>;
}

pub struct PacketBuilder;

impl PacketBuilderTrait<Ipv4Addr, Ipv4Packet<'_>> for PacketBuilder {
    fn build(
        protocol: Protocol,
        source: Ipv4Addr,
        source_port: u16,
        dest: Ipv4Addr,
        dest_port: u16,
        ttl: u8,
    ) -> Result<ProbeBundle<Ipv4Packet<'static>>, TracerouteError> {
        // 24 is the size of the payload we attached to the udp packet;
        let size =
            MutableIpv4Packet::minimum_packet_size() + MutableUdpPacket::minimum_packet_size() + 24;

        let buf = vec![0u8; size];
        let mut ip_header =
            MutableIpv4Packet::owned(buf).ok_or(TracerouteError::MalformedPacket)?;

        // Generate random IPv4 packet id
        let ip_id = rand::thread_rng().gen();

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

        // IDEA: turn this information into a flow struct which can impl fn to get flowhash or hash of
        // just src/dest which may be useful for graph building
        let mut hasher = DefaultHasher::new();
        hasher.write_u8(ip_header.get_dscp());
        hasher.write_u8(ip_header.get_ecn());
        hasher.write_u16(dest_port);
        hasher.write_u16(source_port);
        source.hash(&mut hasher);
        dest.hash(&mut hasher);

        // This is a u64 but in dublin_traceroute it's a u16
        // get the hash and cast it from a u64 to u8
        let flowhash = hasher.finish() as u16;

        let checksum = match protocol {
            Protocol::UDP => {
                build_ipv4_udp_packet(&source, &mut ip_header, &dest, dest_port, source_port)?
            }
            protocol => Err(TracerouteError::UnimplimentedProtocol(protocol))?,
        };

        let packet = ip_header.consume_to_immutable();
        let probe = Probe::new(ttl, ip_id, checksum, flowhash);

        Ok(ProbeBundle { packet, probe })
    }
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
