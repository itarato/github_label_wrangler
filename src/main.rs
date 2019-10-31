extern crate hyper;
extern crate hyper_tls;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use hyper::rt::{self, Future, Stream};
use hyper::{Body, Client, Method, Request};
use hyper_tls::HttpsConnector;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::File;
use std::io::Read;
use std::str;

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    github_api_token: String,
}

impl Config {
    fn load() -> Result<Config, ()> {
        let mut raw_json = String::new();
        let mut config_file = File::open("config.json").map_err(|_| ())?;
        config_file.read_to_string(&mut raw_json).map_err(|_| ())?;

        let config: Config = serde_json::from_str(raw_json.as_ref()).map_err(|_| ())?;

        Ok(config)
    }
}

fn main() {
    let uri: hyper::Uri = "https://api.github.com/graphql".parse().unwrap();
    let config = Config::load().unwrap();
    rt::run(fetch_and_run(uri, &config));
}

fn fetch_and_run(uri: hyper::Uri, config: &Config) -> impl Future<Item = (), Error = ()> {
    let connector = HttpsConnector::new(4).unwrap();
    let client = Client::builder().build::<_, hyper::Body>(connector);

    let mut json_content: String = String::new();
    let mut json_file = File::open("./graphql/issues.graphql").unwrap();
    let _ = json_file.read_to_string(&mut json_content).unwrap();

    json_content = json_content
        .chars()
        .filter(|&ch| ch != '\n')
        .collect::<String>();

    let json = format!(r#"{{"query": {:?}}}"#, json_content);
    let mut req = Request::new(Body::from(json));
    *req.method_mut() = Method::POST;
    *req.uri_mut() = uri.clone();

    req.headers_mut().insert(
        hyper::header::AUTHORIZATION,
        hyper::header::HeaderValue::from_str(
            format!("bearer {}", config.github_api_token).as_ref(),
        )
        .unwrap(),
    );
    req.headers_mut().insert(
        hyper::header::USER_AGENT,
        hyper::header::HeaderValue::from_static("itarato"),
    );

    client
        .request(req)
        .and_then(|res| res.into_body().concat2())
        .map(|a| {
            // let res = dbg!(str::from_utf8(&a.into_bytes()).unwrap());
            let v: Value = serde_json::from_str(str::from_utf8(&a.into_bytes()).unwrap()).unwrap();
            for x in serde_json_dig::dig(
                v.clone(),
                &[
                    serde_json_dig::Shovel::Key("data"),
                    serde_json_dig::Shovel::Key("repository"),
                    serde_json_dig::Shovel::Key("issues"),
                    serde_json_dig::Shovel::Key("edges"),
                ],
            )
            .unwrap()
            .as_array()
            .unwrap()
            {
                println!(
                    "Issue: {}",
                    x.as_object().unwrap()["node"].as_object().unwrap()["title"]
                );
            }

            ()
        })
        .map_err(|err| {
            eprintln!("Error {}", err);
        })
}
