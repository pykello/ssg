use clap::{Arg, Command};
use ssg::utils::{content::Content, images::ImageProcessor, render::Renderer, *};
use std::{
    fs,
    path::{Path, PathBuf},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up command-line argument parsing with clap
    let matches = Command::new("riazi_cafe_generator")
        .version("1.0")
        .author("Riazi Cafe Team")
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

/// Compute the output file path based on the path, build directory and content type
fn compute_output_path(
    path: &Path,
    build_dir: &Path,
    content_dir: &PathBuf,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Determine the relative path based on content_dir if provided
    let rel_path = if let Ok(rel) = path.strip_prefix(content_dir) {
        rel.to_path_buf()
    } else {
        // If path is not under content_dir, use the path directory name
        PathBuf::from(
            path.file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("unknown")),
        )
    };

    // Create output file path that preserves directory structure
    let mut output_file_path = build_dir.join(rel_path);

    // If output_file_path is a directory, append ".html"
    // Otherwise, replace or add ".html" extension
    if output_file_path.extension().is_none() {
        output_file_path.set_extension("html");
    } else {
        let file_stem = output_file_path
            .file_stem()
            .unwrap_or_else(|| std::ffi::OsStr::new("unknown"));
        output_file_path.set_file_name(format!("{}.html", file_stem.to_string_lossy()));
    }

    Ok(output_file_path)
}
