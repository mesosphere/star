use probe::status::{Status, Target};

use jsonway::{ObjectBuilder, ObjectSerializer};

pub struct StatusSerializer;

impl ObjectSerializer<Status> for StatusSerializer {
    fn root(&self) -> Option<&str> { Some("status") }
    fn build(&self, status: &Status, json: &mut ObjectBuilder) {

        let target_json = status.targets.iter().map(|s|
            TargetSerializer.serialize(s, false)).collect::<Vec<_>>();

        json.set("targets", target_json);
    }
}

pub struct TargetSerializer;

impl ObjectSerializer<Target> for TargetSerializer {
    fn root(&self) -> Option<&str> { Some("target") }
    fn build(&self, target: &Target, json: &mut ObjectBuilder) {
        json.set("url", target.url.clone());
        json.set("reachable", target.reachable.clone());
    }
}
