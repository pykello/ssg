use regex::{Captures, Regex};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use walkdir::{DirEntry, WalkDir};

static IMG_REGEX: OnceLock<Regex> = OnceLock::new();
static CSS_URL_REGEX: OnceLock<Regex> = OnceLock::new();

const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "bmp", "tiff", "svg"];
const STATIC_ASSETS_DIR: &str = "static/assets";

fn absolute_path(path: PathBuf) -> Result<PathBuf, Box<dyn Error>> {
    if path.is_absolute() {
        Ok(path)
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}

fn img_regex() -> &'static Regex {
    IMG_REGEX.get_or_init(|| {
        Regex::new(r#"<img\s+[^>]*src=["']([^"']+)["'][^>]*>"#).expect("valid img regex")
    })
}

fn css_url_regex() -> &'static Regex {
    CSS_URL_REGEX
        .get_or_init(|| Regex::new(r#"url\(['"]?([^'"\)]+)['"]?\)"#).expect("valid css url regex"))
}

fn find_images(root: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut images = Vec::new();

    for entry in WalkDir::new(root) {
        let entry = entry?;
        if is_image_file(&entry) {
            images.push(entry.path().strip_prefix(root)?.to_path_buf());
        }
    }

    Ok(images)
}

fn is_image_file(entry: &DirEntry) -> bool {
    entry.file_type().is_file()
        && entry
            .path()
            .extension()
            .and_then(|s| s.to_str())
            .map(|ext| {
                IMAGE_EXTENSIONS
                    .iter()
                    .any(|allowed| ext.eq_ignore_ascii_case(allowed))
            })
            .unwrap_or(false)
}

fn prefix_image_urls(html: &str, image_paths: &[PathBuf], root_url: &str) -> String {
    let normalized_paths: Vec<String> = image_paths.iter().map(normalize_path).collect();
    let html = prefix_img_tags(html, &normalized_paths, root_url);
    let html = prefix_css_urls(&html, &normalized_paths, root_url);
    html.to_string()
}

fn prefix_img_tags<'a>(
    html: &'a str,
    normalized_paths: &'a [String],
    root_url: &'a str,
) -> std::borrow::Cow<'a, str> {
    img_regex().replace_all(html, |caps: &Captures| {
        let full_match = &caps[0];
        let src = &caps[1];

        if should_prefix(src, normalized_paths) {
            let new_src = format!("{}{}", root_url, src);
            full_match.replace(src, &new_src)
        } else {
            full_match.to_string()
        }
    })
}

fn prefix_css_urls<'a>(
    html: &'a str,
    normalized_paths: &'a [String],
    root_url: &'a str,
) -> std::borrow::Cow<'a, str> {
    css_url_regex().replace_all(html, |caps: &Captures| {
        let full_match = &caps[0];
        let url_path = &caps[1];

        if should_prefix(url_path, normalized_paths) {
            format!("url('{}{}')", root_url, url_path)
        } else {
            full_match.to_string()
        }
    })
}

fn should_prefix(path: &str, normalized_paths: &[String]) -> bool {
    if is_external_or_rooted_path(path) {
        return false;
    }

    let normalized = normalize_path(Path::new(path));
    normalized_paths
        .iter()
        .any(|p| normalized == *p || normalized.starts_with(p) || p.starts_with(&normalized))
}

fn is_external_or_rooted_path(path: &str) -> bool {
    path.starts_with("http://")
        || path.starts_with("https://")
        || path.starts_with("data:")
        || path.starts_with("/")
}

fn normalize_path<P: AsRef<Path>>(path: P) -> String {
    let path_str = path.as_ref().to_string_lossy().to_string();
    path_str.replace('\\', "/")
}

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
        let path = absolute_path(path)?;
        let content_dir = absolute_path(content_dir)?;
        let path = content_root(path)?;

        let images = find_images(&path)?;

        Ok(Self {
            path,
            content_dir,
            build_dir,
            images,
            url_prefix: None,
        })
    }

    pub fn has_images(&self) -> bool {
        !self.images.is_empty()
    }

    pub fn image_count(&self) -> usize {
        self.images.len()
    }

    pub fn copy_images_to_build_dir(&mut self) -> Result<(), Box<dyn Error>> {
        if self.images.is_empty() {
            return Ok(());
        }

        let rel_path = self.path.strip_prefix(&self.content_dir)?;
        let static_assets_dir = self.build_dir.join(STATIC_ASSETS_DIR).join(rel_path);

        fs::create_dir_all(&static_assets_dir)?;
        self.copy_images(&static_assets_dir)?;
        self.url_prefix = Some(format!("/{STATIC_ASSETS_DIR}/{}/", rel_path.display()));

        Ok(())
    }

    pub fn update_html_with_image_urls(&self, html: &str) -> String {
        if let Some(ref prefix) = self.url_prefix {
            prefix_image_urls(html, &self.images, prefix)
        } else {
            html.to_string()
        }
    }

    pub fn update_multiple_html(&self, html_contents: &[String]) -> Vec<String> {
        html_contents
            .iter()
            .map(|html| self.update_html_with_image_urls(html))
            .collect()
    }

    fn copy_images(&self, static_assets_dir: &Path) -> Result<(), Box<dyn Error>> {
        for image in &self.images {
            let source_path = self.path.join(image);
            let target_path = static_assets_dir.join(image);

            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::copy(source_path, target_path)?;
        }

        Ok(())
    }
}

fn content_root(path: PathBuf) -> Result<PathBuf, Box<dyn Error>> {
    if path.is_dir() {
        Ok(path)
    } else {
        path.parent()
            .ok_or("Provided path is not a directory or file with no parent")
            .map(Path::to_path_buf)
            .map_err(Into::into)
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

    #[test]
    fn test_image_processor_accepts_absolute_content_path() {
        let temp_dir = tempdir().unwrap();
        let build_dir = temp_dir.path().join("build");
        let cwd = std::env::current_dir().unwrap();
        let content_dir = cwd.join("src");
        let path = cwd.join("src/test_assets/problems/p1/problem.tex");

        let mut processor = ImageProcessor::new(path, content_dir, build_dir.clone()).unwrap();

        processor.copy_images_to_build_dir().unwrap();

        let copied = build_dir.join("static/assets/test_assets/problems/p1/figs/blue.png");
        assert!(copied.exists());

        let html = r#"<img src="figs/blue.png" alt="Blue">"#;
        let updated = processor.update_html_with_image_urls(html);
        assert!(updated.contains("/static/assets/test_assets/problems/p1/figs/blue.png"));
    }
}
