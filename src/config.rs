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

    /// If set to true, math blocks wrapped in `$$` are left untouched by the
    /// markdown renderer and output verbatim.
    #[serde(default = "default_raw_math_blocks")]
    pub raw_math_blocks: bool,
}

fn default_raw_math_blocks() -> bool {
    false
}

impl Config {
    pub fn load(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = std::fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&config_str)?;

        Ok(config)
    }
}
