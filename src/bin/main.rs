use traceroute::Traceroute;
use traceroute::Options;
use structopt::StructOpt;
use std::io::{self, Write};

fn main() {
    let options = Options::from_args();
    let output_file = options.output_file.clone();

//    println!("{:#?}", options);

    let mut traceroute = Traceroute::new(options);
    let results = traceroute.run().expect("Traceroute failed");

    match output_file {
        None => {
            let stdout = io::stdout();
            stdout.lock()
                .write_all(results.as_string().as_bytes()).unwrap();
        },
        Some(file) => results.write(file),
    }
}

