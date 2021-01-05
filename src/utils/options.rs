use structopt::StructOpt;
use url::Host;
use std::path::PathBuf;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::io;

use resolve::resolve_host;

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
    pub target: Vec<Host>,
}

impl Options {
    /// Gather all IP addresses dictated through options
    pub fn target_ips(&self) -> io::Result<Vec<Ipv4Addr>> {
        // @TODO return an iterator for the different targets?
        let mut hosts = vec![];

        for host in &self.target {
            match host {
                Host::Ipv4(ip) => hosts.push(ip.clone()),
                Host::Ipv6(_) => unimplemented!("Can't handle ipv6 yet :("),
                Host::Domain(domain) => {
                    match resolve(&domain) {
                        Ok(mut ips) => hosts.append(&mut ips),
                        Err(e) => return Err(e),
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
fn resolve(domain: &String) -> io::Result<Vec<Ipv4Addr>> {
    resolve_host(&domain)
        .map(|result| {
            result
                .filter(|i| i.is_ipv4())
                .map(|address| match address {
                    IpAddr::V4(ip) => {
                        eprintln!("Resolved {} to {}", domain, ip);
                        ip
                    }
                    IpAddr::V6(_) => unimplemented!("Can't handle ipv6 yet:("),
                })
                .collect::<Vec<Ipv4Addr>>()
        })
}
