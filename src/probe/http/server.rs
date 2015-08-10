use std::io::Write;
use std::sync::{Arc, RwLock};

use probe::status::StatusCache;
use probe::http::json::StatusSerializer;

use hyper;
use hyper::header::ContentType;
use hyper::Server;
use hyper::server::Request;
use hyper::server::Response;
use hyper::net::Fresh;
use hyper::uri::RequestUri::AbsolutePath;
use jsonway::{ObjectSerializer};

pub fn start_server(status_cache: Arc<RwLock<StatusCache>>,
                    address: String,
                    port: u16) {
    let bind_addr: &str = &format!("{}:{}", address, port);
    let status_handler = StatusHandler { status_cache: status_cache, };
    let serve = move |req: Request, res: Response<Fresh>| {
        status_handler.handle(req, res);
    };
    info!("Starting HTTP server on [{}]", bind_addr);
    Server::http(bind_addr).map(|s| s.handle(serve).unwrap()).unwrap();
}

struct StatusHandler {
    status_cache: Arc<RwLock<StatusCache>>,
}

impl StatusHandler {
    fn handle(&self, req: Request, mut res: Response<Fresh>) {
        info!("Request from [{:?}]: {:?} {:?}",
              req.remote_addr,
              req.method,
              req.uri);

        match req.uri {
            AbsolutePath(ref path) =>
                match (&req.method, &path[..]) {
                    (&hyper::Get, "/status") => {
                        // Get the current status from the cache.
                        let status = &self.status_cache.read().unwrap().poll();
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
