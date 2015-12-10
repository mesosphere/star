use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::{Arc, RwLock};

use collect::http::json::{ResourcesSerializer, ResponsesSerializer};
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
use time;
use rand;
use rand::distributions::{IndependentSample, Range};

pub struct TimerSet {
    hm: HashMap<String, HashMap<String, time::Timespec>>,
}

impl TimerSet {
    pub fn new() -> TimerSet {
        TimerSet {
            hm: HashMap::new()
        }
    }

    pub fn push(&mut self, src: String, dst: String, time: time::Timespec) {
        // check if outer map is present, add it otherwise
        if !self.hm.contains_key(&src) {
            self.hm.insert(src.clone(), HashMap::new());
        }
        self.hm.get_mut(&src).map(|hm| hm.insert(dst, time));
    }

    pub fn resources(&self) -> HashMap<Resource, Option<CollectResponse>> {
        let now = time::now().to_timespec().sec;
        let mut responses = HashMap::new();

        for (src, dst_map) in self.hm.iter() {
            let response = jsonway::object(|json| {
                json.object("status", |json| {
                    json.array("targets", |json| {
                        for (dst, heard_from) in dst_map.iter() {
                            json.push(
                                jsonway::object(|json| {
                                    json.set("reachable", now - heard_from.sec);
                                    json.set("url", dst.clone());
                                })
                            );
                        }
                    });
                });
            }).unwrap();

            responses.insert(
                Resource {
                    id: src.clone(),
                    src: src.clone(),
                    dst: "".to_string(),
                    time: 0,
                    count: 0,
                },
                Some(CollectResponse {
                    url: src.clone(),
                    status_code: 200,
                    json: response,
                })
            );
        }

        responses
    }
}

pub fn start_server(timer_set: Arc<RwLock<TimerSet>>,
                    address: String,
                    port: u16) {
    let bind_addr: &str = &format!("{}:{}", address, port);
    let rest_handler = RestHandler::new(timer_set);
    let serve = move |req: Request, res: Response<Fresh>| {
        rest_handler.handle(req, res);
    };
    info!("Starting HTTP server on [{}]", bind_addr);
    Server::http(bind_addr).map(|s| s.handle(serve).unwrap()).unwrap();
}

struct RestHandler {
    timer_set: Arc<RwLock<TimerSet>>,
    static_assets: HashMap<String, &'static str>,
}

impl RestHandler {

    fn new(timer_set: Arc<RwLock<TimerSet>>) -> RestHandler {

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
            timer_set: timer_set,
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
                    (&hyper::Post, "/hits") => {
                        self.post_hits(&mut req, res);
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

    fn post_hits(&self, req: &mut Request, mut res: Response<Fresh>) {
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

        let resources = decode_result.unwrap();
        info!("Adding resources {:?}", resources);

        let resource_json = ResourcesSerializer
            .serialize(&resources, true)
            .to_string();

        for res in resources {
            self.timer_set.write().unwrap().push(res.src, res.dst, time::Timespec{sec:res.time, nsec:0});
        }

        res.headers_mut().set(ContentType::json());
        let mut res = res.start().unwrap();
        res.write_all(resource_json.as_bytes()).unwrap();
        res.end().unwrap();
    }

    fn get_responses(&self, mut res: Response<Fresh>) {
        // Get the current set of cached responses.
        let responses = self.timer_set.read().unwrap().resources();

        let responses_json = ResponsesSerializer
            .serialize(&responses, true)
            .to_string();

        res.headers_mut().set(ContentType::json());

        let mut res = res.start().unwrap();
        res.write_all(responses_json.as_bytes()).unwrap();
        res.end().unwrap();
    }

    fn get_responses_example(&self, mut res: Response<Fresh>) {
        let now = time::now().to_timespec();
        let mut ts = TimerSet{ hm: HashMap::new() };
        let selections = vec![
            "A".to_string(),
            "B".to_string(),
            "C".to_string(),
            "D".to_string(),
            "E".to_string(),
        ];

        let mut rng = rand::thread_rng();
        let between = Range::new(0, 5);
        for (i, selection) in selections.iter().enumerate() {
            for j in (0..selections.len()).filter(|j| *j != i) {
                ts.push(
                    selection.clone(),
                    selections[j].clone(),
                    time::Timespec{
                        sec:now.sec - between.ind_sample(&mut rng),
                        nsec: 0,
                    }
                );
            }
        }

        let responses_json = ResponsesSerializer
            .serialize(&ts.resources(), true)
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
