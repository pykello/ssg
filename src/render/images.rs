use regex::{Captures, Regex};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Finds all images in the given root directory.
/// Returns a vector of paths relative to the root.
fn find_images(root: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let allowed_extensions = ["jpg", "jpeg", "png", "gif", "bmp", "tiff", "svg"];

    let mut images = Vec::new();

    for entry in WalkDir::new(root) {
        let entry = entry?;
        if entry.file_type().is_file() {
            if let Some(ext) = entry
                .path()
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s.to_lowercase())
            {
                if allowed_extensions.contains(&ext.as_str()) {
                    // Get the path relative to the root.
                    let relative_path = entry.path().strip_prefix(root)?.to_path_buf();
                    images.push(relative_path);
                }
            }
        }
    }

    Ok(images)
}

// Modifies HTML content by prefixing image URLs with a root URL if they match any of the provided image paths.
fn prefix_image_urls(html: &str, image_paths: &[PathBuf], root_url: &str) -> String {
    // Create a set of normalized paths for efficient lookup
    let normalized_paths: Vec<String> = image_paths
        .iter()
        .map(|path| normalize_path(path))
        .collect();

    // Regular expressions for HTML img tags and CSS url() references
    let img_regex = Regex::new(r#"<img\s+[^>]*src=["']([^"']+)["'][^>]*>"#).unwrap();
    let css_url_regex = Regex::new(r#"url\(['"]?([^'"\)]+)['"]?\)"#).unwrap();

    // Process img tags
    let html = img_regex.replace_all(html, |caps: &Captures| {
        let full_match = &caps[0];
        let src = &caps[1];

        if should_prefix(src, &normalized_paths) {
            let new_src = format!("{}{}", root_url, src);
            full_match.replace(src, &new_src)
        } else {
            full_match.to_string()
        }
    });

    // Process CSS url() references
    let html = css_url_regex.replace_all(&html, |caps: &Captures| {
        let full_match = &caps[0];
        let url_path = &caps[1];

        if should_prefix(url_path, &normalized_paths) {
            format!("url('{}{}')", root_url, url_path)
        } else {
            full_match.to_string()
        }
    });

    html.to_string()
}

// Determines if a path in HTML should be prefixed with the root URL.
fn should_prefix(path: &str, normalized_paths: &[String]) -> bool {
    // Skip paths that are already absolute URLs or data URLs
    if path.starts_with("http://")
        || path.starts_with("https://")
        || path.starts_with("data:")
        || path.starts_with("/")
    {
        return false;
    }

    // Normalize the path for comparison
    let normalized = normalize_path(Path::new(path));

    // Check if this path (or a parent directory) is in our list
    normalized_paths
        .iter()
        .any(|p| normalized == *p || normalized.starts_with(p) || p.starts_with(&normalized))
}

/// Normalizes a path for comparison by removing extra separators and converting to string.
fn normalize_path<P: AsRef<Path>>(path: P) -> String {
    let path_str = path.as_ref().to_string_lossy().to_string();

    // Convert backslashes to forward slashes for consistent comparison
    path_str.replace('\\', "/")
}

/// A class that manages images within a content directory and handles
/// copying them to the build directory and updating HTML content.
pub struct ImageProcessor {
    path: PathBuf,
    content_dir: PathBuf,
    build_dir: PathBuf,
    images: Vec<PathBuf>,
    url_prefix: Option<String>,
}

impl ImageProcessor {
    pub fn new(
        path: PathBuf,
        content_dir: PathBuf,
        build_dir: PathBuf,
    ) -> Result<Self, Box<dyn Error>> {
        let images = find_images(&path)?;

        Ok(Self {
            path,
            content_dir,
            build_dir,
            images,
            url_prefix: None,
        })
    }

    /// Checks if any images were found in the content directory
    pub fn has_images(&self) -> bool {
        !self.images.is_empty()
    }

    /// Gets the number of images found
    pub fn image_count(&self) -> usize {
        self.images.len()
    }

    /// Copies all images to the build directory and sets up the URL prefix
    pub fn copy_images_to_build_dir(&mut self) -> Result<(), Box<dyn Error>> {
        // Skip if no images found
        if self.images.is_empty() {
            return Ok(());
        }

        // Create the relative path for this content
        let rel_path = self.path.strip_prefix(&self.content_dir)?;
        let static_assets_dir = self.build_dir.join("static/assets").join(rel_path);

        // Ensure the target directory exists
        fs::create_dir_all(&static_assets_dir)?;

        // Copy each image to the build directory
        for image in &self.images {
            let source_path = self.path.join(image);
            let target_path = static_assets_dir.join(image);

            // Create directory if it doesn't exist
            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent)?;
            }

            // Copy the file
            fs::copy(&source_path, &target_path)?;
        }

        // Set the URL prefix for this content
        self.url_prefix = Some(format!("/static/assets/{}/", rel_path.display()));

        Ok(())
    }

    /// Updates HTML content by replacing image references with the prefixed URLs
    pub fn update_html_with_image_urls(&self, html: &str) -> String {
        if let Some(ref prefix) = self.url_prefix {
            prefix_image_urls(html, &self.images, prefix)
        } else {
            // If no URL prefix set (no images copied), return the original HTML
            html.to_string()
        }
    }

    /// Process multiple HTML strings and return updated versions
    pub fn update_multiple_html(&self, html_contents: &[String]) -> Vec<String> {
        html_contents
            .iter()
            .map(|html| self.update_html_with_image_urls(html))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use tempfile::tempdir;

    #[test]
    fn test_find_images_in_test_assets() {
        // Define the root directory for the test assets.
        let root = Path::new("src/test_assets/problems/p1");

        // Call the function to find images.
        let mut images = find_images(root).expect("Failed to find images");
        images.sort();

        assert_eq!(
            images,
            vec![
                PathBuf::from("figs/blue.png"),
                PathBuf::from("figs/green.png"),
                PathBuf::from("figs/red.png"),
            ]
        );
    }

    #[test]
    fn test_prefix_image_urls_basic() {
        let html = r#"<img src="figs/image.png" alt="An image">"#;
        let image_paths = vec![PathBuf::from("figs/image.png")];
        let root_url = "https://example.com/static/";

        let result = prefix_image_urls(html, &image_paths, root_url);
        assert_eq!(
            result,
            r#"<img src="https://example.com/static/figs/image.png" alt="An image">"#
        );
    }

    #[test]
    fn test_prefix_image_urls_multiple() {
        let html = r#"
            <div>
                <img src="figs/image1.png" alt="Image 1">
                <img src="figs/subfolder/image2.jpg" alt="Image 2">
                <img src="https://example.org/image.png" alt="External image">
                <img src="data:image/png;base64,abc123=" alt="Data URL">
                <img src="/absolute/path/image.png" alt="Absolute path">
            </div>
        "#;

        let image_paths = vec![
            PathBuf::from("figs/image1.png"),
            PathBuf::from("figs/subfolder/image2.jpg"),
        ];
        let root_url = "https://example.com/static/";

        let result = prefix_image_urls(html, &image_paths, root_url);
        assert!(result.contains(r#"src="https://example.com/static/figs/image1.png""#));
        assert!(result.contains(r#"src="https://example.com/static/figs/subfolder/image2.jpg""#));
        assert!(result.contains(r#"src="https://example.org/image.png""#));
        assert!(result.contains(r#"src="data:image/png;base64,abc123=""#));
        assert!(result.contains(r#"src="/absolute/path/image.png""#));
    }

    #[test]
    fn test_prefix_css_urls() {
        let html = r#"
            <style>
                .bg-image { background-image: url('figs/bg.jpg'); }
                .another-bg { background: url("figs/pattern.png") repeat; }
                .external-bg { background-image: url('https://example.org/bg.jpg'); }
            </style>
        "#;

        let image_paths = vec![
            PathBuf::from("figs/bg.jpg"),
            PathBuf::from("figs/pattern.png"),
        ];
        let root_url = "https://example.com/static/";

        let result = prefix_image_urls(html, &image_paths, root_url);
        assert!(result.contains("url('https://example.com/static/figs/bg.jpg')"));
        assert!(result.contains("url('https://example.com/static/figs/pattern.png')"));
        assert!(result.contains("url('https://example.org/bg.jpg')"));
    }

    #[test]
    fn test_image_processor() {
        // Create a temporary directory to simulate build dir
        let temp_dir = tempdir().unwrap();
        let build_dir = temp_dir.path().to_path_buf();

        // Path to test assets
        let content_dir = PathBuf::from("src");
        let path = PathBuf::from("src/test_assets/problems/p1");

        // Create an image processor
        let mut processor =
            ImageProcessor::new(path.clone(), content_dir.clone(), build_dir.clone()).unwrap();

        // Check if images were found
        assert!(processor.has_images());
        assert!(processor.image_count() > 0);

        // Copy images to build dir
        processor.copy_images_to_build_dir().unwrap();

        // Verify images were copied
        let rel_path = path.strip_prefix(&content_dir).unwrap();
        let static_assets_dir = build_dir.join("static/assets").join(rel_path);
        assert!(static_assets_dir.join("figs/blue.png").exists());

        // Test HTML updating
        let html = r#"<img src="figs/blue.png" alt="Blue"><p>Some text</p>"#;
        let updated = processor.update_html_with_image_urls(html);
        assert!(updated.contains("/static/assets/test_assets/problems/p1/figs/blue.png"));
    }
}
