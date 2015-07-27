use std::io::Read;
use std::sync::{Arc, RwLock};
use std::thread;

use collect::resource::{Resource, ResourceStore, Response};

use hyper::client::Response as HttpResponse;
use hyper::Client;
use hyper::error::Error;
use hyper::header::Connection;
use hyper::http::RawStatus;
use mio::{EventLoop, Handler};
use rustc_serialize::json::Json;
use threadpool::ThreadPool;

pub fn start_client_driver(http_request_ms: u64,
                           resource_store: Arc<RwLock<ResourceStore>>) {
    info!("Starting client driver");
    let mut event_loop = EventLoop::new().unwrap();
    let _ = event_loop.timeout_ms((), http_request_ms);
    thread::spawn(move || {
        let _ = event_loop.run(&mut ClientHandler {
            http_request_ms: http_request_ms,
            resource_store: resource_store,
            thread_pool: ThreadPool::new(4),
        });
    });
}

struct ClientHandler {
    http_request_ms: u64,
    resource_store: Arc<RwLock<ResourceStore>>,
    thread_pool: ThreadPool,
}

impl Handler for ClientHandler {
    type Timeout = ();
    type Message = Resource;

    fn timeout(&mut self,
               event_loop: &mut EventLoop<ClientHandler>,
               _: ()) {
        info!("Fetching all resources");
        let loop_channel = event_loop.channel();
        for resource in self.resource_store.read().unwrap().resources() {
            let _ = loop_channel.send(resource);
        }
        let _ = event_loop.timeout_ms((), self.http_request_ms);
    }

    fn notify(&mut self,
              _: &mut EventLoop<ClientHandler>,
              resource: Resource) {
        let resource_store = self.resource_store.clone();
        self.thread_pool.execute(move || {
            info!("Fetching resource: [{}]", &resource.url);

            let client = Client::new();

            let response_result: Result<HttpResponse, Error> =
                client.get(&resource.url)
                    .header(Connection::close())
                    .send();

            // Obtain an exclusive write lock to the status cache.
            let mut resource_store = resource_store.write().unwrap();

            match response_result {
                Ok(mut http_response) => {
                    let body = &mut String::new();
                    http_response.read_to_string(body).unwrap();
                    let body_json = Json::from_str(body);

                    if let Err(parse_error) = body_json {
                        let error_str = format!("{}", parse_error);
                        warn!("Failed to parse response body as JSON: [{}]",
                            error_str);
                        resource_store.save_response(resource, None);
                        return;
                    }

                    let &RawStatus(status_code, _) =
                        http_response.status_raw();

                    let response = Response {
                        url: resource.url.clone(),
                        status_code: status_code,
                        json: body_json.unwrap(),
                    };
                    resource_store.save_response(resource, Some(response));
                },
                Err(_) => resource_store.save_response(resource, None),
            }
        });
    }
}

