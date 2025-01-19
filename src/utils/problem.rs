use super::content::Content;
use super::content_metadata::*;
use crate::utils::formatted_text::FormattedText;
use regex::Regex;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

/// Loads a Problem from the given directory path.
pub fn load_problem(
    base_path: &Path,
    metadata: ContentMetadata,
) -> Result<Content, Box<dyn Error>> {
    // Load problem file: try problem.tex then problem.md
    let problem = {
        let problem_tex = base_path.join("problem.tex");
        let problem_md = base_path.join("problem.md");
        if problem_tex.exists() {
            load_formatted_file(&problem_tex)?
        } else if problem_md.exists() {
            load_formatted_file(&problem_md)?
        } else {
            return Err("Problem file not found".into());
        }
    };

    // Load solutions and hints (files may have a numeric suffix).
    let solutions = load_multiple_files(base_path, "solution")?;
    let hints = load_multiple_files(base_path, "hint")?;

    Ok(Content::Problem(metadata, problem, solutions, hints))
}

/// Reads a file and wraps its contents as FormattedText based on the extension.
fn load_formatted_file(file_path: &Path) -> Result<FormattedText, Box<dyn Error>> {
    let content = fs::read_to_string(file_path)?;
    match file_path.extension().and_then(|s| s.to_str()) {
        Some("tex") => Ok(FormattedText::Latex(content)),
        Some("md") => Ok(FormattedText::Markdown(content)),
        _ => Err("Unsupported file extension".into()),
    }
}

/// Loads multiple files (e.g., for solutions or hints) that match the pattern:
/// basename[.number].(tex|md)
fn load_multiple_files(
    base_path: &Path,
    basename: &str,
) -> Result<Vec<FormattedText>, Box<dyn Error>> {
    let mut items: Vec<(usize, PathBuf)> = Vec::new();

    // Regex pattern: ^basename(?:\.(\d+))?\.(tex|md)$
    let pattern = format!(r"^{}(?:\.(\d+))?\.(tex|md)$", regex::escape(basename));
    let re = Regex::new(&pattern)?;

    // Iterate through directory entries
    for entry in fs::read_dir(base_path)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();
        if let Some(caps) = re.captures(&file_name_str) {
            // If a number is provided, use it for sorting; default to 0.
            let order = caps
                .get(1)
                .map_or(0, |m| m.as_str().parse::<usize>().unwrap_or(0));
            items.push((order, entry.path()));
        }
    }

    // Sort items by their order and then by file name to have a deterministic order.
    items.sort_by_key(|(order, path)| (*order, path.clone()));

    // Load each file as FormattedText.
    let mut result = Vec::new();
    for (_, file_path) in items {
        result.push(load_formatted_file(&file_path)?);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_load_problem_p1() {
        let path = Path::new("src/test_assets/problems/p1");

        // First load metadata
        let metadata = ContentMetadata::load(path).expect("Failed to load metadata");

        // Then load the full problem using that metadata
        let content = load_problem(path, metadata).expect("Failed to load problem");

        // Check the content type
        assert_eq!(content.content_type(), "problem");

        // Check problem details
        if let Content::Problem(meta, problem, solutions, hints) = content {
            // Verify metadata
            assert_eq!(meta.title, "Sample Problem");
            assert_eq!(meta.id, Some("sample-problem-001".to_string()));

            // Verify problem content
            let problem_html = problem
                .to_html()
                .expect("Failed to convert problem to HTML");
            assert_eq!(problem_html, "<p>Problem Body</p>\n");

            // Verify solutions
            assert_eq!(solutions.len(), 1);
            let solution_html = solutions[0]
                .to_html()
                .expect("Failed to convert solution to HTML");
            assert_eq!(solution_html, "<p>Some Solution</p>\n");

            // Verify hints
            assert_eq!(hints.len(), 1);
            let hint_html = hints[0].to_html().expect("Failed to convert hint to HTML");
            assert_eq!(hint_html, "<p>Hint Body</p>\n");
        } else {
            panic!("Expected Content::Problem, got something else");
        }
    }

    #[test]
    fn test_load_problem_missing_problem_file() {
        // Create a temporary directory with metadata but no problem file
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let temp_path = temp_dir.path();

        // Create metadata.yaml
        let metadata_content = r#"
title: "Test Problem"
id: "test-123"
type: "problem"
"#;

        std::fs::write(temp_path.join("metadata.yaml"), metadata_content)
            .expect("Failed to write metadata file");

        // Load metadata
        let metadata = ContentMetadata::load(temp_path).expect("Failed to load metadata");

        // Try to load problem - should fail because there's no problem file
        let result = load_problem(temp_path, metadata);
        assert!(result.is_err());

        // Verify the error message mentions the missing problem file
        let err = result.err().unwrap();
        let err_msg = err.to_string();
        assert!(err_msg.contains("Problem file not found"));
    }

    #[test]
    fn test_load_multiple_files() {
        // Create a temporary directory with multiple solution files
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let temp_path = temp_dir.path();

        // Create solution files with different numbering to test sorting
        std::fs::write(temp_path.join("solution.2.md"), "Solution 2")
            .expect("Failed to write solution.2.md");
        std::fs::write(temp_path.join("solution.10.md"), "Solution 10")
            .expect("Failed to write solution.10.md");
        std::fs::write(temp_path.join("solution.1.md"), "Solution 1")
            .expect("Failed to write solution.1.md");
        std::fs::write(temp_path.join("solution.md"), "Default Solution")
            .expect("Failed to write solution.md");

        // Also create some non-solution files
        std::fs::write(temp_path.join("not-a-solution.md"), "Not a solution")
            .expect("Failed to write not-a-solution.md");

        // Load solution files
        let solutions =
            load_multiple_files(temp_path, "solution").expect("Failed to load solutions");

        // Check we have the right number of solutions
        assert_eq!(solutions.len(), 4);

        // Check they're in the right order
        // Default (no number) should come first (index 0)
        if let FormattedText::Markdown(content) = &solutions[0] {
            assert_eq!(content, "Default Solution");
        } else {
            panic!("Expected Markdown");
        }

        // Then solution.1.md (index 1)
        if let FormattedText::Markdown(content) = &solutions[1] {
            assert_eq!(content, "Solution 1");
        } else {
            panic!("Expected Markdown");
        }

        // Then solution.2.md (index 2)
        if let FormattedText::Markdown(content) = &solutions[2] {
            assert_eq!(content, "Solution 2");
        } else {
            panic!("Expected Markdown");
        }

        // Then solution.10.md (index 3)
        if let FormattedText::Markdown(content) = &solutions[3] {
            assert_eq!(content, "Solution 10");
        } else {
            panic!("Expected Markdown");
        }
    }
}
