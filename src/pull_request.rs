use super::config::*;
use super::loader::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct PullRequest {
    title: String,
    id: String,
}

#[derive(Serialize, Debug)]
struct PullRequestVariablePack {
    assignee: String,
    cursor: Option<String>,
}

impl PullRequestVariablePack {
    fn new(assignee: String) -> PullRequestVariablePack {
        PullRequestVariablePack {
            assignee,
            cursor: None,
        }
    }
}

impl CursorAble for PullRequestVariablePack {
    fn set_cursor(&mut self, cursor: Option<String>) {
        self.cursor = cursor;
    }
}

pub fn load_pull_requests(config: &Config) -> Vec<PullRequest> {
    let mut prs = Vec::new();

    let variables = PullRequestVariablePack::new("itarato".into());

    let mut loader = GraphQLLoader::new(
        "./graphql/pull_requests.graphql".into(),
        config.github_api_token.clone(),
        config.user.clone(),
        variables,
    );

    loader.load(&mut |val: &Value| {
        let pr_list = val
            .get("data")
            .and_then(|val| val.get("user"))
            .and_then(|val| val.get("pullRequests"))
            .and_then(|val| val.get("edges"))
            .and_then(|val| val.as_array())
            .unwrap();

        let mut last_cursor: Option<String> = None;
        for pr_node in pr_list {
            last_cursor = Some(
                pr_node
                    .get("cursor")
                    .and_then(|val| val.as_str())
                    .unwrap()
                    .into(),
            );

            let pr: PullRequest =
                serde_json::from_value(pr_node.clone().get("node").unwrap().clone()).unwrap();
            prs.push(pr);
        }

        last_cursor
    });

    prs
}
