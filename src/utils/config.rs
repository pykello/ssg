use serde::Deserialize;
use serde_yaml::Value;
use std::{collections::HashMap, path::PathBuf};

#[derive(Deserialize)]
pub struct Config {
    pub build_dir: PathBuf,
    pub content_dir: PathBuf,
    pub template_dir: PathBuf,
    pub translation_dir: Option<PathBuf>,
    pub context: Option<HashMap<String, Value>>,
}

impl Config {
    pub fn load(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = std::fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&config_str)?;

        Ok(config)
    }
}
