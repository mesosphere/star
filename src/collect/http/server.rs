use std::io::Write;

use collect::http::json::{ResourcesSerializer};
use collect::resource::{Resource, Resources};

use hyper;
use hyper::header::ContentType;
use hyper::Server;
use hyper::server::Request;
use hyper::server::Response;
use hyper::net::Fresh;
use hyper::uri::RequestUri::AbsolutePath;
use jsonway::{ObjectSerializer};

pub fn start_server(address: String,
                    port: u16) {
    let bind_addr: &str = &format!("{}:{}", address, port);
    let rest_handler = RestHandler;
    let serve = move |req: Request, res: Response<Fresh>| {
        rest_handler.handle(req, res);
    };
    println!("Starting HTTP server on [{}]", bind_addr);
    Server::http(serve).listen(bind_addr).unwrap();
}

struct RestHandler;

impl RestHandler {
    fn handle(&self, req: Request, mut res: Response<Fresh>) {
        println!("Request from [{:?}]: {:?} {:?}",
                 req.remote_addr,
                 req.method,
                 req.uri);

        match req.uri {
            AbsolutePath(ref path) =>
                match (&req.method, &path[..]) {
                    (&hyper::Get, "/resources") => {
                        self.get_resources(res);
                    }
                    (&hyper::Post, "/resources") => {
                        self.post_resources(&req, res);
                    }
                    (&hyper::Get, "/responses") => {
                        self.get_responses(res);
                    }
                    _ => {
                        // Anything else is invalid.
                        *res.status_mut() = hyper::NotFound;
                        return;
                    }
                },
                _ => { return; }
        };
    }

    fn get_resources(&self, mut res: Response<Fresh>) {
        // Get the current set of resource targets.

        // TODO(CD): Get real list of resources.
        let resources: Resources = vec![
            Resource {
                id: "A".to_string(),
                url: "http://a/status".to_string(),
            },
            Resource {
                id: "B".to_string(),
                url: "http://b/status".to_string(),
            },
        ];

        let resources_json = ResourcesSerializer
            .serialize(&resources, true)
            .to_string();

        res.headers_mut().set(ContentType::json());

        let mut res = res.start().unwrap();
        res.write_all(resources_json.as_bytes()).unwrap();
        res.end().unwrap();
    }

    fn post_resources(&self, req: &Request, mut res: Response<Fresh>) {
        res.headers_mut().set(ContentType::json());
        let mut res = res.start().unwrap();
        res.write_all("NOT IMPLEMENTED".as_bytes()).unwrap();
        res.end().unwrap();
    }

    fn get_responses(&self, mut res: Response<Fresh>) {
        res.headers_mut().set(ContentType::json());
        let mut res = res.start().unwrap();
        res.write_all("NOT IMPLEMENTED".as_bytes()).unwrap();
        res.end().unwrap();
    }
}
