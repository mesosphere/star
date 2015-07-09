extern crate docopt;
extern crate rustc_serialize;
extern crate star;

use star::probe::common;
use star::collect::http::server;

use docopt::Docopt;

static USAGE: &'static str = "
star-collect - Test program for network policies.

This program periodically fetches each configured HTTP resource and
saves state about the responses.  It provides a REST API for
querying the most recent responses data for its target resource set
as well as modifying the set of target resources.

Usage:
    star-collect --help
    star-collect [--http-address=<address> --http-port=<port> --http-request-seconds=<seconds>]

Options:
    --help                            Show this help message.
    --http-address=<address>          Address to listen on for HTTP requests
                                      [default: 0.0.0.0].
    --http-port=<port>                Port to listen on for HTTP requests
                                      [default: 9000].
    --http-request-seconds=<seconds>  Seconds between resource fetch attempts
                                      [default: 5].
";

fn main() {
    common::print_banner();

    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    println!("{:?}", &args);

    // Create the responses cache

    // Create the resource client driver

    // Create the HTTP server
    server::start_server(
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
}
