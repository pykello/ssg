use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq)]
pub struct ContentMetadata {
    pub title: String,
    pub author: Option<String>,
    pub id: Option<String>,
    pub tags: Option<Vec<String>>,
    pub timestamp: Option<String>,
    pub language: Option<String>,
    pub image: Option<PathBuf>,
    #[serde(default)]
    pub r#type: String,
}

impl ContentMetadata {
    pub fn load(path: &Path) -> Result<ContentMetadata, Box<dyn Error>> {
        let metadata_path = path.join("metadata.yaml");
        let metadata_content = fs::read_to_string(&metadata_path)?;
        let meta: ContentMetadata = serde_yaml::from_str(&metadata_content)?;

        Ok(meta)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_load_metadata_p1() {
        // Path to test asset p1
        let sample_path = Path::new("src/test_assets/problems/p1");

        // Load metadata from the sample problem
        let metadata = ContentMetadata::load(sample_path).expect("Failed to load metadata");

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
    fn test_metadata_missing_fields() {
        // Create a temporary directory for testing
        let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
        let temp_path = temp_dir.path();

        // Create a minimal metadata file
        let metadata_content = r#"
title: "Minimal Metadata"
"#;

        std::fs::create_dir_all(temp_path).expect("Failed to create temp directories");
        std::fs::write(temp_path.join("metadata.yaml"), metadata_content)
            .expect("Failed to write metadata file");

        // Load the metadata
        let metadata = ContentMetadata::load(temp_path).expect("Failed to load metadata");

        // Verify the loaded metadata
        assert_eq!(metadata.title, "Minimal Metadata");
        assert_eq!(metadata.id, None);
        assert_eq!(metadata.tags, None);
        assert_eq!(metadata.timestamp, None);
        assert_eq!(metadata.r#type, ""); // Default value for type field
    }

    #[test]
    fn test_metadata_file_not_found() {
        // Create a temporary directory without a metadata file
        let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");

        // Attempt to load metadata from a directory without a metadata.yaml file
        let result = ContentMetadata::load(temp_dir.path());

        // Verify that the function returns an error
        assert!(result.is_err());
    }
}
