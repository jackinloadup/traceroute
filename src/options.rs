use std::io;
use std::net::IpAddr;
use std::path::PathBuf;
use std::time::Instant;
use log::*;
use structopt::StructOpt;
use crate::protocol::Protocol;
use url::Host;

use resolve::resolve_host;

use crate::TracerouteError;

/// Command line configuration parameters
#[derive(StructOpt, Clone, Debug, Default)]
#[structopt(name = "hollister-traceroute", about)]
pub struct Options {
    /// The network has a broken NAT configuration (e.g. no payload fixup). Try this if you see
    /// fewer hops than expected
    //#[structopt(short, long)]
    //pub broken_nat: bool,
    /// Generate paths using source port instead of destination port
    //#[structopt(short = "i", long = "use-srcport")]
    //pub use_srcport_for_path_generation: bool,
    /// Do not attempt to do reverse DNS lookup of the hops
    //#[structopt(short = "N", long)]
    //pub no_dns: bool,
    /// Source port to send packets from
    #[structopt(short, long, default_value = "12345")]
    pub src_port: u16,
    /// Base destination port to send packets to
    #[structopt(short, long, default_value = "33434")]
    pub dst_port: u16,
    /// The minimum TTL to probe
    #[structopt(short, long, default_value = "1")]
    pub min_ttl: u8,
    /// The maximum TTL to probe. Must be greater than the minimum TTL
    #[structopt(short = "T", long, default_value = "30")]
    pub max_ttl: u8,
    /// The inter-packet delay in milliseconds
    #[structopt(short = "D", long, default_value = "10")]
    pub delay: u16,
    /// Output file name [default: stdout]
    #[structopt(short, long, parse(from_os_str))]
    pub output_file: Option<PathBuf>,
    /// Silence all output
    #[structopt(short = "q", long = "quiet")]
    pub quiet: bool,
    /// Verbose mode (-v, -vv, -vvv, etc)
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    pub verbose: usize,
    /// TTLs to assume Hidden
    #[structopt(long, use_delimiter = true)]
    pub mask: Option<Vec<u8>>,
    /// Hostname or IP address of target
    #[structopt(parse(try_from_str = Host::parse))]
    pub target: Vec<Host>,
    /// Protocol to probe with DCCP, ICMP, SCTP, TCP, UDP
    #[structopt(short, long, default_value = "udp")]
    pub protocol: Protocol,
    /// Output graph in Dot format
    #[structopt(short = "g", long= "graph")]
    pub dot: bool,
}

impl Options {
    /// Gather all IP addresses dictated through options
    pub fn target_ips(&self) -> Result<Vec<IpAddr>, TracerouteError> {
        // @TODO return an iterator for the different targets?
        let mut hosts = vec![];

        if self.target.is_empty() {
            let err = io::Error::new(io::ErrorKind::Other, "No target given");
            return Err(TracerouteError::Io(err));
        }

        for host in &self.target {
            match host {
                Host::Ipv4(ip) => hosts.push(IpAddr::V4(*ip)),
                Host::Ipv6(ip) => hosts.push(IpAddr::V6(*ip)),
                Host::Domain(domain) => Self::resolve_domain(domain, &mut hosts)?,
            }
        }

        Ok(hosts)
    }

    // Get IP addresses associated with domain
    fn resolve_domain(domain: &str, hosts: &mut Vec<IpAddr>) -> Result<(), TracerouteError> {
        let start = Instant::now();

        for addr in resolve_host(&domain)? {
            match addr {
                IpAddr::V4(ip) => hosts.push(IpAddr::V4(ip)),
                IpAddr::V6(ip) => hosts.push(IpAddr::V6(ip)),
            }
        }
        let duration = Instant::now().duration_since(start);
        info!(
            "DNS resolution for {} took {:.3?} and found {} hosts",
            domain,
            duration,
            hosts.len()
        );
        Ok(())
    }

    pub fn get_masked(&self) -> Vec<u8> {
        self.mask.to_owned().unwrap_or_default()
    }
}
