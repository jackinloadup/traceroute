use std::fmt;
use std::net::IpAddr;
use std::time::Duration;

use log::*;
use crate::prelude::Flowhash;
use crate::{Edge, Node};
use crate::{TraceOptions, TracerouteError};
use crate::probe::ProbeResponse;
use crate::trace::{Trace, TraceResponse};
use async_std::stream::StreamExt;
use std::collections::HashMap;

use petgraph::dot::Dot;
use petgraph::Direction::Outgoing;
use petgraph::graphmap::DiGraphMap;
type Graph = DiGraphMap<Node, Edge>;

/// Collect Trace data for visualizing to user
pub struct TraceData {
    options: TraceOptions,
    // Pings accessible via source and destination IP
    pings: HashMap<(IpAddr,IpAddr),Vec<Duration>>,
    // List of all flows we have seen
    flows: Vec<Flowhash>,
    // All endpoints placed into a graph
    graph: Graph,
}

impl TraceData {
    pub fn new( options: TraceOptions) -> Self {
        let graph = Graph::new();
        let pings = HashMap::new();
        let flows = Vec::new();
        Self {
            options,
            pings,
            flows,
            graph,
        }
    }

    pub async fn process(&mut self, mut traces: Vec<Trace>) -> Result<(), TracerouteError> {
        let track_flows = !self.options.dot;

        for trace in &mut traces {
            let responses = match StreamExt::next(trace).await {
                Some(result) => result,
                None => continue,
            }?;

            let iter = responses.iter();

            // Copy iter for peaking values
            let mut peek_iter = iter.clone();

            // Get first response to create some common assets
            let peek_first = loop {
                if let Some(resp) = peek_iter.next() {
                    match resp {
                        TraceResponse::Masked(_ttl) => continue,
                        TraceResponse::TimedOut(_sent) => continue,
                        TraceResponse::Received(resp) => break Some(resp.clone()),
                    }
                }

                break None;
            };

            let peek_resp = match peek_first {
                Some(resp) => resp,
                None => return Ok(()),
            };

            let source = peek_resp.sent.source;
            let flowhash = peek_resp.sent.flowhash;

            let source_node = Node::Hop(source);
            let flow_node = Node::Flow(flowhash);

            if track_flows {
                self.flows.push(flowhash);

                self.graph.add_node(flow_node);
                self.graph.add_edge(flow_node, source_node, Edge::TTL(0));
            }

            let mut prev_node = source_node;

            for response in iter {
                prev_node = match response {
                    TraceResponse::Masked(ttl) => {
                        let ttl = ttl.clone();
                        let new_node = Node::Masked(ttl);

                        if track_flows {
                            // connect node to flow
                            self.graph.add_edge(flow_node, new_node, Edge::TTL(ttl));
                        }

                        // connect prev node to create lineage
                        self.graph.add_edge(prev_node, new_node, Edge::Connected);
                        new_node
                    },
                    TraceResponse::TimedOut(sent) => {
                        let ttl = sent.ttl.clone();
                        let new_node = Node::Hidden(ttl);

                        if track_flows {
                            // connect node to flow
                            self.graph.add_edge(flow_node, new_node, Edge::TTL(ttl));
                        }
                        // connect prev node to create lineage
                        self.graph.add_edge(prev_node, new_node, Edge::Connected);
                        new_node
                    }
                    TraceResponse::Received(resp) => self.handle_received(resp, prev_node, flow_node),
                };
            }
        }

        Ok(())
    }

    fn handle_received(&mut self, resp: &ProbeResponse, prev_node: Node, flow_node: Node) -> Node {
        let track_flows = !self.options.dot;

        let ttl = resp.ttl;
        let source = resp.sent.source;
        let destination = resp.destination;
        let ping = resp.ping;

        info!("{0:>2}. {1:<15} {2:.3?}", ttl, destination, ping);

        self.add_ping(source, destination, ping);

        // Add flow
        let new_node = self.graph.add_node(Node::Hop(destination));

        // Maybe not a good idea. Can merge exact same ping times. Even if unlikely.
        self.graph.add_edge(flow_node, new_node, Edge::RTT(ping));

        if track_flows {
            // connect node to flow
            self.graph.add_edge(flow_node, new_node, Edge::TTL(ttl));
        }

        // connect prev node to create lineage
        if prev_node != new_node {
          self.graph.add_edge(prev_node, new_node, Edge::Connected);
        }

        // Pass new node back to become the new prev_node
        new_node
    }

    // Add ping
    fn add_ping(&mut self, source: IpAddr, destination: IpAddr, ping: Duration) {
        let TraceOptions { max_ttl, ..} = self.options;
        let key = (source, destination);

        match self.pings.get_mut(&key) {
            Some(pings) => {
                if pings.len() == 10 {
                    // remove last item
                    let _ = pings.pop();
                }
                // insert new ping at beginning
                pings.insert(0, ping);
            }
            None => {
                let mut pings = Vec::with_capacity(max_ttl.into());
                pings.insert(0, ping);
                self.pings.insert(key, pings);
            }
        }
    }

    fn get_pings(&self, source: IpAddr, replier: IpAddr) -> Option<&Vec<Duration>> {
        self.pings.get(&(source, replier))
    }
}

impl fmt::Display for TraceData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Print whole graph if requested
        if self.options.dot {
            info!(
                "Graph of results resulting in {} nodes and {} edges",
                self.graph.node_count(),
                self.graph.edge_count()
            );

            return write!(f, "{}", Dot::new(&self.graph))
        }

        // Otherwise print the first flow
        let flow = self.flows[0];

        let edges = self.graph.edges_directed(Node::Flow(flow), Outgoing);
        let mut edges = edges.collect::<Vec<_>>();

        // verify all nodes are in the right order
        edges.sort_by(|(_,_,ttl1),(_,_,ttl2)| ttl1.cmp(ttl2));

        let (_flow, source_hop, _edge) = edges[0];
        let source = match source_hop {
            Node::Hop(ip) => ip,
            _ => return write!(f, ""),
        };

        // remove all hidden/timed out nodes at the end
        loop {
            let pop = edges.pop_if(|(_,node,_)| {
                match node {
                    Node::Hidden(_) => true,
                    _ => false,
                }
            });
            match pop {
                Some(_) => continue,
                None => break,
            }
        };

        for (_flow, hop, edge) in edges {
            let replier = match hop {
                Node::Hop(ip) => ip,
                _ => continue,
            };

            let formatted_durations = self.get_pings(source, replier)
                    .iter()
                    .map(|duration| format!("{:.2?}", duration))
                    .collect::<Vec<String>>()
                    .join(" ");

            if let Edge::TTL(ttl) = edge {
                match writeln!(f, "{:>2}. {:<15} {}", ttl, replier, formatted_durations) {
                    Ok(_) => continue,
                    Err(err) => return Err(err),
                }
            }
        }

        Ok(())
    }
}
