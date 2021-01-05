use traceroute::Traceroute;
use traceroute::Options;
use structopt::StructOpt;

fn main() {
    let options = Options::from_args();
    let output_file = options.output_file.clone();

//    println!("{:#?}", options);

    let mut traceroute = Traceroute::new(options);
    let results = traceroute.run().expect("Traceroute failed");

    match output_file {
        None => results.print(),
        Some(file) => results.write(file),
    }
}

