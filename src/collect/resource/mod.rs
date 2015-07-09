use std::collections::HashMap;

use rustc_serialize::json::Json;

pub mod client;

#[derive(Hash, Eq, PartialEq, Debug)]
pub struct Resource {
    pub id: String,
    pub url: String,
}

pub type Resources = Vec<Resource>;

pub struct Response {
    pub url: String,
    pub status_code: u8,
    pub json: Json,
}

pub type Responses = HashMap<Resource, Response>;
