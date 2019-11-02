use super::config::*;
use super::loader::*;
use serde::Serialize;
use serde_json::Value;

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

pub fn load_issues(config: &Config, assignee: String) -> Vec<Issue> {
    let mut issues = Vec::new();

    let variables = IssueVariablePack::new(config.org.clone(), config.repo.clone(), Some(assignee));
    let mut loader = GraphQLLoader::new(
        "./graphql/issues.graphql".into(),
        config.github_api_token.clone(),
        config.user.clone(),
        variables,
    );

    loader.load(&mut |val: &Value| {
        let issue_list: Option<&Vec<Value>> = val
            .get("data")
            .and_then(|v| v.get("repository"))
            .and_then(|v| v.get("issues"))
            .and_then(|v| v.get("edges"))
            .and_then(|v| v.as_array());

        let mut last_cursor: Option<String> = None;

        issue_list.map(|issue_list| {
            let mut converted: Vec<Issue> = issue_list
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
            issues.append(&mut converted);
        });

        last_cursor
    });

    issues
}

#[derive(Serialize, Debug)]
pub struct IssueVariablePack {
    org: String,
    repo: String,
    assignee: Option<String>,
    cursor: Option<String>,
}

impl IssueVariablePack {
    pub fn new(org: String, repo: String, assignee: Option<String>) -> IssueVariablePack {
        IssueVariablePack {
            org,
            repo,
            assignee,
            cursor: None,
        }
    }
}

impl CursorAble for IssueVariablePack {
    fn set_cursor(&mut self, cursor: Option<String>) {
        self.cursor = cursor;
    }
}
