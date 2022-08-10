use async_std::prelude::StreamExt;
//use async_std::stream::StreamExt;
use async_std::task;
use log::*;
use std::net::Ipv4Addr;
use std::time::Duration;
use std::{collections::HashMap, net::IpAddr};
//use structopt::StructOpt;
use traceroute::{
    ProbeResponse, ProbeSent, TraceActivity, TraceOptions, TraceResponse, TraceType, Traceroute,
    TracerouteError,
};

#[async_std::main]
async fn main() {
    let options = TraceOptions::default();

    stderrlog::new().verbosity(3).init().unwrap();

    let results = task::block_on(app(options));

    match results {
        Ok(_) => (),
        Err(err) => {
            error!("{}", err);
            std::process::exit(1);
        }
    }
}

async fn app(options: TraceOptions) -> Result<(), TracerouteError> {
    let timeout = Duration::from_secs(1);
    let traceroute = Traceroute::new()?;
    let source = traceroute.addresses().first().unwrap().clone();
    let destination = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));
    let trace = traceroute.trace(source, destination, options)?;
    let mut trace = match trace {
        TraceType::V4(trace) => trace,
        TraceType::V6(_) => unimplemented!(),
    };

    let mut probes: HashMap<u16, ProbeSent> = HashMap::new();

    loop {
        let activity = match trace.next().await {
            Some(activity) => activity,
            None => break,
        }?;

        match activity {
            TraceActivity::Masked(_ttl) => unimplemented!(),
            TraceActivity::Sent(probe_sent) => {
                probes.insert(probe_sent.id, probe_sent);
            }
            TraceActivity::Response(response) => {
                if let Some(probe) = probes.remove(response.get_id()) {
                    match response {
                        TraceResponse::Found(response) => {
                            let ping = response.get_instant().duration_since(probe.instant);

                            match response {
                                ProbeResponse::V4 { source, .. } => {
                                    info!("{:>2} {:<15} in {:?}", probe.ttl, source, ping)
                                }
                                ProbeResponse::V6 { source, .. } => {
                                    info!("{} {} in {:?}", probe.ttl, source, ping)
                                }
                            }
                        }
                        TraceResponse::NotReceived(_id) => {
                            info!("ttl {} didn't respond", probe.ttl)
                        }
                    };
                }
            }
        }
        //if first_probe_recieved == true && probes.len() == 0 {
        //    debug!("Found all packets sent! Amazing! Closing shop early.");
        //    break None;
        //}
    }

    Ok(())
    //info!("{}", results);

    //match output_file {
    //    None => io::stdout()
    //        .lock()
    //        .write_all(results.to_dot().as_bytes())
    //        .map_err(TracerouteError::Io),
    //    Some(file) => results.write(file),
    //}?;
}

//enum StreamTypes {
//    Probe,
//    Response,
//    Err,
//}
//
//fn test() {
//    let traceroute = Traceroute::new();
//    let trace = traceroute.trace_host(IpAddr::V4(Ipv4Addr::LOCALHOST));
//    let graph = trace.to_graph();
//
//    let options = Options::new(target);
//    let trace = traceroute.trace_with_options(options);
//
//    while let Some(activity) = trace.next().await {
//        match activity {
//            TraceActivty::Probe(probe) => todo!(),
//            TraceActivty::Response(response) => todo!(),
//        }
//    }
//}
//
//fn to_graph(self) -> Graph {
//    let mut graph = Graph::new();
//    //let probe = Hashmap<Probe,
//
//    while let Some(response) = self.next().await {
//        match response {
//            TraceActivty::Probe(probe) => todo!("store probe"),
//            TraceActivty::Response(response) => {
//                todo!("store response? or match here and store match?");
//            }
//        }
//    }
//
//    todo!("Go through matches(hops) and put them into a graph");
//}
