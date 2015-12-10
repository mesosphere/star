extern crate docopt;
extern crate jsonway;
extern crate rustc_serialize;
extern crate star;

use std::sync::{Arc, RwLock};
use std::fs::File;
use std::io::Read;

use star::collect::http::server::{self, TimerSet};
use star::collect::http::json::ResourcesSerializer;
use star::collect::resource::{Resources};
use star::common::{self, logging, MS_PER_SEC};

use docopt::Docopt;
use jsonway::serializer::ObjectSerializer;
use rustc_serialize::json;

static USAGE: &'static str = "
star-collect - Test program for network policies.

This program periodically fetches each configured HTTP resource and
saves state about the responses.  It provides a REST API for
querying the most recent responses data for its target resource set
as well as modifying the set of target resources.

Usage:
    star-collect --help
    star-collect [--http-address=<address> --http-port=<port> --http-request-seconds=<seconds> --resources-file=<path> --logfile=<path>]

Options:
    --help                            Show this help message.
    --http-address=<address>          Address to listen on for HTTP requests
                                      [default: 0.0.0.0].
    --http-port=<port>                Port to listen on for HTTP requests
                                      [default: 9001].
    --http-request-seconds=<seconds>  Seconds between resource fetch attempts
                                      [default: 5].
    --logfile=<path>                  File to log output to instead of stdout.
    --resources-file=<path>           Path to file containing initial resources
                                      as a JSON array.
";

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    logging::init_logger(args.flag_logfile).unwrap();
    common::print_banner();

    // Read initial resources
    let initial_resources: Resources = args.flag_resources_file.map(
        |path| {
            let mut file = File::open(&path).unwrap();
            let mut raw = String::new();
            file.read_to_string(&mut raw).unwrap();
            let decode_result = json::decode(&raw);
            match decode_result {
                Ok(resources) => resources,
                Err(cause) =>
                    panic!("Failed to parse file [{}] as resources!\n{}",
                           path,
                           cause),
            }
        }
    ).unwrap_or(vec!());

    // Create the resource store
    let timer_set = Arc::new(RwLock::new(
        TimerSet::new()));

    // Create the HTTP server
    server::start_server(
        timer_set.clone(),
        args.flag_http_address,
        args.flag_http_port.parse().unwrap()
    );
}

#[derive(Debug, RustcDecodable)]
struct Args {
    flag_help: bool,
    flag_http_address: String,
    flag_http_port: String,
    flag_http_request_seconds: String,
    flag_logfile: Option<String>,
    flag_resources_file: Option<String>,
}
