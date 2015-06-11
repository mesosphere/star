extern crate hyper;
extern crate jsonway;

use std::io::Write;
use std::sync::{Arc, Mutex};

use status::StatusCache;
use http::json::StatusSerializer;

use self::hyper::header::ContentType;
use self::hyper::Server;
use self::hyper::server::Request;
use self::hyper::server::Response;
use self::hyper::net::Fresh;
use self::hyper::uri::RequestUri::AbsolutePath;
use self::jsonway::{ObjectSerializer};

pub fn start_server(status_cache: Arc<Mutex<StatusCache>>,
                    address: String,
                    port: u16) {
    let bind_addr: &str = &format!("{}:{}", address, port);
    let status_handler = StatusHandler { status_cache: status_cache, };
    let serve = move |req: Request, res: Response<Fresh>| {
        status_handler.handle(req, res);
    };
    println!("Starting HTTP server on [{}]", bind_addr);
    Server::http(serve).listen(bind_addr).unwrap();
}

struct StatusHandler {
    status_cache: Arc<Mutex<StatusCache>>,
}

impl StatusHandler {
    fn handle(&self, req: Request, mut res: Response<Fresh>) {
        println!("Request from [{:?}]: {:?} {:?}",
                 req.remote_addr,
                 req.method,
                 req.uri);

        match req.uri {
            AbsolutePath(ref path) =>
                match (&req.method, &path[..]) {
                    (&hyper::Get, "/status") => {
                        // Get the current status from the cache.
                        let status = &self.status_cache.lock().unwrap().poll();
                        let status_json = StatusSerializer
                            .serialize(&status, true)
                            .to_string();

                        res.headers_mut().set(ContentType::json());

                        let mut res = res.start().unwrap();
                        res.write_all(status_json.as_bytes()).unwrap();
                        res.end().unwrap();
                    }
                    _ => {
                        // Anything but GET /status is invalid.
                        *res.status_mut() = hyper::NotFound;
                        return;
                    }
                },
                _ => {
                    return;
                }
        };
    }
}
