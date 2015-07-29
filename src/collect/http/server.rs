use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::sync::{Arc, RwLock};

use collect::http::json::{ResourceSerializer,
    ResourcesSerializer,
    ResponsesSerializer};
use collect::resource::{Resource, ResourceStore};
use collect::resource::Response as CollectResponse;

use hyper;
use hyper::header::ContentType;
use hyper::Server;
use hyper::server::{Request, Response};
use hyper::status::StatusCode;
use hyper::net::Fresh;
use hyper::uri::RequestUri::AbsolutePath;
use jsonway;
use jsonway::ObjectSerializer;
use rustc_serialize::json;

pub fn start_server(resource_store: Arc<RwLock<ResourceStore>>,
                    address: String,
                    port: u16) {
    let bind_addr: &str = &format!("{}:{}", address, port);
    let rest_handler = RestHandler {
        resource_store: resource_store,
    };
    let serve = move |req: Request, res: Response<Fresh>| {
        rest_handler.handle(req, res);
    };
    info!("Starting HTTP server on [{}]", bind_addr);
    Server::http(bind_addr).map(|s| s.handle(serve)).unwrap();
}

struct RestHandler {
    resource_store: Arc<RwLock<ResourceStore>>,
}

impl RestHandler {
    fn handle(&self, mut req: Request, mut res: Response<Fresh>) {
        info!("Request from [{:?}]: {:?} {:?}",
                 req.remote_addr,
                 req.method,
                 req.uri);

        let uri = req.uri.clone(); // prevent simultaneous mutable borrow

        match uri {
            AbsolutePath(ref path) =>
                match (&req.method, &path[..]) {
                    (&hyper::Get, "/") => {
                        self.get_index(res);
                    }
                    (&hyper::Get, abs_path)
                            if abs_path.starts_with("/assets/") => {
                        let asset_name = abs_path.replace("/assets/", "");
                        self.get_asset(res, asset_name);
                    }
                    (&hyper::Get, "/resources") => {
                        self.get_resources(res);
                    }
                    (&hyper::Post, "/resources") => {
                        self.post_resources(&mut req, res);
                    }
                    (&hyper::Get, "/responses") => {
                        self.get_responses(res);
                    }
                    (&hyper::Get, "/responses/example") => {
                        self.get_responses_example(res);
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

    fn get_index(&self, mut res: Response<Fresh>) {
        let mut index_file = File::open("assets/index.html").unwrap();
        let mut index_content = String::new();
        index_file.read_to_string(&mut index_content).unwrap();

        res.headers_mut().set(ContentType::html());
        let mut res = res.start().unwrap();
        res.write_all(index_content.as_bytes()).unwrap();
        res.end().unwrap();
    }

    fn get_asset(&self, mut res: Response<Fresh>, name: String) {
        let asset_file = File::open(format!("assets/{}", name));
        if asset_file.is_err() {
            *res.status_mut() = hyper::NotFound;
            return;
        }
        let mut asset_content = String::new();
        asset_file.unwrap().read_to_string(&mut asset_content).unwrap();

        let content_type = guess_content_type(&name);
        info!("Serving asset [{}] with content type [{}]", name, content_type);
        res.headers_mut().set_raw("content-type",
                                  vec![content_type.into_bytes()]);
        let mut res = res.start().unwrap();
        res.write_all(asset_content.as_bytes()).unwrap();
        res.end().unwrap();
    }

    fn get_resources(&self, mut res: Response<Fresh>) {
        // Get the current set of resource targets.
        let resources = self.resource_store.read().unwrap().resources();

        let resources_json = ResourcesSerializer
            .serialize(&resources, true)
            .to_string();

        res.headers_mut().set(ContentType::json());

        let mut res = res.start().unwrap();
        res.write_all(resources_json.as_bytes()).unwrap();
        res.end().unwrap();
    }

    fn post_resources(&self, req: &mut Request, mut res: Response<Fresh>) {
        let mut resource_raw = &mut String::new();
        req.read_to_string(resource_raw).unwrap();
        let decode_result = json::decode(resource_raw);
        if let Err(decode_error) = decode_result {
            warn!("{}", decode_error);
            *res.status_mut() = StatusCode::BadRequest;
            res.headers_mut().set(ContentType::plaintext());
            let mut res = res.start().unwrap();
            res.write_all(format!("{}", decode_error).as_bytes()).unwrap();
            res.end().unwrap();
            return;
        }

        let resource = decode_result.unwrap();
        info!("Adding resource [{:?}]", resource);

        let resource_json = ResourceSerializer
            .serialize(&resource, true)
            .to_string();

        self.resource_store.write().unwrap().save_resource(resource);

        res.headers_mut().set(ContentType::json());
        let mut res = res.start().unwrap();
        res.write_all(resource_json.as_bytes()).unwrap();
        res.end().unwrap();
    }

    fn get_responses(&self, mut res: Response<Fresh>) {
        // Get the current set of cached responses.
        let responses = self.resource_store.read().unwrap().responses();

        let responses_json = ResponsesSerializer
            .serialize(&responses, true)
            .to_string();

        res.headers_mut().set(ContentType::json());

        let mut res = res.start().unwrap();
        res.write_all(responses_json.as_bytes()).unwrap();
        res.end().unwrap();
    }

    fn get_responses_example(&self, mut res: Response<Fresh>) {
        let mut responses = HashMap::new();

        let a_response = jsonway::object(|json| {
            json.object("status", |json| {
                json.array("targets", |json| {
                    json.push(
                        jsonway::object(|json| {
                            json.set("reachable", true);
                            json.set("url", "http://b/status".to_string());
                        })
                    );
                    json.push(
                        jsonway::object(|json| {
                            json.set("reachable", true);
                            json.set("url", "http://c/status".to_string());
                        })
                    );
                    json.push(
                        jsonway::object(|json| {
                            json.set("reachable", true);
                            json.set("url", "http://d/status".to_string());
                        })
                    );
                });
            });
        }).unwrap();

        let b_response = jsonway::object(|json| {
            json.object("status", |json| {
                json.array("targets", |json| {
                    json.push(
                        jsonway::object(|json| {
                            json.set("reachable", true);
                            json.set("url", "http://a/status".to_string());
                        })
                    );
                    json.push(
                        jsonway::object(|json| {
                            json.set("reachable", false);
                            json.set("url", "http://c/status".to_string());
                        })
                    );
                    json.push(
                        jsonway::object(|json| {
                            json.set("reachable", true);
                            json.set("url", "http://d/status".to_string());
                        })
                    );
                });
            });
        }).unwrap();

        let c_response = jsonway::object(|json| {
            json.object("status", |json| {
                json.array("targets", |json| {
                    json.push(
                        jsonway::object(|json| {
                            json.set("reachable", false);
                            json.set("url", "http://a/status".to_string());
                        })
                    );
                    json.push(
                        jsonway::object(|json| {
                            json.set("reachable", false);
                            json.set("url", "http://b/status".to_string());
                        })
                    );
                    json.push(
                        jsonway::object(|json| {
                            json.set("reachable", false);
                            json.set("url", "http://d/status".to_string());
                        })
                    );
                });
            });
        }).unwrap();

        let d_response = jsonway::object(|json| {
            json.object("status", |json| {
                json.array("targets", |json| {
                    json.push(
                        jsonway::object(|json| {
                            json.set("reachable", false);
                            json.set("url", "http://a/status".to_string());
                        })
                    );
                    json.push(
                        jsonway::object(|json| {
                            json.set("reachable", false);
                            json.set("url", "http://b/status".to_string());
                        })
                    );
                    json.push(
                        jsonway::object(|json| {
                            json.set("reachable", false);
                            json.set("url", "http://c/status".to_string());
                        })
                    );
                });
            });
        }).unwrap();

        responses.insert(
            Resource {
                id: "A".to_string(),
                url: "http://a/status".to_string(),
            },
            Some(CollectResponse {
                url: "http://a/status".to_string(),
                status_code: 200,
                json: a_response,
            })
        );

        responses.insert(
            Resource {
                id: "B".to_string(),
                url: "http://b/status".to_string(),
            },
            Some(CollectResponse {
                url: "http://b/status".to_string(),
                status_code: 200,
                json: b_response,
            })
        );

        responses.insert(
            Resource {
                id: "C".to_string(),
                url: "http://c/status".to_string(),
            },
            Some(CollectResponse {
                url: "http://c/status".to_string(),
                status_code: 200,
                json: c_response,
            })
        );

        responses.insert(
            Resource {
                id: "D".to_string(),
                url: "http://d/status".to_string(),
            },
            Some(CollectResponse {
                url: "http://d/status".to_string(),
                status_code: 200,
                json: d_response,
            })
        );

        let responses_json = ResponsesSerializer
            .serialize(&responses, true)
            .to_string();

        res.headers_mut().set(ContentType::json());

        let mut res = res.start().unwrap();
        res.write_all(responses_json.as_bytes()).unwrap();
        res.end().unwrap();
    }
}

fn guess_content_type(name: &String) -> String {
    match name {
        ref r if r.ends_with(".css") => "text/css".to_string(),
        ref r if r.ends_with(".js") => "application/javascript".to_string(),
        ref r if r.ends_with(".html") => "text/html".to_string(),
        _ => "text/plain".to_string(),
    }
}
