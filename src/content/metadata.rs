use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::Config;

use super::content::{content_output_path, content_url};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ContentKind {
    Problem,
    Blog,
    Page,
    #[default]
    Unknown,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct ContentMetadata {
    pub title: String,
    pub author: Option<String>,
    pub id: Option<String>,
    pub tags: Option<Vec<String>>,
    pub timestamp: Option<String>,
    pub language: Option<String>,
    pub image: Option<PathBuf>,
    #[serde(rename = "type")]
    pub kind: ContentKind,

    #[serde(skip_deserializing, default)]
    pub output_path: PathBuf,
    #[serde(skip_deserializing, default)]
    pub url: String,
}

impl ContentMetadata {
    pub fn load(path: &Path, config: &Config) -> Result<ContentMetadata, Box<dyn Error>> {
        let yaml = fs::read_to_string(path.join("metadata.yaml"))?;
        let mut meta: Self = serde_yaml::from_str(&yaml)?;

        meta.output_path = content_output_path(path, config)?;
        meta.url = content_url(path, config)?;

        Ok(meta)
    }
}

#[cfg(test)]
mod tests {
    use super::super::test::get_test_config;
    use super::*;
    use std::path::Path;

    #[test]
    fn test_load_metadata_p1() {
        // Path to test asset p1
        let sample_path = Path::new("src/test_assets/problems/p1");

        // Load metadata from the sample problem
        let metadata = ContentMetadata::load(sample_path, &get_test_config())
            .expect("Failed to load metadata");

        // Verify the loaded metadata matches expectations
        assert_eq!(metadata.title, "Sample Problem");
        assert_eq!(metadata.id, Some("sample-problem-001".to_string()));
        assert_eq!(
            metadata.tags,
            Some(vec![
                "sample".to_string(),
                "yaml".to_string(),
                "tutorial".to_string()
            ])
        );
        assert_eq!(metadata.timestamp, Some("2025-03-06T12:00:00Z".to_string()));
    }

    #[test]
    fn test_metadata_file_not_found() {
        // Create a temporary directory without a metadata file
        let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");

        // Attempt to load metadata from a directory without a metadata.yaml file
        let result = ContentMetadata::load(temp_dir.path(), &get_test_config());

        // Verify that the function returns an error
        assert!(result.is_err());
    }
}
