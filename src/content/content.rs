use super::metadata::{ContentKind, ContentMetadata};
use crate::formatted_text::FormattedText;
use std::error::Error;
use std::path::Path;

#[derive(Debug)]
pub enum Content {
    Problem {
        metadata: ContentMetadata,
        statement: FormattedText,
        solutions: Vec<FormattedText>,
        hints: Vec<FormattedText>,
    },
    Blog {
        metadata: ContentMetadata,
        body: FormattedText,
    },
    Page {
        metadata: ContentMetadata,
        body: FormattedText,
    },
}

impl Content {
    // Factory function to load content based on type
    pub fn load(path: &Path) -> Result<Content, Box<dyn Error>> {
        let metadata = ContentMetadata::load(path)?;

        match metadata.kind {
            ContentKind::Problem => super::problem::load_problem(path, metadata),
            ContentKind::Blog => {
                load_single_content_file(path, metadata, "body", |metadata, body| Content::Blog {
                    metadata,
                    body,
                })
            }
            ContentKind::Page => {
                load_single_content_file(path, metadata, "content", |metadata, body| {
                    Content::Page { metadata, body }
                })
            }
            ContentKind::Unknown => {
                Err(format!("Unknown content type: {:?}", metadata.kind).into())
            }
        }
    }

    pub fn metadata(&self) -> &ContentMetadata {
        match self {
            Content::Problem { metadata, .. } => metadata,
            Content::Blog { metadata, .. } => metadata,
            Content::Page { metadata, .. } => metadata,
        }
    }
}

/// Helper function to load a single content file (used by Blog and Page types)
fn load_single_content_file<F>(
    base_path: &Path,
    metadata: ContentMetadata,
    file_basename: &str,
    constructor: F,
) -> Result<Content, Box<dyn Error>>
where
    F: FnOnce(ContentMetadata, FormattedText) -> Content,
{
    use std::fs;

    let md_file = base_path.join(format!("{}.md", file_basename));
    let tex_file = base_path.join(format!("{}.tex", file_basename));

    let content = if md_file.exists() {
        let text = fs::read_to_string(md_file)?;
        FormattedText::Markdown(text)
    } else if tex_file.exists() {
        let text = fs::read_to_string(tex_file)?;
        FormattedText::Latex(text)
    } else {
        return Err(format!("No {} file found", file_basename).into());
    };

    Ok(constructor(metadata, content))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_load_page_content() {
        // Create a temporary test directory with page content
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create metadata.yaml
        fs::write(
            temp_path.join("metadata.yaml"),
            r#"
title: "Test Page"
type: "page"
id: "test-page"
"#,
        )
        .unwrap();

        // Create content.md
        fs::write(
            temp_path.join("content.md"),
            "# Test Page Content\n\nThis is a test page with some *markdown* content.",
        )
        .unwrap();

        // Load the page content
        let content = Content::load(temp_path).unwrap();

        // Check that it loaded as a Page type
        assert!(matches!(content, Content::Page { .. }));

        // Verify metadata
        let metadata = content.metadata();
        assert_eq!(metadata.title, "Test Page");
        assert_eq!(metadata.id, Some("test-page".to_string()));

        // Verify content by rendering to HTML
        if let Content::Page { body, .. } = content {
            let html = body.to_html().unwrap();
            assert!(html.contains("<h1"));
            assert!(html.contains("Test Page Content"));
            assert!(html.contains("<em>markdown</em>"));
        } else {
            panic!("Expected Page content type");
        }
    }
}
