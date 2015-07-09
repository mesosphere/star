use std::collections::HashMap;

pub mod client;

pub struct Status {
    pub targets: Vec<Target>,
}

pub struct Target {
    pub url: String,
    pub reachable: bool,
}

pub struct StatusCache {
    state: HashMap<String, bool>,
}

impl StatusCache {
    pub fn new(target_urls: &Vec<String>) -> StatusCache {
        let mut initial_state = HashMap::new();
        for target in target_urls {
            initial_state.insert(target.clone(), false);
        }
        StatusCache { state: initial_state, }
    }

    pub fn poll(&self) -> Status {
        let targets = self.state.iter().map(|(url, reachable)|
                        Target {
                            url: url.clone(),
                            reachable: reachable.clone(),
                        }
                    ).collect();
        Status { targets: targets, }
    }

    fn update(&mut self, target_url: String, reachable: bool) {
        if !self.state.contains_key(&target_url) {
            warn!("Received update state for unknown target [{}]",
                     target_url);
            return;
        }
        self.state.insert(target_url, reachable);
    }

    pub fn reachable(&mut self, target_url: String) {
        info!("Target [{}] is now reachable.", target_url);
        self.update(target_url, true);
    }

    pub fn unreachable(&mut self, target_url: String) {
        info!("Target [{}] is now unreachable.", target_url);
        self.update(target_url, false);
    }
}
