use serde::Deserialize;
use serde_yaml::Value;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

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

fn default_math_shorthand() -> bool {
    false
}

/*
 * Options:
 * `base16-ocean.dark`,`base16-eighties.dark`,`base16-mocha.dark`,`base16-ocean.light`
 * `InspiredGitHub`, `Solarized (dark)` and `Solarized (light)`
 */
fn default_syntax_highlighter_theme() -> String {
    "base16-ocean.dark".to_string()
}

fn default_pandoc_timeout_seconds() -> u64 {
    10
}

#[derive(Deserialize)]
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

    #[serde(default = "default_math_shorthand")]
    pub math_shorthand: bool,

    #[serde(default = "default_pandoc_timeout_seconds")]
    pub pandoc_timeout_seconds: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            build_dir: PathBuf::new(),
            content_dir: PathBuf::new(),
            template_dir: PathBuf::new(),
            translations_csv: None,
            syntax_highlighter_theme: default_syntax_highlighter_theme(),
            language: default_language(),
            text_direction: default_text_direction(),
            context: None,
            theorems: Vec::new(),
            escape_markdown_in_math: default_escape_markdown_in_math(),
            math_shorthand: default_math_shorthand(),
            pandoc_timeout_seconds: default_pandoc_timeout_seconds(),
        }
    }
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = std::fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&config_str)?;

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_uses_runtime_defaults() {
        let config = Config::default();

        assert_eq!(config.language, "en");
        assert_eq!(config.text_direction, "ltr");
        assert!(config.escape_markdown_in_math);
        assert!(!config.math_shorthand);
        assert_eq!(config.pandoc_timeout_seconds, 10);
    }

    #[test]
    fn load_uses_default_pandoc_timeout_when_omitted() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempfile::tempdir()?;
        let config_path = temp_dir.path().join("config.yaml");
        std::fs::write(
            &config_path,
            "build_dir: build\ncontent_dir: content\ntemplate_dir: templates\n",
        )?;

        let config = Config::load(&config_path)?;

        assert_eq!(config.pandoc_timeout_seconds, 10);

        Ok(())
    }
}
