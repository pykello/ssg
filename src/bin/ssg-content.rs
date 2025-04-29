use clap::{Arg, Command};
use ssg::{config, content::*, render::*};
use std::{fs, path::PathBuf};

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

    fs::create_dir_all(&config.build_dir)?;

    let content = Content::load(&path, &config)
        .expect(&format!("Failed to load content from {}", path.display()));

    let renderer = Renderer::new(&config);

    let mut html = content.render_html(&renderer)?;
    let mut image_processor = ImageProcessor::new(
        path.clone(),
        config.content_dir.clone(),
        config.build_dir.clone(),
    )?;

    if image_processor.has_images() {
        image_processor.copy_images_to_build_dir()?;
        html = image_processor.update_html_with_image_urls(&html);
    }

    let output_file_path = content.metadata().output_path.clone();

    if let Some(parent) = output_file_path.parent() {
        fs::create_dir_all(parent)?;
    }

    println!("Generating {}", output_file_path.display());
    fs::write(output_file_path, html)?;

    Ok(())
}
