extern crate reqwest;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;

mod config;
mod issue;
mod loader;

use config::*;
use issue::*;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT};
use serde::{Deserialize, Serialize};
use serde_json::Value;

fn main() {
    let config = Config::load().unwrap();
    let mut il = IssueLoader::new(&config, Some("itarato".into()));
    il.load();
    dbg!(il.issues);
}
