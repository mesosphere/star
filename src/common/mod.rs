pub struct Status {
    pub peers: Vec<Peer>,
}

pub struct Peer {
    pub host: String,
    pub port: u16,
    pub reachable: bool,
}
