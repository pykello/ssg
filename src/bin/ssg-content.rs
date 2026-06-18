use clap::{Arg, Command};
use ssg::{config, content::*, formatted_text::check_math_markdown, render::*, version};
use std::{
    fs,
    path::{Path, PathBuf},
};

// These crates are used by the `ssg` library crate. We re-declare them here
// (as _) so that `cargo check` with -W unused_crate_dependencies does not
// complain when building only this binary target.
use chrono as _;
use comrak as _;
use regex as _;
use serde as _;
use serde_json as _;
use serde_yaml as _;
use tera as _;
use walkdir::WalkDir;

struct CliArgs {
    path: PathBuf,
    config_path: Option<PathBuf>,
    check_math: bool,
    strict_math: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run(parse_args()?)
}

fn parse_args() -> Result<CliArgs, Box<dyn std::error::Error>> {
    let matches = cli_command().get_matches();

    let path = matches
        .get_one::<String>("path")
        .map(PathBuf::from)
        .ok_or("Missing required 'path' argument")?;
    let config_path = matches.get_one::<PathBuf>("config").cloned();
    let check_math = matches.get_flag("check-math");
    let strict_math = matches.get_flag("strict-math");

    if !check_math && config_path.is_none() {
        return Err("Missing required --config argument".into());
    }

    Ok(CliArgs {
        path,
        config_path,
        check_math,
        strict_math,
    })
}

fn cli_command() -> Command {
    Command::new("ssg-content")
        .version(version::VERSION)
        .author("Hadi Moshayedi")
        .about("Generates HTML files from definitions")
        .arg(
            Arg::new("check-math")
                .long("check-math")
                .help("Check Markdown math directives without rendering HTML")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("strict-math")
                .long("strict-math")
                .help("Treat math shorthand and OCR warnings as check errors")
                .requires("check-math")
                .action(clap::ArgAction::SetTrue),
        )
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
                .value_name("FILE")
                .value_parser(clap::value_parser!(PathBuf)),
        )
}

fn run(args: CliArgs) -> Result<(), Box<dyn std::error::Error>> {
    if args.check_math {
        let default_math_shorthand = load_optional_config(args.config_path.as_deref())?
            .is_some_and(|config| config.math_shorthand);
        return check_math_path(&args.path, default_math_shorthand, args.strict_math);
    }

    let config_path = args
        .config_path
        .as_deref()
        .ok_or("Missing required --config argument")?;
    let config = config::Config::load(config_path)?;

    fs::create_dir_all(&config.build_dir)?;

    let content = load_content(&args.path, &config)?;
    let renderer = Renderer::new(&config)?;
    let html = render_with_images(&args.path, &content, &renderer, &config)?;

    write_content_output(&content, html)?;

    Ok(())
}

fn load_optional_config(
    config_path: Option<&Path>,
) -> Result<Option<config::Config>, Box<dyn std::error::Error>> {
    config_path.map(config::Config::load).transpose()
}

fn check_math_path(
    path: &Path,
    default_math_shorthand: bool,
    strict: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let files = markdown_files(path)?;
    let mut error_count = 0;

    for file in files {
        let markdown = fs::read_to_string(&file)?;
        for diagnostic in check_math_markdown(&markdown, default_math_shorthand, strict) {
            if diagnostic.severity.as_str() == "error" {
                error_count += 1;
            }
            eprintln!(
                "{}:{}: {}: {}",
                file.display(),
                diagnostic.line,
                diagnostic.severity.as_str(),
                diagnostic.message
            );
        }
    }

    if error_count > 0 {
        Err(format!("{error_count} math check error(s)").into())
    } else {
        Ok(())
    }
}

fn markdown_files(path: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    if path.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }

    let mut files = Vec::new();
    for entry in WalkDir::new(path).sort_by_file_name() {
        let entry = entry?;
        if entry.file_type().is_file()
            && entry
                .path()
                .extension()
                .and_then(|extension| extension.to_str())
                == Some("md")
        {
            files.push(entry.path().to_path_buf());
        }
    }

    Ok(files)
}

fn load_content(
    path: &Path,
    config: &config::Config,
) -> Result<Content, Box<dyn std::error::Error>> {
    Content::load(path, config)
        .map_err(|e| format!("Failed to load content from {}: {e}", path.display()).into())
}

fn render_with_images(
    path: &Path,
    content: &Content,
    renderer: &Renderer,
    config: &config::Config,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut html = content.render_html(renderer, config)?;
    let mut image_processor = ImageProcessor::new(
        path.to_path_buf(),
        config.content_dir.clone(),
        config.build_dir.clone(),
    )?;

    if image_processor.has_images() {
        image_processor.copy_images_to_build_dir()?;
        html = image_processor.update_html_with_image_urls(&html);
    }

    Ok(html)
}

fn write_content_output(content: &Content, html: String) -> Result<(), Box<dyn std::error::Error>> {
    let output_file_path = &content.metadata().output_path;

    if let Some(parent) = output_file_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(output_file_path, html)?;

    Ok(())
}
