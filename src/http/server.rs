extern crate hyper;
extern crate jsonway;

use super::json::StatusSerializer;
use super::super::{Peer, Status};
use std::io::Write;
use self::hyper::Server;
use self::hyper::server::Request;
use self::hyper::server::Response;
use self::hyper::net::Fresh;
use self::jsonway::{ObjectSerializer};

pub fn start_server(address: String, port: u16) {
    let bind_addr: &str = &format!("{}:{}", address, port);
    println!("Starting HTTP server on [{}]", bind_addr);
    Server::http(serve).listen(bind_addr).unwrap();
}

fn serve(_: Request, res: Response<Fresh>) {
    let mut res = res.start().unwrap();
    let status = status();
    let status_json = StatusSerializer.serialize(&status, true).to_string();
    res.write_all(status_json.as_bytes()).unwrap();
    res.end().unwrap();
}

// TODO(CD): Replace this mocked dummy data...
fn status() -> Status {
    Status{
        peers: vec![
            Peer {
                host: "1.2.3.4".to_string(),
                port: 80,
                reachable: true,
            },
            Peer {
                host: "4.3.2.1".to_string(),
                port: 88,
                reachable: false,
            },
        ]
    }
}
