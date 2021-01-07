use structopt::StructOpt;
use url::Host;
use std::path::PathBuf;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::io;

use resolve::resolver::ResolveHost;
use resolve::resolve_host;

use crate::TracerouteError;

#[derive(StructOpt, Debug)]
#[structopt(name = "traceroute", about)]
pub struct Options {
    /// The network has a broken NAT configuration (e.g. no payload fixup). Try this if you see
    /// fewer hops than expected
    #[structopt(short, long)]
    pub broken_nat: bool,
    /// Generate paths using source port instead of destination port
    #[structopt(short = "i", long = "use-srcport")]
    pub use_srcport_for_path_generation: bool,
    /// Do not attempt to do reverse DNS lookup of the hops
    #[structopt(short = "N", long)]
    pub no_dns: bool,
    /// Source port to send packets from
    #[structopt(short, long, default_value = "12345")]
    pub src_port: u16,
    /// Base destination port to send packets to
    #[structopt(short, long, default_value = "33434")]
    pub dst_port: u16,
    /// Number of paths to probe
    #[structopt(short, long, default_value = "20")]
    pub npaths: u8,
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
    /// TTLs to assume Hidden
    #[structopt(long, use_delimiter = true)]
    mask: Option<Vec<u8>>,
    /// Hostname or IP address of target
    #[structopt(parse(try_from_str = Host::parse))]
    target: Vec<Host>,
}

impl Options {
    /// Gather all IP addresses dictated through options
    pub fn target_ips(&self) -> Result<Vec<Ipv4Addr>, TracerouteError> {
        // @TODO return an iterator for the different targets?
        let mut hosts = vec![];

        if self.target.is_empty() {
            let err = io::Error::new(io::ErrorKind::Other, "No target given");
            return Err(TracerouteError::Io(err));
        }

        for host in &self.target {
            match host {
                Host::Ipv4(ip) => hosts.push(ip.clone()),
                Host::Ipv6(_) => return Err(TracerouteError::NoIpv6),
                Host::Domain(domain) => {
                    for ip in resolve(&domain)? {
                        match ip {
                            IpAddr::V4(ipv4) => hosts.push(ipv4),
                            IpAddr::V6(_) => continue,
                        }
                    }
                }
            }
        }

        Ok(hosts)
    }

    pub fn get_masked(&self) -> Vec<u8> {
        self.mask.to_owned().unwrap_or(vec![])
    }
}

/// Resolve a domain name to ip addresses
fn resolve(domain: &String) -> Result<ResolveHost, TracerouteError> {
    resolve_host(&domain)
        .map_err(|err| TracerouteError::Io(err))
}
