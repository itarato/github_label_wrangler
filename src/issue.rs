use super::config::*;
use super::loader::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::File;
use std::io::Read;

#[derive(Debug)]
pub struct Issue {
    title: String,
    labels: Vec<String>,
}

impl Issue {
    pub fn new(title: String, labels: Vec<String>) -> Issue {
        Issue { title, labels }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IssueLoaderVariablePack {
    org: String,
    repo: String,
    assignee: Option<String>,
    cursor: Option<String>,
}

impl IssueLoaderVariablePack {
    pub fn new(
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

pub struct IssueLoader<'a> {
    config: &'a Config,
    pub issues: Vec<Issue>,
    assignee: Option<String>,
}

impl<'a> IssueLoader<'a> {
    pub fn new(config: &'a Config, assignee: Option<String>) -> IssueLoader {
        IssueLoader {
            config,
            issues: Vec::new(),
            assignee,
        }
    }

    pub fn load(&mut self) {
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
