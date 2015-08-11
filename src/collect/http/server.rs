use std::collections::HashMap;
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
    let rest_handler = RestHandler::new(resource_store);
    let serve = move |req: Request, res: Response<Fresh>| {
        rest_handler.handle(req, res);
    };
    info!("Starting HTTP server on [{}]", bind_addr);
    Server::http(bind_addr).map(|s| s.handle(serve).unwrap()).unwrap();
}

struct RestHandler {
    resource_store: Arc<RwLock<ResourceStore>>,
    static_assets: HashMap<String, &'static str>,
}

impl RestHandler {

    fn new(resource_store: Arc<RwLock<ResourceStore>>) -> RestHandler {

        let mut static_assets = HashMap::new();

        static_assets.insert("index.html".to_string(),
                             include_str!("../../../assets/index.html"));
        static_assets.insert("js/arbor.js".to_string(),
                             include_str!("../../../assets/js/arbor.js"));
        static_assets.insert("js/arbor-tween.js".to_string(),
                             include_str!("../../../assets/js/arbor-tween.js"));
        static_assets.insert("js/jquery.min.js".to_string(),
                             include_str!("../../../assets/js/jquery.min.js"));

        return RestHandler {
            resource_store: resource_store,
            static_assets: static_assets,
        }
    }

    fn handle(&self, mut req: Request, mut res: Response<Fresh>) {
        info!("Request from [{:?}]: {:?} {:?}",
                 req.remote_addr,
                 req.method,
                 req.uri);


        let uri = req.uri.clone(); // prevent simultaneous mutable borrow

        match uri {
            AbsolutePath(ref path) =>
                match (&req.method, &path[..]) {
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
                    (&hyper::Get, abs_path)
                          if abs_path == "/" ||
                             abs_path.starts_with("/?") => {
                        self.get_index(res);
                    }
                    (&hyper::Get, abs_path)
                            if abs_path.starts_with("/assets/") => {
                        let asset_name = abs_path.replace("/assets/", "")
                            .replace("..", "");
                        self.get_asset(res, asset_name);
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

    fn get_index(&self, res: Response<Fresh>) {
        self.get_asset(res, "index.html".to_string());
    }

    fn get_asset(&self, mut res: Response<Fresh>, name: String) {
        let content = self.static_assets.get(&name);
        match content {
            Some(content) => {
                let content_type = guess_content_type(&name);
                info!("Serving asset [{}] with content type [{}]",
                      name,
                      content_type);
                res.headers_mut().set_raw("content-type",
                                          vec![content_type.into_bytes()]);

                let mut res = res.start().unwrap();
                res.write_all(content.as_bytes()).unwrap();
                res.end().unwrap();
            },
            None => {
                *res.status_mut() = hyper::NotFound;
            },
        }
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
                            json.set("url", "http://foobar/status".to_string());
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

        responses.insert(
            Resource {
                id: "E".to_string(),
                url: "http://e/status".to_string(),
            },
            None
        );

        let responses_json = ResponsesSerializer
            .serialize(&responses, true)
            .to_string();

        info!("{}", responses_json);

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
