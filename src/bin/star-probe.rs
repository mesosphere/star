extern crate docopt;
#[macro_use]
extern crate log;
extern crate rustc_serialize;
extern crate star;

use std::sync::{Arc, RwLock};

use star::common;
use star::common::MS_PER_SEC;
use star::common::logging;
use star::probe::http::server;
use star::probe::status::{client, StatusCache};

use docopt::Docopt;

static USAGE: &'static str = "
star-probe - Test program for network policies.

This program periodically attempts to connect to each configured target URL and
saves state about which ones are reachable.  It provides a REST API for
querying the most recent reachability data for its target set.

Usage:
    star-probe --help
    star-probe --urls=<urls> [--http-address=<address> --http-port=<port> --http-probe-seconds=<seconds> --logfile=<path>]

Options:
    --help                          Show this help message.
    --http-address=<address>        Address to listen on for HTTP requests
                                    [default: 0.0.0.0].
    --http-port=<port>              Port to listen on for HTTP requests
                                    [default: 9000].
    --http-probe-seconds=<seconds>  Seconds between probe connection attempts
                                    [default: 5].
    --urls=<urls>                   List of comma-delimited URLs to probe, e.g:
                                    foo.baz.com:80,bar.baz.com:80
    --logfile=<path>                File to log output to instead of stdout.
";

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    logging::init_logger(args.flag_logfile).unwrap();
    common::print_banner();

    let target_urls: Vec<String> = args.flag_urls
        .split(",")
        .map(|s| s.to_string())
        .filter(|s| s != "")
        .collect();

    info!("Target URLs: {:?}", &target_urls);

    // Create the status cache
    let status_cache = Arc::new(RwLock::new(StatusCache::new(&target_urls)));

    // Create the peer probe client driver
    let http_probe_ms =
        args.flag_http_probe_seconds.parse::<u32>().unwrap() * MS_PER_SEC;

    client::start_client_driver(target_urls,
                              http_probe_ms as u64,
                              status_cache.clone());

    // Create the HTTP server
    server::start_server(
        status_cache.clone(),
        args.flag_http_address,
        args.flag_http_port.parse().unwrap()
    );
}

#[derive(Debug, RustcDecodable)]
struct Args {
    flag_help: bool,
    flag_http_address: String,
    flag_http_port: String,
    flag_http_probe_seconds: String,
    flag_urls: String,
    flag_logfile: Option<String>,
}
