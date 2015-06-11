extern crate hyper;
extern crate mio;
extern crate threadpool;

use std::sync::{Arc, Mutex};
use std::thread;

use status::StatusCache;

use self::hyper::client::Response;
use self::hyper::Client;
use self::hyper::error::Error;
use self::hyper::header::Connection;
use self::mio::{EventLoop, Handler};
use self::threadpool::ThreadPool;

pub fn start_probe_driver(peers: Vec<String>,
                          http_probe_ms: u64,
                          status_cache: Arc<Mutex<StatusCache>>) {
    println!("Starting probe driver");
    let mut event_loop = EventLoop::new().unwrap();
    let _ = event_loop.timeout_ms((), http_probe_ms);
    thread::spawn(move || {
        let _ = event_loop.run(&mut ProbeHandler {
            peers: peers,
            http_probe_ms: http_probe_ms,
            status_cache: status_cache,
            thread_pool: ThreadPool::new(4),
        });
    });
}

struct ProbeHandler {
    peers: Vec<String>,
    http_probe_ms: u64,
    status_cache: Arc<Mutex<StatusCache>>,
    thread_pool: ThreadPool,
}

impl Handler for ProbeHandler {
    type Timeout = ();
    type Message = String;

    fn timeout(&mut self,
               event_loop: &mut EventLoop<ProbeHandler>,
               _: ()) {
        println!("Probing all peers");
        let loop_channel = event_loop.channel();
        for peer in self.peers.clone() {
            let _ = loop_channel.send(peer);
        }
        let _ = event_loop.timeout_ms((), self.http_probe_ms);
    }

    fn notify(&mut self,
              _: &mut EventLoop<ProbeHandler>,
              peer_url: String) {
        let status_cache = self.status_cache.clone();
        self.thread_pool.execute(move || {
            println!("Probing peer: [{}]", peer_url);

            let mut client = Client::new();

            let response: Result<Response, Error> =
                client.get(&peer_url)
                    .header(Connection::close())
                    .send();

            match response {
                Ok(_) => {
                    let mut status_cache = status_cache.lock().unwrap();
                    status_cache.reachable(peer_url);
                }
                Err(_) => {
                    let mut status_cache = status_cache.lock().unwrap();
                    status_cache.unreachable(peer_url);
                }
            }
        });
    }
}
