extern crate mio;

use std::thread;
use self::mio::{EventLoop, Handler};

pub fn start_probe_driver(peers: Vec<String>,
                          http_probe_ms: u64) {
    println!("Starting probe driver");
    let mut event_loop = EventLoop::new().unwrap();
    let _ = event_loop.timeout_ms((), http_probe_ms);
    thread::spawn(move || {
        let _ = event_loop.run(&mut ProbeHandler {
            peers: peers,
            http_probe_ms: http_probe_ms,
        });
    });
}

struct ProbeHandler {
    peers: Vec<String>,
    http_probe_ms: u64,
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
        println!("probing peer: [{}]", peer_url);
    }
}
