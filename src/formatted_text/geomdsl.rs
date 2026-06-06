use std::error::Error;
use std::ffi::OsString;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use crate::config::Config;

const STATIC_ASSETS_DIR: &str = "static/assets";
const DEFAULT_FORMAT: &str = "svg";

const RENDER_SCRIPT: &str = r#"
import sys
from geomdsl import render

source = sys.stdin.read()
output = sys.argv[1]
base_path = sys.argv[2]
fmt = sys.argv[3] or None
dpi = int(sys.argv[4]) if sys.argv[4] else None

render(source, output=output, fmt=fmt, dpi=dpi, base_path=base_path)
"#;

#[derive(Debug, Clone)]
struct GeomDslBlockConfig {
    format: String,
    dpi: Option<u32>,
    width: Option<String>,
    alt: String,
    caption: Option<String>,
    class: Option<String>,
    id: Option<String>,
}

impl Default for GeomDslBlockConfig {
    fn default() -> Self {
        Self {
            format: DEFAULT_FORMAT.to_string(),
            dpi: None,
            width: None,
            alt: String::new(),
            caption: None,
            class: None,
            id: None,
        }
    }
}

pub fn preprocess_geomdsl_blocks(
    markdown: &str,
    source_path: &Path,
    config: &Config,
) -> Result<String, Box<dyn Error>> {
    if !markdown.contains(":::geomdsl") {
        return Ok(markdown.to_string());
    }

    let mut out = String::new();
    let mut lines = markdown.lines();
    let mut in_fence = false;
    let mut block_index = 0;

    while let Some(line) = lines.next() {
        if is_code_fence_line(line) {
            in_fence = !in_fence;
            append_line(&mut out, line);
        } else if !in_fence && starts_directive(line, ":::geomdsl") {
            block_index += 1;
            let body = take_directive_body(&mut lines);
            let block_config = parse_block_config(directive_remainder(line, ":::geomdsl"))?;
            let html = render_geomdsl_block(
                &body.join("\n"),
                source_path,
                config,
                block_index,
                &block_config,
            )?;
            out.push_str(&html);
            out.push('\n');
        } else {
            append_line(&mut out, line);
        }
    }

    Ok(out)
}

fn render_geomdsl_block(
    source: &str,
    source_path: &Path,
    config: &Config,
    block_index: usize,
    block_config: &GeomDslBlockConfig,
) -> Result<String, Box<dyn Error>> {
    let asset = build_asset_paths(
        source,
        source_path,
        config,
        block_index,
        &block_config.format,
    )?;
    render_geomdsl_source(
        source,
        source_path,
        config,
        block_config,
        &asset.output_path,
    )?;
    Ok(render_image_html(&asset.url, block_config))
}

struct AssetPaths {
    output_path: PathBuf,
    url: String,
}

fn build_asset_paths(
    source: &str,
    source_path: &Path,
    config: &Config,
    block_index: usize,
    format: &str,
) -> Result<AssetPaths, Box<dyn Error>> {
    let source_path = absolute_path(source_path)?;
    let source_dir = source_path
        .parent()
        .ok_or_else(|| format!("Source path has no parent: {}", source_path.display()))?;
    let content_dir = absolute_path(&config.content_dir)?;
    let build_dir = absolute_path(&config.build_dir)?;
    let relative_source_dir = source_dir.strip_prefix(&content_dir).map_err(|_e| {
        format!(
            "Source path {} is not under content directory {}",
            source_path.display(),
            content_dir.display()
        )
    })?;
    let relative_asset_dir = PathBuf::from(STATIC_ASSETS_DIR)
        .join(relative_source_dir)
        .join(".geomdsl");
    let file_name = generated_file_name(source_path.as_path(), source, block_index, format);
    let relative_asset_path = relative_asset_dir.join(file_name);
    let output_path = build_dir.join(&relative_asset_path);

    let url = format!("/{}", normalize_path(&relative_asset_path));

    Ok(AssetPaths { output_path, url })
}

fn generated_file_name(
    source_path: &Path,
    source: &str,
    block_index: usize,
    format: &str,
) -> String {
    let stem = source_path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(sanitize_file_component)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "geomdsl".to_string());
    let hash = fnv1a_hash(source.as_bytes());
    format!("{stem}-{block_index}-{hash:016x}.{format}")
}

