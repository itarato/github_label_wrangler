use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::File;
use std::io::Read;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub github_api_token: String,
    pub user: String,
    pub org: String,
    pub repo: String,
}

impl Config {
    pub fn load() -> Result<Config, ()> {
        let mut raw_json = String::new();
        let mut config_file = File::open("config.json").map_err(|_| ())?;
        config_file.read_to_string(&mut raw_json).map_err(|_| ())?;

        let config: Config = serde_json::from_str(raw_json.as_ref()).map_err(|_| ())?;

        Ok(config)
    }
}
