extern crate docopt;
extern crate rustc_serialize;
extern crate star;

use star::http::server;
use star::status::{probe, StatusCache};

use docopt::Docopt;

static MS_PER_SEC: &'static u32 = &1000;

static USAGE: &'static str = "
star-probe - Test program for network policies.

This program periodically attempts to connect to each configured peer URL and
saves state about which ones are reachable.  It provides a REST API for
querying the most recent reachability data for its peer set.

Usage:
    star-probe --help
    star-probe [--http-address=<address>]
         [--http-port=<port>]
         [--http-probe-seconds=<seconds>]
         --peers=<peers>

Options:
    --help                          Show this help message.
    --http-address=<address>        Address to listen on for HTTP requests
                                    [default: 0.0.0.0].
    --http-port=<port>              Port to listen on for HTTP requests
                                    [default: 9000].
    --http-probe-seconds=<seconds>  Seconds between peer connection attempts
                                    [default: 5].
    --peers=<peers>                 List of comma-delimited peer URLs, e.g:
                                    foo.baz.com:80,bar.baz.com:80
";

fn main() {
    print_banner();

    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    let peer_urls: Vec<String> = args.flag_peers
        .split(",")
        .map(|s| s.to_string())
        .filter(|s| s != "")
        .collect();

    println!("Peers: {:?}", &peer_urls);

    // Create the status cache
    let status_cache = StatusCache::new(&peer_urls);

    // Create the peer probe driver
    let http_probe_ms =
        args.flag_http_probe_seconds.parse::<u32>().unwrap() * MS_PER_SEC;
    probe::start_probe_driver(peer_urls,
                              http_probe_ms as u64);

    // Create the HTTP server
    server::start_server(
        status_cache,
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
    flag_peers: String,
}

fn print_banner() {
    println!("
   _____ _____ ___  ______
  /  ___|_   _/ _ \\ | ___ \\
  \\ `--.  | |/ /_\\ \\| |_/ /
   `--. \\ | ||  _  ||    /
  /\\__/ / | || | | || |\\ \\
  \\____/  \\_/\\_| |_/\\_| \\_|
    ");
}
