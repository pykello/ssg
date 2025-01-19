use super::content_metadata::ContentMetadata;
use super::formatted_text::FormattedText;
use std::error::Error;
use std::path::Path;

#[derive(Debug)]
pub enum Content {
    Problem(
        ContentMetadata,
        FormattedText,
        Vec<FormattedText>,
        Vec<FormattedText>,
    ),
    Blog(ContentMetadata, FormattedText),
    Page(ContentMetadata, FormattedText),
    // Add other content types as needed
}

impl Content {
    pub fn content_type(&self) -> &'static str {
        match self {
            Content::Problem(_, _, _, _) => "problem",
            Content::Blog(_, _) => "blog",
            Content::Page(_, _) => "page",
        }
    }

    // Factory function to load content based on type
    pub fn load(path: &Path) -> Result<Content, Box<dyn Error>> {
        let base_path = path;

        // First load metadata to determine content type
        let metadata = ContentMetadata::load(base_path)?;

        // Then load the appropriate content based on type
        match metadata.r#type.as_str() {
            "problem" => super::problem::load_problem(base_path, metadata),
            "blog" => load_single_content_file(base_path, metadata, "body", Content::Blog),
            "page" => load_single_content_file(base_path, metadata, "content", Content::Page),
            _ => Err(format!("Unknown content type: {}", metadata.r#type).into()),
        }
    }

    pub fn render_html(
        &self,
        renderer: &crate::utils::render::Renderer,
    ) -> Result<String, Box<dyn Error>> {
        match self {
            Content::Problem(metadata, problem, solutions, hints) => {
                // Convert FormattedText to HTML strings
                let problem_html = problem.to_html()?;
                let solution_htmls: Vec<String> =
                    solutions.iter().filter_map(|s| s.to_html().ok()).collect();
                let hint_htmls: Vec<String> =
                    hints.iter().filter_map(|h| h.to_html().ok()).collect();

                // Create a context for the template
                use serde_json::json;
                use std::collections::HashMap;

                let mut context = HashMap::new();
                context.insert(
                    "problem".to_string(),
                    json!({
                        "title": metadata.title,
                        "id": metadata.id,
                        "tags": metadata.tags,
                        "timestamp": metadata.timestamp,
                        "statement": problem_html,
                        "solutions": solution_htmls,
                        "hints": hint_htmls,
                        "image": metadata.image,
                    }),
                );

                // Render the problem template
                renderer
                    .render("problem.html", context)
                    .map_err(|e| e.into())
            }
            Content::Blog(metadata, body) => {
                // Convert body to HTML
                let body_html = body.to_html()?;

                // Create a context for the template
                use serde_json::json;
                use std::collections::HashMap;

                let mut context = HashMap::new();
                context.insert(
                    "blog".to_string(),
                    json!({
                        "title": metadata.title,
                        "id": metadata.id,
                        "tags": metadata.tags,
                        "timestamp": metadata.timestamp,
                        "body": body_html,
                    }),
                );

                // Render the blog template
                renderer.render("blog.html", context).map_err(|e| e.into())
            }
            Content::Page(metadata, body) => {
                // Convert body to HTML
                let body_html = body.to_html()?;

                // Create a context for the template
                use serde_json::json;
                use std::collections::HashMap;

                let mut context = HashMap::new();
                context.insert(
                    "page".to_string(),
                    json!({
                        "title": metadata.title,
                        "id": metadata.id,
                        "body": body_html,
                    }),
                );

                // Render the page template - simpler than blog template
                renderer.render("page.html", context).map_err(|e| e.into())
            }
        }
    }

    pub fn metadata(&self) -> &ContentMetadata {
        match self {
            Content::Problem(metadata, _, _, _) => metadata,
            Content::Blog(metadata, _) => metadata,
            Content::Page(metadata, _) => metadata,
        }
    }
}

/// Helper function to load a single content file (used by Blog and Page types)
fn load_single_content_file<F, P: AsRef<std::path::Path>>(
    base_path: P,
    metadata: ContentMetadata,
    file_basename: &str,
    constructor: F,
) -> Result<Content, Box<dyn Error>>
where
    F: FnOnce(ContentMetadata, FormattedText) -> Content,
{
    use std::fs;

    let base_path = base_path.as_ref();
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
        assert_eq!(content.content_type(), "page");

        // Verify metadata
        let metadata = content.metadata();
        assert_eq!(metadata.title, "Test Page");
        assert_eq!(metadata.id, Some("test-page".to_string()));

        // Verify content by rendering to HTML
        if let Content::Page(_, body) = content {
            let html = body.to_html().unwrap();
            assert!(html.contains("<h1"));
            assert!(html.contains("Test Page Content"));
            assert!(html.contains("<em>markdown</em>"));
        } else {
            panic!("Expected Page content type");
        }
    }
}
