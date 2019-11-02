use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT};
use serde::Serialize;
use serde_json::Value;
use std::fs::File;
use std::io::Read;

static GITHUB_API_URL_V4: &'static str = "https://api.github.com/graphql";

pub trait CursorAble {
    fn set_cursor(&mut self, cursor: Option<String>);
}

pub struct GraphQLLoader<V: Serialize + CursorAble> {
    query_file_name: String,
    api_token: String,
    app_name: String,
    variables: V,
}

impl<V: Serialize + CursorAble> GraphQLLoader<V> {
    pub fn new(
        query_file_name: String,
        api_token: String,
        app_name: String,
        variables: V,
    ) -> GraphQLLoader<V> {
        GraphQLLoader {
            query_file_name,
            api_token,
            app_name,
            variables,
        }
    }

    pub fn load(&mut self, collector: &mut dyn FnMut(&Value) -> Option<String>) {
        let mut cursor: Option<String> = None;
        loop {
            let current_cursor = cursor.clone();
            cursor = self.fetch_page(current_cursor, collector);

            if cursor.is_none() {
                break;
            }
        }
    }

    fn fetch_page(
        &mut self,
        cursor: Option<String>,
        collector: &mut dyn FnMut(&Value) -> Option<String>,
    ) -> Option<String> {
        let mut json_content: String = String::new();
        let mut json_file = File::open(self.query_file_name.clone()).unwrap();
        let _ = json_file.read_to_string(&mut json_content).unwrap();

        json_content = json_content
            .chars()
            .filter(|&ch| ch != '\n')
            .collect::<String>();

        self.variables.set_cursor(cursor);
        let json = format!(
            r#"{{"query": {:?}, "variables": {:?} }}"#,
            json_content,
            serde_json::to_string(&self.variables).unwrap(),
        );

        let cli = reqwest::Client::new();
        let raw_result = cli
            .post(GITHUB_API_URL_V4)
            .headers(github_headers(
                self.api_token.clone(),
                self.app_name.clone(),
            ))
            .body(json)
            .send()
            .unwrap()
            .text()
            .unwrap();

        let result_json: Value = serde_json::from_str(raw_result.as_ref()).unwrap();

        collector(&result_json)
    }
}

pub fn github_headers(api_token: String, app_name: String) -> HeaderMap {
    let mut hm = HeaderMap::new();
    hm.insert(
        AUTHORIZATION,
        HeaderValue::from_str(format!("bearer {}", api_token).as_ref()).unwrap(),
    );
    hm.insert(
        USER_AGENT,
        HeaderValue::from_str(app_name.as_ref()).unwrap(),
    );
    hm
}
