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
    dbg!(load_issues(&config));
    dbg!(load_pull_requests(&config));
}
