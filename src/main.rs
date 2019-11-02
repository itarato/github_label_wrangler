extern crate reqwest;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;

mod config;
mod issue;
mod loader;

use config::*;
use issue::*;

fn main() {
    let config = Config::load().unwrap();
    dbg!(load_issues(&config));
}
