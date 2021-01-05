use petgraph::dot::Dot;
use petgraph::graphmap::DiGraphMap;
use crate::utils::{Node, Edge};
use std::io::Write;
use std::path::PathBuf;
use std::fs::File;


pub struct TracerouteResults {
    //flows: &flow_map_t,
    //min_ttl: u8,
    //compressed: bool,
    //broken_nat: bool,
    //use_srcport_for_path_generation: bool,
    graph: DiGraphMap<Node, Edge>,
}

impl TracerouteResults {
    pub fn new(graph: DiGraphMap<Node, Edge>) -> TracerouteResults {
        TracerouteResults {
            //min_ttl: 1,
            //compressed: false,
            //broken_nat: false,
            //use_srcport_for_path_generation: false,
            graph,
        }
    }
    pub fn as_string(&self) -> String {
        format!("{}", Dot::new(&self.graph))
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
    pub fn to_json() {
    }
    pub fn show() {
    }
}
