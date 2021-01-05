use petgraph::dot::Dot;
use petgraph::graphmap::DiGraphMap;
use crate::utils::{Node, Edge};
use std::io::Write;
use std::path::PathBuf;
use std::fs::File;
use std::string::ToString;
use std::net::IpAddr;
pub use super::{Probe, ProbeResponse};
pub use super::Hop;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

pub struct TracerouteResults {
    //flows: &flow_map_t,
    //min_ttl: u8,
    //compressed: bool,
    //broken_nat: bool,
    //use_srcport_for_path_generation: bool,
    source: IpAddr,
    target: IpAddr,
    graph: DiGraphMap<Node, Edge>,
}

impl TracerouteResults {
    pub fn new(sent: HashMap<u16, Probe>, recv: Vec<ProbeResponse>, source: IpAddr, target: IpAddr, masked: Vec<u8>) -> TracerouteResults {


        // Don't bother the host with more probes than are required. We want to be good
        // neighbors
        //let mut target_ttl = None;
        //if Some(ttl) = target_ttl {
        //    max_ttl = ttl;
        //}

        let graph = Self::match_packets(sent, recv, source, target, masked);
        TracerouteResults {
            //min_ttl: 1,
            //compressed: false,
            //broken_nat: false,
            //use_srcport_for_path_generation: false,
            graph,
            source,
            target,
        }
    }
    pub fn write(&self, file: PathBuf) {
        let dot = Dot::new(&self.graph);
        let mut file = File::create(file).expect("create failed");
        file.write_all(dot.to_string().as_bytes()).expect("write failed");
    }
    pub fn compress() {
    }
    //pub fn flows() -> flow_map_t {
    //}
    //pub fn match_packet(Packet) -> &IpAddr {
    //}

    // TODO what to do when target isn't found
    /// Correlate sent and received packets
    fn match_packets(mut sent: HashMap::<u16, Probe>, recv: Vec<ProbeResponse>, source: IpAddr, target: IpAddr, masked: Vec<u8>) -> DiGraphMap<Node, Edge> {
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
            Some(ttl) => eprintln!("Target TTL is {}", ttl),
            None => eprintln!("Target wasn't found"),
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
                let hidden_ttl = prev_ttl + i;
                let new_node = if masked.contains(&hidden_ttl) {
                    Node::Masked(hidden_ttl)
                } else {
                    Node::Hidden(hidden_ttl, hop.flowhash())
                };
                graph.add_node(new_node);
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
}

impl ToString for TracerouteResults {
    fn to_string(&self) -> String {
        format!("{}", Dot::new(&self.graph))
    }
}

impl Deref for TracerouteResults {
    type Target = DiGraphMap<Node, Edge>;

    fn deref(&self) -> &Self::Target {
        &self.graph
    }
}

impl DerefMut for TracerouteResults {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.graph
    }
}
