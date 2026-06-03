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

// These crates are used by the `ssg` library that this binary depends on.
// Declaring them here silences `unused_crate_dependencies` when building
// just the ssg-list binary target.
use chrono as _;
use comrak as _;
use regex as _;
use tera as _;

fn default_template() -> String {
    "list.html".to_string()
}

fn absolute_path(path: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct IndexConfig {
    title: Option<String>,
    #[serde(rename = "content-type")]
    content_type: ContentKind,
    path: Option<String>,
    #[serde(default = "default_template")]
    template: String,
}

struct CliArgs {
    index_yaml_path: PathBuf,
    config_path: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run(parse_args()?)
}

fn parse_args() -> Result<CliArgs, Box<dyn std::error::Error>> {
    let matches = cli_command().get_matches();

    let index_yaml_path = matches
        .get_one::<String>("path")
        .map(PathBuf::from)
        .ok_or("Missing required index.yaml path argument")?;
    let config_path = matches
        .get_one::<PathBuf>("config")
        .cloned()
        .ok_or("Missing required --config argument")?;

    Ok(CliArgs {
        index_yaml_path,
        config_path,
    })
}

fn cli_command() -> Command {
    Command::new("ssg-list")
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
}

fn run(args: CliArgs) -> Result<(), Box<dyn std::error::Error>> {
    let config = config::Config::load(&args.config_path)?;

    fs::create_dir_all(&config.build_dir)?;

    println!(
        "Loading index_config from: {}",
        args.index_yaml_path.display()
    );
    println!("Build directory: {}", config.build_dir.display());

    let index_config = load_index_config(&args.index_yaml_path)?;
    let renderer = Renderer::new(&config)?;
    let output_base_dir = output_base_dir(&args.index_yaml_path, &config)?;

    println!("Base content path: {}", output_base_dir.display());

    let search_path = search_path(&args.index_yaml_path, &index_config)?;
    let mut content_items = find_content_files(&search_path, index_config.content_type, &config)?;
    sort_content_items(&mut content_items);

    println!("Found {} content items", content_items.len());

    fs::create_dir_all(&output_base_dir)?;
    let html = render_list(&renderer, &index_config, &content_items)?;
    fs::write(output_base_dir.join("index.html"), html)?;

    println!("List generation completed successfully!");
    Ok(())
}

fn load_index_config(path: &Path) -> Result<IndexConfig, Box<dyn std::error::Error>> {
    let config_content = fs::read_to_string(path)?;
    Ok(serde_yaml::from_str(&config_content)?)
}

fn output_base_dir(
    index_yaml_path: &Path,
    config: &config::Config,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let parent_dir = index_parent_dir(index_yaml_path)?;
    let absolute_parent_dir = absolute_path(parent_dir)?;
    let absolute_content_dir = absolute_path(&config.content_dir)?;

    if let Ok(rel) = absolute_parent_dir.strip_prefix(&absolute_content_dir) {
        Ok(config.build_dir.join(rel))
    } else {
        Ok(config.build_dir.join(parent_dir))
    }
}

fn index_parent_dir(index_yaml_path: &Path) -> Result<&Path, Box<dyn std::error::Error>> {
    index_yaml_path.parent().ok_or_else(|| {
        format!(
            "Index file has no parent directory: {}",
            index_yaml_path.display()
        )
        .into()
    })
}

fn search_path(
    index_yaml_path: &Path,
    index_config: &IndexConfig,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let parent_dir = index_parent_dir(index_yaml_path)?;
    Ok(index_config
        .path
        .as_ref()
        .map_or_else(|| parent_dir.to_owned(), |path| parent_dir.join(path)))
}

fn sort_content_items(content_items: &mut [ContentMetadata]) {
    content_items.sort_by(|a, b| match (&a.timestamp, &b.timestamp) {
        (Some(a_date), Some(b_date)) => b_date.cmp(a_date),
        _ => a.title.cmp(&b.title),
    });
}

fn render_list(
    renderer: &Renderer,
    index_config: &IndexConfig,
    content_items: &[ContentMetadata],
) -> Result<String, Box<dyn std::error::Error>> {
    let mut context = HashMap::new();
    if let Some(title) = &index_config.title {
        context.insert("title".to_string(), Value::String(title.clone()));
    }

    context.insert(
        "content_items".to_string(),
        Value::Array(serializable_content_items(content_items)),
    );

    renderer.render(&index_config.template, context)
}

fn serializable_content_items(content_items: &[ContentMetadata]) -> Vec<Value> {
    content_items
        .iter()
        .map(|item| serde_json::to_value(item).unwrap())
        .collect()
}

fn find_content_files(
    base_path: &Path,
    content_type: ContentKind,
    config: &config::Config,
) -> Result<Vec<ContentMetadata>, Box<dyn std::error::Error>> {
    let mut content_items = Vec::new();

    for entry in WalkDir::new(base_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        if path.is_dir() {
            continue;
        }

        if path.file_name() == Some("metadata.yaml".as_ref()) {
            load_directory_metadata(path, content_type, config, &mut content_items);
            continue;
        }

        if content_type == ContentKind::Page && is_bare_content_file(path) {
            if has_directory_metadata(path) {
                continue;
            }
            load_bare_page_metadata(path, config, &mut content_items);
        }
    }

    Ok(content_items)
}

fn load_directory_metadata(
    metadata_path: &Path,
    content_type: ContentKind,
    config: &config::Config,
    content_items: &mut Vec<ContentMetadata>,
) {
    let Some(dir) = metadata_path.parent() else {
        println!(
            "Warning: Failed to load metadata from {}: metadata.yaml has no parent directory",
            metadata_path.display()
        );
        return;
    };

    match ContentMetadata::load(dir, config) {
        Ok(metadata) => {
            if metadata.kind == content_type {
                content_items.push(metadata);
            }
        }
        Err(err) => {
            println!(
                "Warning: Failed to load metadata from {}: {}",
                metadata_path.display(),
                err
            );
        }
    }
}

fn has_directory_metadata(path: &Path) -> bool {
    path.parent()
        .map(|parent| parent.join("metadata.yaml").exists())
        .unwrap_or(false)
}

fn load_bare_page_metadata(
    path: &Path,
    config: &config::Config,
    content_items: &mut Vec<ContentMetadata>,
) {
    match Content::load(path, config) {
        Ok(Content::Page { metadata, .. }) => content_items.push(metadata),
        Ok(_) => {}
        Err(err) => {
            println!(
                "Warning: Failed to load bare page from {}: {}",
                path.display(),
                err
            );
        }
    }
}

fn is_bare_content_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("md" | "html" | "tex")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn find_content_files_includes_bare_pages() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let content_dir = temp_dir.path().join("content");
        let build_dir = temp_dir.path().join("build");
        fs::create_dir_all(&content_dir)?;
        fs::write(content_dir.join("about.md"), "# About\n\nBody")?;

        let config = config::Config {
            content_dir: content_dir.clone(),
            build_dir,
            ..Default::default()
        };

        let items = find_content_files(&content_dir, ContentKind::Page, &config)?;

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "About");
        assert!(items[0].url.ends_with("/about.html"));

        Ok(())
    }

    #[test]
    fn find_content_files_skips_bare_body_in_metadata_directory(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let content_dir = temp_dir.path().join("content");
        let build_dir = temp_dir.path().join("build");
        let page_dir = content_dir.join("page");
        fs::create_dir_all(&page_dir)?;
        fs::write(page_dir.join("metadata.yaml"), "title: Page\ntype: page\n")?;
        fs::write(page_dir.join("body.md"), "# Body\n")?;

        let config = config::Config {
            content_dir: content_dir.clone(),
            build_dir,
            ..Default::default()
        };

        let items = find_content_files(&content_dir, ContentKind::Page, &config)?;

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Page");

        Ok(())
    }
}
