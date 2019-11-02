use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT};

pub fn github_headers(api_token: String) -> HeaderMap {
    let mut hm = HeaderMap::new();
    hm.insert(
        AUTHORIZATION,
        HeaderValue::from_str(format!("bearer {}", api_token).as_ref()).unwrap(),
    );
    hm.insert(USER_AGENT, HeaderValue::from_static("itarato"));
    hm
}
