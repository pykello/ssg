use std::path::PathBuf;

use crate::config::Config;

pub fn get_test_config() -> Config {
    // Create a test configuration
    Config {
        content_dir: PathBuf::from("src/test_assets"),
        build_dir: PathBuf::from("build"),
        template_dir: PathBuf::from("templates"),
        syntax_highlighter_theme: "base16-ocean.dark".to_string(),
        raw_math_blocks: true,
        ..Default::default()
    }
}
