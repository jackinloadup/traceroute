use std::io;
use std::net::IpAddr;
use std::path::PathBuf;
use structopt::StructOpt;
use url::Host;

pub struct TraceParam<A> {
    /// IP address of source
    source: A,
    /// IP address of target
    target: A,
    /// Source port to send packets from
    src_port: u16,
    /// Base destination port to send packets to
    dst_port: u16,
    /// Number of paths to probe
    npaths: u8,
    /// The minimum TTL to probe
    min_ttl: u8,
    /// The maximum TTL to probe. Must be greater than the minimum TTL
    max_ttl: u8,
    /// TTLs to assume Hidden
    mask: Option<Vec<u8>>,
}
