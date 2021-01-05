pub mod options;
pub mod traceroute_results;
pub mod hop;
pub mod packet_builder;
pub mod probe;
pub mod node;
pub mod edge;

pub use traceroute_results::TracerouteResults;
pub use options::Options;
pub use hop::Hop;
pub use probe::{Probe, ProbeResponse};
pub use node::Node;
pub use edge::Edge;
//use std::net::IpAddr;

//use std::collections::HashMap;

/// Protocol to be used for traceroute
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
