extern crate docopt;
extern crate rustc_serialize;
extern crate star;

use star::http::server;

use docopt::Docopt;

static USAGE: &'static str = "
star - test program for network policies

Usage:
    star --help
    star [--http-address=<address>] [--http-port=<port>]
";

fn main() {
    print_banner();

    let default_args = Args::default();

    let mut args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    if args.flag_http_address == "" {
        args.flag_http_address = default_args.flag_http_address;
    }
    if args.flag_http_port == "" {
        args.flag_http_port = default_args.flag_http_port;
    }

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
}

impl Default for Args {
    fn default() -> Args {
        Args {
            flag_help: false,
            flag_http_address: "0.0.0.0".to_string(),
            flag_http_port: "9000".to_string(),
        }
    }
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
