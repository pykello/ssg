use super::metadata::{ContentKind, ContentMetadata};
use crate::config::Config;
use crate::formatted_text::FormattedText;
use std::error::Error;
use std::path::{Path, PathBuf};

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
    pub fn load(path: &Path, config: &Config) -> Result<Content, Box<dyn Error>> {
        if !path.is_dir() {
            return load_bare_page(path, config);
        }

        let metadata = ContentMetadata::load(path, config)?;

        match metadata.kind {
            ContentKind::Problem => super::problem::load_problem(path, metadata),
            ContentKind::Blog => {
                load_single_content_file(path, metadata, "body", |metadata, body| Content::Blog {
                    metadata,
                    body,
                })
            }
            ContentKind::Page => {
                load_single_content_file(path, metadata, "body", |metadata, body| Content::Page {
                    metadata,
                    body,
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

/// Load a Markdown file and expand simple `#include "file"` directives.
///
/// Includes are resolved relative to the directory of `path` and are not
/// processed recursively.
pub(super) fn load_markdown_with_includes(path: &Path) -> Result<String, Box<dyn Error>> {
    let content = std::fs::read_to_string(path)?;
    let base_dir = path.parent().unwrap_or(Path::new(""));

    let mut out = String::new();
    let lines: Vec<&str> = content.lines().collect();
    let ends_with_newline = content.ends_with('\n');

    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("#include") {
            let include_path = trimmed.trim_start_matches("#include").trim();
            let include_path = include_path.trim_matches('"');
            let include_file = base_dir.join(include_path);
            let included = std::fs::read_to_string(include_file)?;
            out.push_str(&included);
        } else {
            out.push_str(line);
        }

        if idx < lines.len() - 1 || ends_with_newline {
            out.push('\n');
        }
    }

    Ok(out)
}

fn load_bare_page(path: &Path, config: &Config) -> Result<Content, Box<dyn Error>> {
    let extension = path.extension().and_then(|s| s.to_str());
    let mut metadata = ContentMetadata::default();
    metadata.kind = ContentKind::Page;
    metadata.output_path = content_output_path(path, config)?;
    metadata.url = content_url(path, config)?;
    match extension {
        Some("md") => {
            let text = load_markdown_with_includes(path)?;
            let lines = text.lines().collect::<Vec<_>>();
            if lines.len() > 0 && (lines[0].starts_with("# ") || lines[0].starts_with("## ")) {
                metadata.title = lines[0]
                    .replace("## ", "")
                    .replace("# ", "")
                    .trim()
                    .to_string();
            }
            Ok(Content::Page {
                metadata: metadata,
                body: FormattedText::Markdown(text),
            })
        }
        Some("tex") => {
            let text = std::fs::read_to_string(path)?;
            Ok(Content::Page {
                metadata: metadata,
                body: FormattedText::Latex(text),
            })
        }
        Some("html") => {
            let text = std::fs::read_to_string(path)?;
            Ok(Content::Page {
                metadata: metadata,
                body: FormattedText::Html(text),
            })
        }
        _ => Err(format!("Unsupported file type: {:?}", extension).into()),
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
    let html_file = base_path.join(format!("{}.html", file_basename));

    let content = if md_file.exists() {
        let text = load_markdown_with_includes(&md_file)?;
        FormattedText::Markdown(text)
    } else if tex_file.exists() {
        let text = fs::read_to_string(tex_file)?;
        FormattedText::Latex(text)
    } else if html_file.exists() {
        let text = fs::read_to_string(html_file)?;
        FormattedText::Html(text)
    } else {
        return Err(format!("No {} file found", file_basename).into());
    };

    Ok(constructor(metadata, content))
}

pub fn content_output_path(
    path: &Path,
    config: &Config,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let cwd = std::env::current_dir()?;
    let path = path.with_extension("");
    let path = cwd.join(path);
    let content_dir = cwd.join(&config.content_dir);
    let rel_path = path.strip_prefix(content_dir.clone()).map_err(|_e| {
        format!(
            "Path {} is not a subpath of content directory {}",
            path.display(),
            content_dir.display()
        )
    })?;

    // Create output file path that preserves directory structure
    let mut output_file_path = config.build_dir.join(rel_path);
    output_file_path.set_extension("html");

    Ok(output_file_path)
}

pub fn content_url(path: &Path, config: &Config) -> Result<String, Box<dyn std::error::Error>> {
    let output_path = content_output_path(path, config)?;
    let rel_path = output_path.strip_prefix(&config.build_dir).map_err(|_e| {
        format!(
            "Path {} is not a subpath of build directory {}",
            output_path.display(),
            config.build_dir.display()
        )
    })?;
    let url = rel_path.to_string_lossy().to_string();
    let url = url.replace("\\", "/"); // Normalize path separators for URLs
    Ok(format!("/{}", url))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_load_page_content() -> Result<(), Box<dyn std::error::Error>> {
        // Create a temporary test directory with page content
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();

        let config = Config {
            content_dir: PathBuf::from("/tmp"),
            build_dir: PathBuf::from("/tmp/build"),
            template_dir: PathBuf::from("/tmp/templates"),
            ..Default::default()
        };

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

        // Create body.md
        fs::write(
            temp_path.join("body.md"),
            "# Test Page Content\n\nThis is a test page with some *markdown* content.",
        )
        .unwrap();

        // Load the page content
        let content = Content::load(temp_path, &config)?;

        // Check that it loaded as a Page type
        assert!(matches!(content, Content::Page { .. }));

        // Verify metadata
        let metadata = content.metadata();
        assert_eq!(metadata.title, "Test Page");
        assert_eq!(metadata.id, Some("test-page".to_string()));

        // Verify content by rendering to HTML
        if let Content::Page { body, .. } = content {
            let html = body.to_html(&config).unwrap();
            assert!(html.contains("<h1"));
            assert!(html.contains("Test Page Content"));
            assert!(html.contains("<em>markdown</em>"));
        } else {
            panic!("Expected Page content type");
        }

        Ok(())
    }

    #[test]
    fn test_markdown_include() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();

        let config = Config {
            content_dir: PathBuf::from("/tmp"),
            build_dir: PathBuf::from("/tmp/build"),
            template_dir: PathBuf::from("/tmp/templates"),
            ..Default::default()
        };

        fs::write(
            temp_path.join("metadata.yaml"),
            "title: 'Inc'\ntype: 'page'",
        )?;

        fs::write(temp_path.join("part.md"), "Included text")?;

        fs::write(temp_path.join("body.md"), "# H\n#include \"part.md\"")?;

        let content = Content::load(temp_path, &config)?;
        if let Content::Page { body, .. } = content {
            let html = body.to_html(&config)?;
            assert!(html.contains("Included text"));
        } else {
            panic!("Expected Page content type");
        }

        Ok(())
    }

    #[test]
    fn test_content_output_path_abs() -> Result<(), Box<dyn std::error::Error>> {
        let conf = Config {
            content_dir: PathBuf::from("/content"),
            build_dir: PathBuf::from("/build"),
            template_dir: PathBuf::from("/templates"),
            ..Default::default()
        };

        let path = Path::new("/content/page1.md");
        let output_path = content_output_path(path, &conf)?;
        assert_eq!(output_path, Path::new("/build/page1.html"));

        Ok(())
    }

    #[test]
    fn test_content_output_path_rel() -> Result<(), Box<dyn std::error::Error>> {
        let conf = Config {
            content_dir: PathBuf::from("content"),
            build_dir: PathBuf::from("build"),
            ..Default::default()
        };

        let path = Path::new("content/subdir/page1.md");
        let output_path = content_output_path(path, &conf)?;
        assert_eq!(output_path, Path::new("build/subdir/page1.html"));

        Ok(())
    }

    #[test]
    fn test_content_url() -> Result<(), Box<dyn std::error::Error>> {
        let conf = Config {
            content_dir: PathBuf::from("content"),
            build_dir: PathBuf::from("build"),
            ..Default::default()
        };

        let path = Path::new("content/subdir/page1.md");
        let url = content_url(path, &conf)?;
        assert_eq!(url, "/subdir/page1.html");

        Ok(())
    }
}
