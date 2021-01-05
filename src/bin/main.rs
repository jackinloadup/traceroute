use traceroute::Traceroute;
use traceroute::Options;
use structopt::StructOpt;
use std::io::{self, Write};

fn main() {
    let options = Options::from_args();
    let output_file = options.output_file.clone();

    let mut traceroute = Traceroute::new(options);
    let results = traceroute.run().expect("Traceroute failed");

    match output_file {
        None => {
            io::stdout()
                .lock()
                .write_all(results.to_string().as_bytes()).unwrap();
        },
        Some(file) => results.write(file),
    }
}