fn fnv1a_hash(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn sanitize_file_component(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect()
}

fn render_geomdsl_source(
    source: &str,
    source_path: &Path,
    config: &Config,
    block_config: &GeomDslBlockConfig,
    output_path: &Path,
) -> Result<(), Box<dyn Error>> {
    let geomdsl_dir = geomdsl_dir(config)?;
    if !geomdsl_dir.is_dir() {
        return Err(format!(
            "GeomDSL directory does not exist: {}",
            geomdsl_dir.display()
        )
        .into());
    }

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let dpi = block_config.dpi.or(config.geomdsl_dpi);
    let dpi_arg = dpi.map(|value| value.to_string()).unwrap_or_default();
    let source_path = absolute_path(source_path)?;
    let timeout = Duration::from_secs(config.geomdsl_timeout_seconds);
    let mut command = Command::new(&config.geomdsl_python);
    command
        .arg("-c")
        .arg(RENDER_SCRIPT)
        .arg(output_path)
        .arg(source_path)
        .arg(&block_config.format)
        .arg(dpi_arg)
        .current_dir(&geomdsl_dir)
        .env("MPLBACKEND", "Agg")
        .env("PYTHONPATH", python_path_with_geomdsl(&geomdsl_dir)?)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .map_err(|e| format!("Failed to start GeomDSL renderer: {e}"))?;
    write_child_stdin(&mut child, source)?;
    wait_for_child(&mut child, timeout).map_err(|e| format!("GeomDSL render failed: {e}").into())
}

fn geomdsl_dir(config: &Config) -> Result<PathBuf, Box<dyn Error>> {
    if let Some(path) = &config.geomdsl_dir {
        return absolute_path(path);
    }

    let home = std::env::var_os("HOME").ok_or("HOME is not set; configure geomdsl_dir")?;
    Ok(PathBuf::from(home).join("projects/geomdsl2"))
}

fn python_path_with_geomdsl(geomdsl_dir: &Path) -> Result<OsString, Box<dyn Error>> {
    let mut paths = vec![geomdsl_dir.to_path_buf()];
    if let Some(existing) = std::env::var_os("PYTHONPATH") {
        paths.extend(std::env::split_paths(&existing));
    }
    Ok(std::env::join_paths(paths)?)
}

fn write_child_stdin(child: &mut Child, source: &str) -> Result<(), Box<dyn Error>> {
    let mut stdin = child
        .stdin
        .take()
        .ok_or("No stdin available for GeomDSL renderer")?;
    stdin.write_all(source.as_bytes())?;
    Ok(())
}

fn wait_for_child(child: &mut Child, timeout: Duration) -> Result<(), String> {
    let start = Instant::now();

    loop {
        if start.elapsed() > timeout {
            let _ = child.kill();
            return Err(format!("Timeout after {:?}", timeout));
        }

        if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
            let stdout = read_pipe(&mut child.stdout)?;
            let stderr = read_pipe(&mut child.stderr)?;
            if status.success() {
                return Ok(());
            }

            let diagnostic = if stderr.trim().is_empty() {
                stdout
            } else {
                stderr
            };
            return Err(diagnostic.trim().to_string());
        }

        std::thread::sleep(Duration::from_millis(10));
    }
}

fn read_pipe<R: Read>(pipe: &mut Option<R>) -> Result<String, String> {
    let mut output = String::new();
    if let Some(mut pipe) = pipe.take() {
        pipe.read_to_string(&mut output)
            .map_err(|e| format!("Output read failed: {e}"))?;
    }
    Ok(output)
}

