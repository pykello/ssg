use super::metadata::{ContentKind, ContentMetadata};
use crate::config::Config;
use crate::formatted_text::FormattedText;
use std::error::Error;
use std::path::{Path, PathBuf};

const BODY_BASENAME: &str = "body";

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
    pub fn load(path: &Path, config: &Config) -> Result<Content, Box<dyn Error>> {
        if path.is_dir() {
            load_directory_content(path, config)
        } else {
            load_bare_page(path, config)
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

fn load_directory_content(path: &Path, config: &Config) -> Result<Content, Box<dyn Error>> {
    let metadata = ContentMetadata::load(path, config)?;

    match metadata.kind {
        ContentKind::Problem => super::problem::load_problem(path, metadata),
        ContentKind::Blog => {
            load_single_content_file(path, metadata, BODY_BASENAME, |metadata, body| {
                Content::Blog { metadata, body }
            })
        }
        ContentKind::Page => {
            load_single_content_file(path, metadata, BODY_BASENAME, |metadata, body| {
                Content::Page { metadata, body }
            })
        }
        ContentKind::Unknown => Err(format!("Unknown content type: {:?}", metadata.kind).into()),
    }
}

/// Load a Markdown file and expand simple `#include "file"` directives.
///
/// Includes are resolved relative to the directory of `path` and are not
/// processed recursively.
pub(super) fn load_markdown_with_includes(path: &Path) -> Result<String, Box<dyn Error>> {
    let content = std::fs::read_to_string(path)?;
    let base_dir = path.parent().unwrap_or(Path::new(""));
    let canonical_base_dir = base_dir.canonicalize()?;

    let mut out = String::new();
    let lines: Vec<&str> = content.lines().collect();
    let ends_with_newline = content.ends_with('\n');
    let mut in_fence = false;

    for (idx, line) in lines.iter().enumerate() {
        if is_fence_line(line) {
            in_fence = !in_fence;
        }

        if in_fence {
            out.push_str(line);
        } else if let Some(included) = load_include_for_line(line, base_dir, &canonical_base_dir)? {
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

fn load_include_for_line(
    line: &str,
    base_dir: &Path,
    canonical_base_dir: &Path,
) -> Result<Option<String>, Box<dyn Error>> {
    let Some(include_path) = parse_include_directive(line) else {
        return Ok(None);
    };

    let include_path = Path::new(include_path);
    if include_path.is_absolute() {
        return Err(format!("Absolute include path is not allowed: {}", line).into());
    }

    let include_file = base_dir.join(include_path);
    let canonical_include_file = include_file.canonicalize()?;
    if !canonical_include_file.starts_with(canonical_base_dir) {
        return Err(format!(
            "Include path escapes content directory: {}",
            include_path.display()
        )
        .into());
    }

    Ok(Some(std::fs::read_to_string(canonical_include_file)?))
}

fn is_fence_line(line: &str) -> bool {
    let line = line.trim_start();
    line.starts_with("```") || line.starts_with("~~~")
}

fn parse_include_directive(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    let rest = trimmed.strip_prefix("#include")?.trim();
    let rest = rest.strip_prefix('"')?;
    let end_quote = rest.find('"')?;
    let (include_path, trailing) = rest.split_at(end_quote);
    if trailing[1..].trim().is_empty() {
        Some(include_path)
    } else {
        None
    }
}

fn load_bare_page(path: &Path, config: &Config) -> Result<Content, Box<dyn Error>> {
    let mut metadata = bare_page_metadata(path, config)?;
    let body = load_bare_page_body(path, &mut metadata)?;

    Ok(Content::Page { metadata, body })
}

fn bare_page_metadata(path: &Path, config: &Config) -> Result<ContentMetadata, Box<dyn Error>> {
    Ok(ContentMetadata {
        kind: ContentKind::Page,
        output_path: content_output_path(path, config)?,
        url: content_url(path, config)?,
        ..Default::default()
    })
}

fn load_bare_page_body(
    path: &Path,
    metadata: &mut ContentMetadata,
) -> Result<FormattedText, Box<dyn Error>> {
    match path.extension().and_then(|s| s.to_str()) {
        Some("md") => {
            let text = load_markdown_with_includes(path)?;
            if let Some(title) = first_markdown_heading(&text) {
                metadata.title = title;
            }
            Ok(FormattedText::Markdown(text))
        }
        Some("tex") => {
            let text = std::fs::read_to_string(path)?;
            Ok(FormattedText::Latex(text))
        }
        Some("html") => {
            let text = std::fs::read_to_string(path)?;
            Ok(FormattedText::Html(text))
        }
        extension => Err(format!("Unsupported file type: {:?}", extension).into()),
    }
}

fn first_markdown_heading(markdown: &str) -> Option<String> {
    let first_line = markdown.lines().next()?;
    if first_line.starts_with("# ") || first_line.starts_with("## ") {
        Some(
            first_line
                .replace("## ", "")
                .replace("# ", "")
                .trim()
                .to_string(),
        )
    } else {
        None
    }
}

fn load_single_content_file<F>(
    base_path: &Path,
    metadata: ContentMetadata,
    file_basename: &str,
    constructor: F,
) -> Result<Content, Box<dyn Error>>
where
    F: FnOnce(ContentMetadata, FormattedText) -> Content,
{
    let content = load_named_content_file(base_path, file_basename)?;
    Ok(constructor(metadata, content))
}

fn load_named_content_file(
    base_path: &Path,
    file_basename: &str,
) -> Result<FormattedText, Box<dyn Error>> {
    let md_file = base_path.join(format!("{}.md", file_basename));
    let tex_file = base_path.join(format!("{}.tex", file_basename));
    let html_file = base_path.join(format!("{}.html", file_basename));

    if md_file.exists() {
        let text = load_markdown_with_includes(&md_file)?;
        Ok(FormattedText::Markdown(text))
    } else if tex_file.exists() {
        let text = std::fs::read_to_string(tex_file)?;
        Ok(FormattedText::Latex(text))
    } else if html_file.exists() {
        let text = std::fs::read_to_string(html_file)?;
        Ok(FormattedText::Html(text))
    } else {
        Err(format!("No {} file found", file_basename).into())
    }
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
    fn test_markdown_include_ignored_inside_code_fence() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();
        let md_path = temp_path.join("body.md");

        fs::write(temp_path.join("part.md"), "Included text")?;
        fs::write(
            &md_path,
            "```markdown\n#include \"part.md\"\n```\n\n#include \"part.md\"",
        )?;

        let output = load_markdown_with_includes(&md_path)?;

        assert!(output.contains("```markdown\n#include \"part.md\"\n```"));
        assert!(output.ends_with("Included text"));

        Ok(())
    }

    #[test]
    fn test_markdown_include_rejects_parent_traversal() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let content_dir = temp_dir.path().join("content");
        fs::create_dir_all(&content_dir)?;
        fs::write(temp_dir.path().join("secret.md"), "Secret")?;
        let md_path = content_dir.join("body.md");
        fs::write(&md_path, "#include \"../secret.md\"")?;

        let err = load_markdown_with_includes(&md_path)
            .expect_err("include traversal should be rejected")
            .to_string();

        assert!(err.contains("escapes content directory"));

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
