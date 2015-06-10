use std::collections::hash_map::HashMap;

pub struct Status {
    pub peers: Vec<Peer>,
}

pub struct Peer {
    pub url: String,
    pub reachable: bool,
}

pub struct StatusCache {
    state: HashMap<String, bool>,
}

impl StatusCache {
    pub fn new(peer_urls: Vec<String>) -> StatusCache {
        let mut initial_state = HashMap::new();
        for peer in peer_urls { initial_state.insert(peer, false); }
        StatusCache { state: initial_state, }
    }

    pub fn poll(&self) -> Status {
        let peers = self.state.iter().map(|(url, reachable)|
                        Peer {
                            url: url.clone(),
                            reachable: reachable.clone(),
                        }
                    ).collect();
        Status { peers: peers, }
    }

    fn update(&mut self, peer_url: String, reachable: bool) {
        if !self.state.contains_key(&peer_url) {
            println!("Warning: received update state for unknown peer [{}]",
                     peer_url);
            return;
        }
        self.state.insert(peer_url, reachable);
    }

    pub fn reachable(&mut self, peer_url: String) {
        println!("Peer [{}] is now reachable.", peer_url);
        self.update(peer_url, true);
    }

    pub fn unreachable(&mut self, peer_url: String) {
        println!("Peer [{}] is now unreachable.", peer_url);
        self.update(peer_url, false);
    }
}