fn render_image_html(url: &str, config: &GeomDslBlockConfig) -> String {
    let class = config
        .class
        .as_deref()
        .map(|class| format!(" {}", escape_html(class)))
        .unwrap_or_default();
    let id_attr = config
        .id
        .as_deref()
        .map(|id| format!(r#" id="{}""#, escape_html(id)))
        .unwrap_or_default();
    let width_style = config
        .width
        .as_deref()
        .map(|width| {
            format!(
                r#" style="width:90%; max-width: {}; margin: 1.5rem auto; text-align: center;""#,
                escape_html(&css_width(width))
            )
        })
        .unwrap_or_else(|| r#" style="margin: 1.5rem auto; text-align: center;""#.to_string());
    let img_style = config
        .width
        .as_deref()
        .map(|width| {
            format!(
                r#" style="width:100%; max-width: {}; height: auto;""#,
                escape_html(&css_width(width))
            )
        })
        .unwrap_or_default();
    let caption = config
        .caption
        .as_deref()
        .map(|caption| format!("\n  <figcaption>{}</figcaption>", escape_html(caption)))
        .unwrap_or_default();

    format!(
        r#"<figure{id_attr} class="figure-block geomdsl-figure{class}"{width_style}>
  <img src="{}" alt="{}"{img_style}>{caption}
</figure>
"#,
        escape_html(url),
        escape_html(&config.alt),
        id_attr = id_attr,
        class = class,
        width_style = width_style,
        img_style = img_style,
        caption = caption,
    )
}

fn css_width(width: &str) -> String {
    let trimmed = width.trim();
    if trimmed.chars().all(|ch| ch.is_ascii_digit() || ch == '.') {
        format!("{trimmed}px")
    } else {
        trimmed.to_string()
    }
}

fn parse_block_config(header: &str) -> Result<GeomDslBlockConfig, Box<dyn Error>> {
    let mut config = GeomDslBlockConfig::default();

    for token in split_header_tokens(header)? {
        if token.is_empty() {
            continue;
        }

        if let Some((key, value)) = token.split_once('=') {
            config.set(key, value)?;
        } else if token == "svg" || token == "png" {
            config.format = token;
        }
    }

    validate_format(&config.format)?;
    Ok(config)
}

impl GeomDslBlockConfig {
    fn set(&mut self, key: &str, value: &str) -> Result<(), Box<dyn Error>> {
        match key {
            "format" | "fmt" => {
                validate_format(value)?;
                self.format = value.to_string();
            }
            "dpi" => self.dpi = Some(value.parse()?),
            "width" => self.width = Some(value.to_string()),
            "alt" => self.alt = value.to_string(),
            "caption" => self.caption = Some(value.to_string()),
            "class" => self.class = Some(value.to_string()),
            "id" => self.id = Some(value.to_string()),
            _ => {}
        }
        Ok(())
    }
}

fn validate_format(format: &str) -> Result<(), Box<dyn Error>> {
    match format {
        "svg" | "png" => Ok(()),
        _ => Err(format!("Unsupported GeomDSL output format: {format}").into()),
    }
}

fn split_header_tokens(header: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;

    for ch in header.chars() {
        match (quote, ch) {
            (Some(active), ch) if ch == active => quote = None,
            (Some(_), ch) => current.push(ch),
            (None, '"' | '\'') => quote = Some(ch),
            (None, ch) if ch.is_whitespace() => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            (None, ch) => current.push(ch),
        }
    }

    if let Some(active) = quote {
        return Err(format!("Unclosed quote in GeomDSL block header: {active}").into());
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    Ok(tokens)
}

fn is_code_fence_line(line: &str) -> bool {
    let line = line.trim_start();
    line.starts_with("```") || line.starts_with("~~~")
}

fn starts_directive(line: &str, directive: &str) -> bool {
    let line = line.trim_start();
    line == directive
        || line.strip_prefix(directive).is_some_and(|rest| {
            rest.chars().next().is_some_and(char::is_whitespace) || rest.starts_with('[')
        })
}

fn directive_remainder<'a>(line: &'a str, directive: &str) -> &'a str {
    line.trim_start()
        .strip_prefix(directive)
        .unwrap_or("")
        .trim()
}

fn take_directive_body<'a>(lines: &mut impl Iterator<Item = &'a str>) -> Vec<&'a str> {
    let mut body = Vec::new();
    let mut in_fence = false;

    for line in lines {
        if is_code_fence_line(line) {
            in_fence = !in_fence;
        }
        if !in_fence && line.trim_start().starts_with(":::") {
            break;
        }
        body.push(line);
    }

    body
}

fn append_line(out: &mut String, line: &str) {
    out.push_str(line);
    out.push('\n');
}

fn absolute_path(path: &Path) -> Result<PathBuf, Box<dyn Error>> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn escape_html(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn parses_header_tokens_with_quotes() -> Result<(), Box<dyn Error>> {
        let config = parse_block_config(
            r#"png dpi=220 width=540 alt="Triangle construction" caption='Uses includes'"#,
        )?;

        assert_eq!(config.format, "png");
        assert_eq!(config.dpi, Some(220));
        assert_eq!(config.width, Some("540".to_string()));
        assert_eq!(config.alt, "Triangle construction");
        assert_eq!(config.caption, Some("Uses includes".to_string()));

        Ok(())
    }

    #[test]
    fn rejects_unsupported_formats() {
        let err = parse_block_config("format=pdf")
            .expect_err("pdf should not be accepted for img output")
            .to_string();

        assert!(err.contains("Unsupported GeomDSL output format"));
    }

    #[test]
    fn leaves_geomdsl_directive_inside_code_fence() -> Result<(), Box<dyn Error>> {
        let dir = tempdir()?;
        let source_path = dir.path().join("content/page.md");
        fs::create_dir_all(source_path.parent().unwrap())?;
        fs::write(&source_path, "")?;
        let config = Config {
            content_dir: dir.path().join("content"),
            build_dir: dir.path().join("build"),
            geomdsl_dir: Some(dir.path().join("missing")),
            ..Default::default()
        };
        let input = r#"```markdown
:::geomdsl
scene()
:::
```
"#;

        let output = preprocess_geomdsl_blocks(input, &source_path, &config)?;

        assert!(output.contains(":::geomdsl"));

        Ok(())
    }

    #[test]
    fn builds_stable_asset_urls() -> Result<(), Box<dyn Error>> {
        let dir = tempdir()?;
        let config = Config {
            content_dir: dir.path().join("content"),
            build_dir: dir.path().join("build"),
            ..Default::default()
        };
        let source_path = dir.path().join("content/en/page.md");
        fs::create_dir_all(source_path.parent().unwrap())?;
        fs::write(&source_path, "")?;

        let first = build_asset_paths("scene()", &source_path, &config, 1, "svg")?;
        let second = build_asset_paths("scene()", &source_path, &config, 1, "svg")?;

        assert_eq!(first.url, second.url);
        assert!(first.url.starts_with("/static/assets/en/.geomdsl/page-1-"));
        assert!(first.url.ends_with(".svg"));

        Ok(())
    }
}
