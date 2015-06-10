extern crate jsonway;

use status::{Peer, Status};

use self::jsonway::{ObjectBuilder, ObjectSerializer};

pub struct StatusSerializer;

impl ObjectSerializer<Status> for StatusSerializer {
    fn root(&self) -> Option<&str> { Some("status") }
    fn build(&self, status: &Status, json: &mut ObjectBuilder) {

        let peer_json = status.peers.iter().map(|s|
            PeerSerializer.serialize(s, false)).collect::<Vec<_>>();

        json.set("peers", peer_json);
    }
}

pub struct PeerSerializer;

impl ObjectSerializer<Peer> for PeerSerializer {
    fn root(&self) -> Option<&str> { Some("peer") }
    fn build(&self, peer: &Peer, json: &mut ObjectBuilder) {
        json.set("url", peer.url.clone());
        json.set("reachable", peer.reachable.clone());
    }
}
