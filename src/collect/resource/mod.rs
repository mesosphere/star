use std::collections::HashMap;

use rustc_serialize::json;

pub mod client;

#[derive(Clone, Debug, Hash, Eq, PartialEq, RustcDecodable)]
pub struct Resource {
    pub id: String,
    pub url: String,
}

pub type Resources = Vec<Resource>;

#[derive(Clone)]
pub struct Response {
    pub url: String,
    pub status_code: u16,
    pub json: json::Json,
}

pub type Responses = HashMap<Resource, Option<Response>>;

pub struct ResourceStore {
    responses: Responses,
}

impl ResourceStore {
    pub fn new(resources: Vec<Resource>) -> ResourceStore {
        let mut result = ResourceStore {
            responses: Responses::new(),
        };
        for resource in resources.iter() {
            result.responses.insert(resource.clone(), None);
        }
        result
    }

    pub fn resources(&self) -> Vec<Resource> {
        self.responses.keys().map(|r| r.clone()).collect()
    }

    pub fn save_resource(&mut self, resource: Resource) {
        self.save_response(resource, None);
    }

    pub fn responses(&self) -> Responses {
        self.responses.clone()
    }

    pub fn save_response(&mut self,
                         resource: Resource,
                         response: Option<Response>) {
        self.responses.insert(resource, response);
    }
}
