use log::*;
use std::io::{self, Write};
use structopt::StructOpt;
use traceroute::{Options, Traceroute, TracerouteError};

fn main() {
    let options = Options::from_args();
    let output_file = options.output_file.clone();

    stderrlog::new()
        .verbosity(options.verbose)
        .quiet(options.quiet)
        .init()
        .unwrap();

    let mut traceroute = Traceroute::new(options);
    let results = match traceroute.run() {
        Ok(graph) => graph,
        Err(err) => return handle_error(err),
    };

    let output = match output_file {
        None => io::stdout()
            .lock()
            .write_all(results.to_string().as_bytes())
            .map_err(TracerouteError::Io),
        Some(file) => results.write(file),
    };

    if let Err(error) = output {
        handle_error(error);
    }
}

fn handle_error(error: TracerouteError) {
    error!("{}", error);
    std::process::exit(1);
}
