extern crate docopt;
extern crate rustc_serialize;
extern crate star;

use std::sync::{Arc, RwLock};

use star::common;
use star::common::MS_PER_SEC;
use star::common::logging;
use star::collect::http::server;
use star::collect::resource::{client, ResourceStore};

use docopt::Docopt;

static USAGE: &'static str = "
star-collect - Test program for network policies.

This program periodically fetches each configured HTTP resource and
saves state about the responses.  It provides a REST API for
querying the most recent responses data for its target resource set
as well as modifying the set of target resources.

Usage:
    star-collect --help
    star-collect [--http-address=<address> --http-port=<port> --http-request-seconds=<seconds> --logfile=<file>]

Options:
    --help                            Show this help message.
    --http-address=<address>          Address to listen on for HTTP requests
                                      [default: 0.0.0.0].
    --http-port=<port>                Port to listen on for HTTP requests
                                      [default: 9001].
    --http-request-seconds=<seconds>  Seconds between resource fetch attempts
                                      [default: 5].
    --logfile=<path>                File to log output to instead of stdout.
";

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    logging::init_logger(args.flag_logfile).unwrap();
    common::print_banner();

    // Create the resource store
    let resource_store = Arc::new(RwLock::new(ResourceStore::new(vec!())));

    // Create the resource client driver
    let http_req_ms =
        args.flag_http_request_seconds.parse::<u32>().unwrap() * MS_PER_SEC;

    client::start_client_driver(http_req_ms as u64, resource_store.clone());

    // Create the HTTP server
    server::start_server(
        resource_store.clone(),
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
}
