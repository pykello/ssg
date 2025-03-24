use clap::{Arg, Command};
use ssg::{config, content::*, render::*};
use std::{
    fs,
    path::{Path, PathBuf},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up command-line argument parsing with clap
    let matches = Command::new("ssg-content")
        .version("1.0")
        .author("Hadi Moshayedi")
        .about("Generates HTML files from definitions")
        .arg(
            Arg::new("path")
                .help("Path to the directory to process")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("config")
                .long("config")
                .help("Path to the configuration file")
                .required(true)
                .value_name("FILE")
                .value_parser(clap::value_parser!(PathBuf)),
        )
        .get_matches();

    // Extract values from arguments
    let path = PathBuf::from(matches.get_one::<String>("path").unwrap());
    let config_path = matches.get_one::<PathBuf>("config").unwrap().clone();
    let config = config::Config::load(&config_path)?;

    // Create build directory if it doesn't exist
    fs::create_dir_all(&config.build_dir)?;

    // Load the content using the generic content loader
    let content =
        Content::load(&path).expect(&format!("Failed to load content from {}", path.display()));

    let language = content
        .metadata()
        .language
        .clone()
        .unwrap_or_else(|| "en".to_string());
    let renderer = Renderer::new(&config, language);

    let mut html = content.render_html(&renderer)?;
    let mut image_processor = ImageProcessor::new(
        path.clone(),
        config.content_dir.clone(),
        config.build_dir.clone(),
    )?;

    if image_processor.has_images() {
        // Copy all images to the build directory and set up URL prefixing
        image_processor.copy_images_to_build_dir()?;

        // Update image references in the HTML
        html = image_processor.update_html_with_image_urls(&html);
    }

    // Compute the output file path
    let output_file_path = compute_output_path(&path, &config.build_dir, &config.content_dir)?;

    // Create parent directories if needed
    if let Some(parent) = output_file_path.parent() {
        fs::create_dir_all(parent)?;
    }

    println!("Generating {}", output_file_path.display());
    fs::write(output_file_path, html)?;

    Ok(())
}

fn compute_output_path(
    path: &Path,
    build_dir: &Path,
    content_dir: &PathBuf,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let cwd = std::env::current_dir()?;
    let path = cwd.join(path);
    let content_dir = cwd.join(content_dir);
    let rel_path = path.strip_prefix(content_dir)?;

    // Create output file path that preserves directory structure
    let mut output_file_path = build_dir.join(rel_path);
    output_file_path.set_extension("html");

    Ok(output_file_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_output_path_abs() -> Result<(), Box<dyn std::error::Error>> {
        let content_dir = PathBuf::from("/content");
        let build_dir = Path::new("/build");

        let path = Path::new("/content/page1.md");
        let output_path = compute_output_path(path, build_dir, &content_dir)?;
        assert_eq!(output_path, Path::new("/build/page1.html"));

        Ok(())
    }

    #[test]
    fn test_compute_output_path_rel() -> Result<(), Box<dyn std::error::Error>> {
        let content_dir = PathBuf::from("content");
        let build_dir = Path::new("build");

        let path = Path::new("content/subdir/page1.md");
        let output_path = compute_output_path(path, build_dir, &content_dir)?;
        assert_eq!(output_path, Path::new("build/subdir/page1.html"));

        Ok(())
    }
}
