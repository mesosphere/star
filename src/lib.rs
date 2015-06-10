pub mod http;

pub struct Status {
    peers: Vec<Peer>,
}

pub struct Peer {
    host: String,
    port: u16,
    reachable: bool,
}
