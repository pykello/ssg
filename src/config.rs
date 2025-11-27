use serde::Deserialize;
use serde_yaml::Value;
use std::{collections::HashMap, path::PathBuf};

use crate::formatted_text::Theorem;

fn default_language() -> String {
    "en".to_string()
}

fn default_text_direction() -> String {
    "ltr".to_string()
}

fn default_escape_markdown_in_math() -> bool {
    true
}

/*
 * Options:
 * `base16-ocean.dark`,`base16-eighties.dark`,`base16-mocha.dark`,`base16-ocean.light`
 * `InspiredGitHub`, `Solarized (dark)` and `Solarized (light)`
 */
fn default_syntax_highlighter_theme() -> String {
    "base16-ocean.dark".to_string()
}

#[derive(Deserialize, Default)]
pub struct Config {
    pub build_dir: PathBuf,
    pub content_dir: PathBuf,
    pub template_dir: PathBuf,
    pub translations_csv: Option<PathBuf>,

    #[serde(default = "default_syntax_highlighter_theme")]
    pub syntax_highlighter_theme: String,

    #[serde(default = "default_language")]
    pub language: String,

    #[serde(default = "default_text_direction")]
    pub text_direction: String,

    pub context: Option<HashMap<String, Value>>,

    #[serde(default)]
    pub theorems: Vec<Theorem>,

    #[serde(default = "default_escape_markdown_in_math")]
    pub escape_markdown_in_math: bool,
}

impl Config {
    pub fn load(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = std::fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&config_str)?;

        Ok(config)
    }
}
