use crate::utils::Protocol;
use crate::utils::Probe;

use rand::Rng;
use pnet::datalink::{NetworkInterface, MacAddr};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::{MutableIpv4Packet, Ipv4Packet};
use pnet::packet::udp::MutableUdpPacket;
use pnet::packet::MutablePacket;
use std::net::IpAddr;
use std::net::Ipv4Addr;

use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

pub struct PacketBuilderIpv4 {
    source_ip: Ipv4Addr,
}

// We can identify packets by ip::identifier or ip::udp::checksum

impl PacketBuilderIpv4 {
    pub fn new() -> Self {
        let default_interface = get_available_interfaces()
            .iter()
            .next()
            .expect("no interfaces available")
            .clone();

        let source_ip = default_interface.ips
            .iter()
            .filter(|i| i.is_ipv4())
            .next()
            .expect("Couldn't get interface IPv4 address")
            .ip();

        let source_ip = match source_ip {
            IpAddr::V4(ip) => ip,
            _ => panic!("Not possible to get here"),
        };

        Self { source_ip }
    }

    pub fn build_packet(&mut self, protocol: Protocol, destination_ip: Ipv4Addr, ttl: u8, port: u16) -> (Ipv4Packet ,Probe) {
        let buf = vec![0u8; 66]; // FIXME length of 66 is from libtraceroute

        // Generate random IPv4 packet id
        let ip_id = rand::thread_rng().gen();

        let mut ip_header = MutableIpv4Packet::owned(buf).unwrap();

        ip_header.set_version(4);
        ip_header.set_header_length(5);
        ip_header.set_total_length(52);
        ip_header.set_ttl(ttl);
        ip_header.set_next_level_protocol(IpNextHeaderProtocols::Udp);
        ip_header.set_source(self.source_ip);
        ip_header.set_destination(destination_ip);
        ip_header.set_identification(ip_id);
        ip_header.set_checksum(pnet::packet::ipv4::checksum(&ip_header.to_immutable()));

        //let source_port = rand::thread_rng().gen_range(49152, 65535);
        let source_port = 49153;

        let mut hasher = DefaultHasher::new();
        hasher.write_u8(ip_header.get_dscp());
        hasher.write_u8(ip_header.get_ecn());
        for octet in self.source_ip.octets().to_vec() {
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
            Protocol::UDP => Self::build_ipv4_udp_packet(self, &mut ip_header, &destination_ip, port, source_port),
            Protocol::TCP => unimplemented!("Can't build TCP packets yet"),
            Protocol::ICMP => unimplemented!("Can't build ICMP packets yet"),
            Protocol::DCCP => unimplemented!("Can't build DCCP packets yet"),
        };

        (ip_header.consume_to_immutable(), Probe::new(ttl, ip_id, checksum, flowhash))
    }

    fn build_ipv4_udp_packet(&self, ip_header: &mut MutableIpv4Packet, destination_ip: &Ipv4Addr, port: u16, source_port: u16) -> u16 {
        let mut udp_header = MutableUdpPacket::new(ip_header.payload_mut()).unwrap();

        udp_header.set_source(source_port);
        udp_header.set_destination(port);
        udp_header.set_length(32 as u16);
        udp_header.set_payload(&[0; 24]);

        let checksum = pnet::packet::udp::ipv4_checksum(
            &udp_header.to_immutable(),
            &self.source_ip,
            &destination_ip);
        udp_header.set_checksum(checksum);

        checksum
    }

    pub fn ip(&self) -> IpAddr {
        IpAddr::V4(self.source_ip)
    }
}

/// Returns the list of interfaces that are up, not loopback, not point-to-point,
/// and have an IPv4 address associated with them.
pub fn get_available_interfaces() -> Vec<NetworkInterface> {
    let all_interfaces = pnet::datalink::interfaces();

    let available_interfaces: Vec<NetworkInterface>;

    available_interfaces = if cfg!(target_family = "windows") {
        all_interfaces
            .into_iter()
            .filter(|e| e.mac.is_some()
                && e.mac.unwrap() != MacAddr::zero()
                && e.ips
                .iter()
                .filter(|ip| ip.ip().to_string() != "0.0.0.0")
                .next().is_some())
            .collect()
    } else {
        all_interfaces
            .into_iter()
            .filter(|e| e.is_up()
                && !e.is_loopback()
                && e.ips.iter().filter(|ip| ip.is_ipv4()).next().is_some()
                && e.mac.is_some()
                && e.mac.unwrap() != MacAddr::zero())
            .collect()
    };

    available_interfaces
}
