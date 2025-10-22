mod edge;
mod node;
mod options;
mod trace;
pub mod prelude;
mod protocol;
mod probe;
mod packet;
mod traceroute;
mod sockets;
mod utils;

use std::fs::File;
use std::io::prelude::*;
pub use prelude::*;
use crate::edge::Edge;
use crate::node::Node;
use async_std::task;
use log::*;
pub use options::Options;
use std::io;
use std::net::IpAddr;
use structopt::StructOpt;

fn main() -> Result<(), io::Error> {
    let options = Options::from_args();

    stderrlog::new()
        .verbosity(options.verbose)
        .quiet(options.quiet)
        .init()
        .unwrap();

    let results = task::block_on(app(options));

    match results {
        Ok(_) => (),
        Err(err) => {
            error!("{}", err);
            std::process::exit(1);
        }
    }
    Ok(())
}

async fn app(options: Options) -> Result<(), TracerouteError> {
    let targets = options.target_ips()?;

    let Options {
        min_ttl,
        max_ttl,
        delay,
        mask,
        protocol,
        dot,
        output_file,
        ..
    } = options;

    // Lock to ensure traceroute isn't running at the same time as another
    let agent = Traceroute::new(delay)?;


    let mut config = TraceOptions {
        min_ttl,
        max_ttl,
        delay,
        mask: Default::default(),
        timeout: 300,
        protocol,
        dot,
    };

    // @TODO maybe a default source?
    let source = agent.addresses().first().unwrap().clone();

    // Fill in mask from options
    if let Some(mask) = mask {
        for ttl in mask {
            config.mask(ttl.clone());
        }
    }

    let mut traces = vec![];
    for target in targets {
        match target {
            IpAddr::V4(_) => traces.push(agent.trace(source.clone(), target, config.clone())?),
            IpAddr::V6(ip) => warn!("Skipped IPv6 target {}", ip),
        }

        // break after the first trace if we are not building a graph
        if !dot {
            break;
        }
    }


    let mut data = TraceData::new(config.clone());
    let _ = data.process(traces).await;

    match output_file {
        None => io::stdout()
            .lock()
            .write_all(format!("{}",data).as_bytes())
            .map_err(TracerouteError::Io),
        Some(file) => {
            let mut handle =File::create(file)?;
            let result = handle.write_all(format!("{}", data).as_bytes())?;
            Ok(result)
        },
    }?;

    Ok(())
}
