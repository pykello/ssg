use std::path::PathBuf;

use crate::config::Config;

pub fn get_test_config() -> Config {
    // Create a test configuration
    Config {
        content_dir: PathBuf::from("src/test_assets"),
        build_dir: PathBuf::from("build"),
        template_dir: PathBuf::from("templates"),
        translation_dir: None,
        context: None,
        ..Default::default()
    }
}
