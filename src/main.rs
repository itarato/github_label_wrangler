extern crate reqwest;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;

mod config;
mod issue;
mod loader;
mod pull_request;

use config::*;
use issue::*;
use pull_request::*;

fn main() {
    let config = Config::load().unwrap();
    let assignee: String = "itarato".into();
    dbg!(load_issues(&config, assignee.clone()));
    dbg!(load_pull_requests(&config, assignee.clone()));
}
