extern crate reqwest;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::File;
use std::io::Read;

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    github_api_token: String,
    user: String,
    org: String,
    repo: String,
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

fn github_headers(api_token: String) -> HeaderMap {
    let mut hm = HeaderMap::new();
    hm.insert(
        AUTHORIZATION,
        HeaderValue::from_str(format!("bearer {}", api_token).as_ref()).unwrap(),
    );
    hm.insert(USER_AGENT, HeaderValue::from_static("itarato"));
    hm
}

#[derive(Debug)]
struct Issue {
    title: String,
    labels: Vec<String>,
}

impl Issue {
    fn new(title: String, labels: Vec<String>) -> Issue {
        Issue { title, labels }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct IssueLoaderVariablePack {
    org: String,
    repo: String,
    assignee: Option<String>,
    cursor: Option<String>,
}

impl IssueLoaderVariablePack {
    fn new(
        org: String,
        repo: String,
        assignee: Option<String>,
        cursor: Option<String>,
    ) -> IssueLoaderVariablePack {
        IssueLoaderVariablePack {
            org,
            repo,
            assignee,
            cursor,
        }
    }
}

struct IssueLoader<'a> {
    config: &'a Config,
    issues: Vec<Issue>,
    assignee: Option<String>,
}

impl<'a> IssueLoader<'a> {
    fn new(config: &'a Config, assignee: Option<String>) -> IssueLoader {
        IssueLoader {
            config,
            issues: Vec::new(),
            assignee,
        }
    }

    fn load(&mut self) {
        let mut cursor: Option<String> = None;
        loop {
            let current_cursor = cursor.clone();
            cursor = self.fetch_page(current_cursor);

            if cursor.is_none() {
                break;
            }
        }
    }

    fn fetch_page(&mut self, cursor: Option<String>) -> Option<String> {
        let mut json_content: String = String::new();
        let mut json_file = File::open("./graphql/issues.graphql").unwrap();
        let _ = json_file.read_to_string(&mut json_content).unwrap();

        json_content = json_content
            .chars()
            .filter(|&ch| ch != '\n')
            .collect::<String>();

        let variable_pack = IssueLoaderVariablePack::new(
            self.config.org.clone(),
            self.config.repo.clone(),
            self.assignee.clone(),
            cursor,
        );

        let json = format!(
            r#"{{"query": {:?}, "variables": {:?} }}"#,
            json_content,
            serde_json::to_string(&variable_pack).unwrap(),
        );

        let cli = reqwest::Client::new();
        let raw_result = cli
            .post("https://api.github.com/graphql")
            .headers(github_headers(self.config.github_api_token.clone()))
            .body(json)
            .send()
            .unwrap()
            .text()
            .unwrap();

        let result_json: Value = serde_json::from_str(raw_result.as_ref()).unwrap();
        let issues: Option<&Vec<Value>> = result_json
            .get("data")
            .and_then(|v| v.get("repository"))
            .and_then(|v| v.get("issues"))
            .and_then(|v| v.get("edges"))
            .and_then(|v| v.as_array());

        let mut last_cursor: Option<String> = None;

        issues.map(|issues| {
            println!("Fetched {} issues.", issues.len());
            let mut converted: Vec<Issue> = issues
                .into_iter()
                .map(|issue| {
                    last_cursor = Some(
                        issue
                            .get("cursor")
                            .and_then(|val| val.as_str())
                            .unwrap()
                            .into(),
                    );

                    let label_nodes: &Vec<Value> = issue
                        .get("node")
                        .and_then(|val| val.get("labels"))
                        .and_then(|val| val.get("edges"))
                        .and_then(|val| val.as_array())
                        .unwrap();

                    let labels: Vec<String> = label_nodes
                        .into_iter()
                        .map(|node| {
                            node.get("node")
                                .and_then(|val| val.get("name"))
                                .and_then(|val| val.as_str())
                                .unwrap()
                                .into()
                        })
                        .collect();

                    Issue::new(
                        issue
                            .get("node")
                            .and_then(|val| val.get("title"))
                            .and_then(|val| val.as_str())
                            .unwrap()
                            .into(),
                        labels,
                    )
                })
                .collect();
            self.issues.append(&mut converted);
        });

        last_cursor
    }
}

fn main() {
    let config = Config::load().unwrap();
    let mut il = IssueLoader::new(&config, Some("itarato".into()));
    il.load();
    dbg!(il.issues);
}
