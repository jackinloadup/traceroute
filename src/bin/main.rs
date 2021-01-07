use std::io::{self, ErrorKind, Write};
use structopt::StructOpt;
use traceroute::{Options, Traceroute, TracerouteError};

fn main() {
    let options = Options::from_args();
    let output_file = options.output_file.clone();

    let mut traceroute = Traceroute::new(options);
    let results = match traceroute.run() {
        Ok(graph) => graph,
        Err(err) => return handle_error(err),
    };

    let output = match output_file {
        None => io::stdout()
            .lock()
            .write_all(results.to_string().as_bytes())
            .map_err(|err| TracerouteError::Io(err)),
        Some(file) => results.write(file),
    };

    if let Err(error) = output {
        handle_error(error);
    }
}

fn handle_error(error: TracerouteError) {
    match error {
        TracerouteError::Io(err) => match err.kind() {
            ErrorKind::PermissionDenied => eprintln!(
                "Couldn't open network: {:?}. Try again with sudo",
                err.kind()
            ),
            ErrorKind::Other => match err.into_inner() {
                Some(err) => eprintln!("Failed: {}", err),
                None => eprintln!("Failed with unhandled error"),
            },
            err_kind => eprintln!("Failed with unhandled error: {:?}", err_kind),
        },
        TracerouteError::Impossible(err) | TracerouteError::UnmatchedPacket(err) => {
            eprintln!("Failed: {:?}", err)
        }
        TracerouteError::ICMPTypeUnexpected(_)
        | TracerouteError::PacketDecode
        | TracerouteError::MalformedPacket => eprintln!("Failed decoding received packets"),
        TracerouteError::NoIpv6 => eprintln!("No support for ipv6 yet"),
        TracerouteError::UnimplimentedProtocol(proto) => {
            eprintln!("No support for {:?} probes yet", proto)
        }
    };
    std::process::exit(1);
}
