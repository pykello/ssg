use clap::{Arg, Command};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ssg::{config, content::*, render::*};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

#[derive(Debug, Deserialize, Serialize)]
struct IndexConfig {
    title: String,
    #[serde(rename = "content-type")]
    content_type: String,
    language: Option<String>,
    path: Option<String>,
    #[serde(rename = "items-per-page", default = "default_items_per_page")]
    items_per_page: usize,
}

fn default_items_per_page() -> usize {
    50
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up command-line argument parsing with clap
    let matches = Command::new("ssg-list")
        .version("1.0")
        .author("Hadi Moshayedi")
        .about("Generates paginated list pages from content")
        .arg(
            Arg::new("path")
                .help("Path to the index.yaml configuration file")
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
    let index_yaml_path = PathBuf::from(matches.get_one::<String>("path").unwrap());
    let config_path = matches.get_one::<PathBuf>("config").unwrap().clone();
    let config = config::Config::load(&config_path)?;

    // Create build directory if it doesn't exist
    fs::create_dir_all(&config.build_dir)?;

    println!("Loading index_config from: {}", index_yaml_path.display());
    println!("Build directory: {}", config.build_dir.display());

    // Load the index configuration
    let config_content = fs::read_to_string(&index_yaml_path)?;
    let index_config: IndexConfig = serde_yaml::from_str(&config_content)?;

    let language = index_config
        .language
        .clone()
        .unwrap_or_else(|| "en".to_string());
    let renderer = Renderer::new(&config, language);

    let parent_dir = index_yaml_path.parent().unwrap();
    let output_base_dir = if let Ok(rel) = parent_dir.strip_prefix(&config.content_dir) {
        config.build_dir.join(rel.to_path_buf())
    } else {
        config.build_dir.join(parent_dir.to_path_buf())
    };
    fs::create_dir_all(&output_base_dir)?;

    println!("Base content path: {}", output_base_dir.display());

    // Find all content files of the specified type recursively
    let mut content_items = find_content_files(&parent_dir, &index_config.content_type)?;

    // Sort content by date (newest first) or title if date is not available
    content_items.sort_by(|a, b| {
        let a_date = a.timestamp.clone();
        let b_date = b.timestamp.clone();

        match (a_date, b_date) {
            (Some(a_date), Some(b_date)) => b_date.cmp(&a_date),
            _ => a.title.cmp(&b.title),
        }
    });

    println!("Found {} content items", content_items.len());

    // Calculate number of pages needed
    let total_items = content_items.len();
    let items_per_page = index_config.items_per_page;
    let total_pages = (total_items + items_per_page - 1) / items_per_page;

    println!(
        "Creating {} pages with {} items per page",
        total_pages, items_per_page
    );

    fs::create_dir_all(&output_base_dir)?;

    // Generate the paginated index pages
    for page_num in 1..=total_pages {
        let start = (page_num - 1) * items_per_page;
        let end = std::cmp::min(start + items_per_page, total_items);
        let page_items = &content_items[start..end];

        let page_filename = if page_num == 1 {
            "index.html".to_string()
        } else {
            format!("index_p{}.html", page_num)
        };

        let output_path = output_base_dir.join(&page_filename);

        // Create context for rendering
        let mut context = HashMap::new();
        context.insert(
            "title".to_string(),
            Value::String(index_config.title.clone()),
        );

        // Convert content items to serializable format
        let serializable_items: Vec<Value> = page_items
            .iter()
            .map(|item| {
                let mut obj = serde_json::Map::new();
                obj.insert("title".to_string(), Value::String(item.title.clone()));
                obj.insert(
                    "url".to_string(),
                    Value::String(format!("{}.html", "url".to_string())),
                );
                Value::Object(obj)
            })
            .collect();

        context.insert(
            "content_items".to_string(),
            Value::Array(serializable_items),
        );
        context.insert("current_page".to_string(), Value::Number(page_num.into()));
        context.insert("total_pages".to_string(), Value::Number(total_pages.into()));
        context.insert("has_prev".to_string(), Value::Bool(page_num > 1));
        context.insert("has_next".to_string(), Value::Bool(page_num < total_pages));
        context.insert(
            "prev_page".to_string(),
            Value::Number((page_num - 1).into()),
        );
        context.insert(
            "next_page".to_string(),
            Value::Number((page_num + 1).into()),
        );

        // Use render function from render.rs
        let html = renderer.render("list.html", context)?;

        println!("Writing page {} to: {}", page_num, output_path.display());
        fs::write(output_path, html)?;
    }

    println!("List generation completed successfully!");
    Ok(())
}

// Function to find all content files of a specified type in a directory recursively
fn find_content_files(
    base_path: &Path,
    content_type: &str,
) -> Result<Vec<ContentMetadata>, Box<dyn std::error::Error>> {
    let mut content_items = Vec::new();

    for entry in WalkDir::new(base_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        // Skip directories and non-supported files
        if path.is_dir() || path.file_name() != Some("metadata.yaml".as_ref()) {
            continue;
        }

        // Try to load the content
        match ContentMetadata::load(path.parent().unwrap()) {
            Ok(metadata) => {
                if metadata.r#type == content_type {
                    content_items.push(metadata);
                }
            }
            Err(err) => {
                println!(
                    "Warning: Failed to load metadata from {}: {}",
                    path.display(),
                    err
                );
            }
        }
    }

    Ok(content_items)
}
