use collect::resource::{Resource, Resources, Response, Responses};

use jsonway::{ArrayBuilder, ObjectBuilder, ObjectSerializer};
use rustc_serialize::json::Json;

pub struct ResourceSerializer;

impl ObjectSerializer<Resource> for ResourceSerializer {
    fn root(&self) -> Option<&str> { None }
    fn build(&self, resource: &Resource, json: &mut ObjectBuilder) {
        json.set("id", resource.id.clone());
        json.set("url", resource.url.clone());
    }
}

pub struct ResourcesSerializer;

impl ObjectSerializer<Resources> for ResourcesSerializer {
    fn root(&self) -> Option<&str> { None }
    fn build(&self, resources: &Resources, json: &mut ObjectBuilder) {
        let mut resources_json = ArrayBuilder::new();
        for item in resources {
            let item_json = ResourceSerializer.serialize(item, false);
            resources_json.push(item_json);
        }
        json.set("resources", resources_json);
    }
}

pub struct ResponseSerializer;

impl ObjectSerializer<Response> for ResponseSerializer {
    fn root(&self) -> Option<&str> { None }
    fn build(&self, response: &Response, json: &mut ObjectBuilder) {
        json.set("url", response.url.clone());
        json.set("statusCode", response.status_code.clone());
        json.set("json", response.json.clone());
    }
}

pub struct ResponsesSerializer;

impl ObjectSerializer<Responses> for ResponsesSerializer {
    fn root(&self) -> Option<&str> { Some("responses") }
    fn build(&self, responses: &Responses, json: &mut ObjectBuilder) {
        for (resource, response) in responses {
            let response_json = match response.as_ref() {
                Some(r) => ResponseSerializer.serialize(r, false),
                None => Json::Null,
            };
            json.set(resource.id.clone(), response_json);
        }
    }
}
