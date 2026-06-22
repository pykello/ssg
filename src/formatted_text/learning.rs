use std::collections::BTreeMap;
use std::error::Error;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::config::Config;

#[derive(Debug, Clone)]
struct LearningItem {
    kind: String,
    section: String,
    status: String,
    source_path: PathBuf,
}

#[derive(Debug, Default)]
struct LearningItemConfig {
    id: Option<String>,
    kind: Option<String>,
    title: Option<String>,
    section: Option<String>,
    status: Option<String>,
}

#[derive(Debug, Default)]
struct LearningProgressConfig {
    root: Option<String>,
    title: Option<String>,
}

#[derive(Debug, Default)]
struct ProgressCounts {
    kinds: BTreeMap<String, usize>,
    done: usize,
    partial: usize,
    todo: usize,
}

impl ProgressCounts {
    fn increment_kind(&mut self, kind: &str) {
        *self.kinds.entry(kind.to_string()).or_insert(0) += 1;
    }

    fn kind_count(&self, kind: &str) -> usize {
        self.kinds.get(kind).copied().unwrap_or(0)
    }

    fn total(&self) -> usize {
        self.kinds.values().sum()
    }
}

pub fn preprocess_learning_blocks(
    markdown: &str,
    source_path: &Path,
    config: &Config,
) -> Result<String, Box<dyn Error>> {
    if !markdown.contains(":::learning-") {
        return Ok(markdown.to_string());
    }

    let mut out = String::new();
    let mut lines = markdown.lines();
    let mut in_fence = false;

    while let Some(line) = lines.next() {
        if is_code_fence_line(line) {
            in_fence = !in_fence;
            append_line(&mut out, line);
        } else if !in_fence && starts_directive(line, ":::learning-item") {
            let body = take_directive_body(&mut lines, directive_marker_len(line).unwrap_or(3));
            let item_config =
                parse_learning_item_config(directive_remainder(line, ":::learning-item"))?;
            write_learning_item(&mut out, &item_config, &body);
        } else if !in_fence && starts_directive(line, ":::learning-progress") {
            let _body = take_directive_body(&mut lines, directive_marker_len(line).unwrap_or(3));
            let progress_config =
                parse_learning_progress_config(directive_remainder(line, ":::learning-progress"))?;
            out.push_str(&render_learning_progress(
                &progress_config,
                source_path,
                config,
            )?);
            out.push('\n');
        } else {
            append_line(&mut out, line);
        }
    }

    Ok(out)
}

fn write_learning_item(out: &mut String, config: &LearningItemConfig, body: &[String]) {
    let kind = normalize_kind(config.kind.as_deref().unwrap_or("exercise"));
    let status = normalize_status(config.status.as_deref().unwrap_or("todo"));
    let title = config
        .title
        .clone()
        .or_else(|| config.id.clone())
        .unwrap_or_else(|| title_case(&kind));
    let section = config.section.as_deref().unwrap_or("Unsectioned");
    let id_attr = config
        .id
        .as_deref()
        .map(|id| format!(r#" id="{}""#, escape_html_attr(id)))
        .unwrap_or_default();
    let heading = learning_item_heading(&kind, &title);

    out.push_str(&format!(
        r#"<section{id_attr} class="learning-item learning-item--{status}" data-learning-type="{kind}" data-learning-status="{status}" data-learning-section="{section}">
<div class="learning-item__header"><strong>{heading}</strong><span class="learning-status learning-status--{status}">{status_label}</span></div>

"#,
        id_attr = id_attr,
        status = escape_html_attr(&status),
        kind = escape_html_attr(&kind),
        section = escape_html_attr(section),
        heading = escape_html(&heading),
        status_label = escape_html(&title_case(&status)),
    ));

    for line in body {
        append_line(out, line);
    }

    out.push_str("\n</section>\n");
}

fn learning_item_heading(kind: &str, title: &str) -> String {
    let kind_label = title_case(kind);
    let labels: &[&str] = match kind {
        "exercise" => &["Exercise"],
        "theorem" => &["Theorem", "Proposition", "Lemma", "Corollary"],
        _ => &[],
    };

    if title_starts_with_label(title, &kind_label)
        || labels
            .iter()
            .any(|label| title_starts_with_label(title, label))
    {
        title.to_string()
    } else {
        format!("{kind_label}: {title}")
    }
}

fn title_starts_with_label(title: &str, label: &str) -> bool {
    let Some(rest) = title.trim_start().strip_prefix(label) else {
        return false;
    };
    rest.is_empty()
        || rest
            .chars()
            .next()
            .is_some_and(|ch| ch.is_whitespace() || matches!(ch, ':' | '.' | '('))
}

fn render_learning_progress(
    progress_config: &LearningProgressConfig,
    source_path: &Path,
    config: &Config,
) -> Result<String, Box<dyn Error>> {
    let root = progress_root(progress_config, source_path)?;
    let items = collect_learning_items(&root, source_path)?;
    let mut by_section: BTreeMap<(PathBuf, String), ProgressCounts> = BTreeMap::new();
    let mut totals = ProgressCounts::default();

    for item in items {
        let key = (item.source_path.clone(), item.section.clone());
        let counts = by_section.entry(key).or_default();
        counts.increment_kind(&item.kind);
        totals.increment_kind(&item.kind);
        match item.status.as_str() {
            "done" => {
                counts.done += 1;
                totals.done += 1;
            }
            "partial" => {
                counts.partial += 1;
                totals.partial += 1;
            }
            _ => {
                counts.todo += 1;
                totals.todo += 1;
            }
        }
    }

    let title = progress_config
        .title
        .as_deref()
        .unwrap_or("Learning Progress");
    let kind_columns = progress_kind_columns(&totals);
    let mut html = String::new();
    html.push_str(&format!(
        r#"<section class="learning-progress">
<h2>{}</h2>
<table>
<thead><tr><th>Page</th><th>Section</th>"#,
        escape_html(title)
    ));
    for kind in &kind_columns {
        html.push_str(&format!(
            "<th>{}</th>",
            escape_html(&plural_kind_label(kind))
        ));
    }
    html.push_str(
        r#"<th>Done</th><th>Partial</th><th>Todo</th><th>Progress</th></tr></thead>
<tbody>
"#,
    );

    for ((path, section), counts) in &by_section {
        let page_title = markdown_title(path).unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Page")
                .to_string()
        });
        let url = crate::content::content_url(path, config)?;
        let total = counts.total();
        html.push_str(&format!(
            r#"<tr><td><a href="{url}">{page}</a></td><td>{section}</td>"#,
            url = escape_html_attr(&url),
            page = escape_html(&page_title),
            section = escape_html(section),
        ));
        for kind in &kind_columns {
            html.push_str(&format!("<td>{}</td>", counts.kind_count(kind)));
        }
        html.push_str(&format!(
            r#"<td>{done}</td><td>{partial}</td><td>{todo}</td><td>{progress}</td></tr>
"#,
            done = counts.done,
            partial = counts.partial,
            todo = counts.todo,
            progress = progress_label(counts.done, total),
        ));
    }

    let total_items = totals.total();
    html.push_str(
        r#"</tbody>
<tfoot><tr><th colspan="2">Total</th>"#,
    );
    for kind in &kind_columns {
        html.push_str(&format!("<th>{}</th>", totals.kind_count(kind)));
    }
    html.push_str(&format!(
        r#"<th>{done}</th><th>{partial}</th><th>{todo}</th><th>{progress}</th></tr></tfoot>
</table>
</section>
"#,
        done = totals.done,
        partial = totals.partial,
        todo = totals.todo,
        progress = progress_label(totals.done, total_items),
    ));

    Ok(html)
}

fn collect_learning_items(
    root: &Path,
    source_path: &Path,
) -> Result<Vec<LearningItem>, Box<dyn Error>> {
    let mut items = Vec::new();
    if !root.exists() {
        return Ok(items);
    }

    for entry in WalkDir::new(root).sort_by_file_name() {
        let entry = entry?;
        if !entry.file_type().is_file()
            || entry.path().extension().and_then(|s| s.to_str()) != Some("md")
        {
            continue;
        }
        if entry.path() == source_path {
            continue;
        }
        let markdown = std::fs::read_to_string(entry.path())?;
        items.extend(parse_learning_items(&markdown, entry.path())?);
    }

    Ok(items)
}

fn parse_learning_items(
    markdown: &str,
    source_path: &Path,
) -> Result<Vec<LearningItem>, Box<dyn Error>> {
    let mut items = Vec::new();
    let mut in_fence = false;

    for line in markdown.lines() {
        if is_code_fence_line(line) {
            in_fence = !in_fence;
        } else if !in_fence && starts_directive(line, ":::learning-item") {
            let config = parse_learning_item_config(directive_remainder(line, ":::learning-item"))?;
            items.push(LearningItem {
                kind: normalize_kind(config.kind.as_deref().unwrap_or("exercise")),
                section: config.section.unwrap_or_else(|| "Unsectioned".to_string()),
                status: normalize_status(config.status.as_deref().unwrap_or("todo")),
                source_path: source_path.to_path_buf(),
            });
        }
    }

    Ok(items)
}

fn progress_root(
    progress_config: &LearningProgressConfig,
    source_path: &Path,
) -> Result<PathBuf, Box<dyn Error>> {
    let base_dir = source_path
        .parent()
        .ok_or_else(|| format!("Source path has no parent: {}", source_path.display()))?;
    let root = progress_config.root.as_deref().unwrap_or(".");
    let root = Path::new(root);
    if root.is_absolute() {
        return Err("learning-progress root must be relative".into());
    }
    Ok(base_dir.join(root))
}

fn markdown_title(path: &Path) -> Option<String> {
    let markdown = std::fs::read_to_string(path).ok()?;
    markdown.lines().find_map(first_markdown_heading)
}

fn first_markdown_heading(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    let heading = trimmed.strip_prefix("# ")?;
    Some(heading.trim().to_string())
}

fn parse_learning_item_config(header: &str) -> Result<LearningItemConfig, Box<dyn Error>> {
    let values = parse_key_values(header)?;
    let mut config = LearningItemConfig::default();
    for (key, value) in values {
        match key.as_str() {
            "id" => config.id = Some(value),
            "type" | "kind" => config.kind = Some(value),
            "title" => config.title = Some(value),
            "section" => config.section = Some(value),
            "status" => config.status = Some(value),
            _ => {}
        }
    }
    Ok(config)
}

fn parse_learning_progress_config(header: &str) -> Result<LearningProgressConfig, Box<dyn Error>> {
    let values = parse_key_values(header)?;
    let mut config = LearningProgressConfig::default();
    for (key, value) in values {
        match key.as_str() {
            "root" | "path" => config.root = Some(value),
            "title" => config.title = Some(value),
            _ => {}
        }
    }
    Ok(config)
}

fn parse_key_values(header: &str) -> Result<Vec<(String, String)>, Box<dyn Error>> {
    let mut values = Vec::new();
    for token in tokenize_header(header)? {
        if let Some((key, value)) = token.split_once('=') {
            values.push((key.to_string(), value.to_string()));
        }
    }
    Ok(values)
}

fn tokenize_header(header: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut chars = header.chars().peekable();
    let mut in_quote = false;

    while let Some(ch) = chars.next() {
        match ch {
            '"' => in_quote = !in_quote,
            '\\' if in_quote => {
                if let Some(next) = chars.next() {
                    current.push(next);
                }
            }
            c if c.is_whitespace() && !in_quote => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(ch),
        }
    }

    if in_quote {
        return Err("Unterminated quoted learning directive value".into());
    }
    if !current.is_empty() {
        tokens.push(current);
    }

    Ok(tokens)
}

fn normalize_kind(kind: &str) -> String {
    let normalized = kind.trim().to_ascii_lowercase().replace([' ', '-'], "_");
    match normalized.as_str() {
        "" => "exercise".to_string(),
        "proof" => "theorem".to_string(),
        _ => normalized,
    }
}

fn normalize_status(status: &str) -> String {
    match status.trim().to_ascii_lowercase().as_str() {
        "done" | "solved" | "complete" | "completed" => "done".to_string(),
        "partial" | "in-progress" | "started" => "partial".to_string(),
        _ => "todo".to_string(),
    }
}

fn progress_label(done: usize, total: usize) -> String {
    if total == 0 {
        "0/0".to_string()
    } else {
        format!("{done}/{total}")
    }
}

fn progress_kind_columns(totals: &ProgressCounts) -> Vec<String> {
    let mut columns: Vec<String> = totals
        .kinds
        .iter()
        .filter(|(_, count)| **count > 0)
        .map(|(kind, _)| kind.clone())
        .collect();
    columns.sort_by(|left, right| {
        kind_order(left)
            .cmp(&kind_order(right))
            .then_with(|| left.cmp(right))
    });
    columns
}

fn kind_order(kind: &str) -> usize {
    match kind {
        "theorem" => 0,
        "review_question" => 1,
        "exercise" => 2,
        "computer_problem" => 3,
        _ => 4,
    }
}

fn plural_kind_label(kind: &str) -> String {
    let label = title_case(kind);
    match label.as_str() {
        "Exercise" => "Exercises".to_string(),
        "Theorem" => "Theorems".to_string(),
        "Review Question" => "Review Questions".to_string(),
        "Computer Problem" => "Computer Problems".to_string(),
        _ if label.ends_with('y') => format!("{}ies", label.trim_end_matches('y')),
        _ if label.ends_with('s') => label,
        _ => format!("{label}s"),
    }
}

fn title_case(value: &str) -> String {
    value
        .split(|ch: char| ch == '_' || ch == '-' || ch.is_whitespace())
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => {
                    format!(
                        "{}{}",
                        first.to_uppercase(),
                        chars.as_str().to_ascii_lowercase()
                    )
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn starts_directive(line: &str, directive: &str) -> bool {
    let line = line.trim_start();
    line == directive
        || line
            .strip_prefix(directive)
            .is_some_and(|rest| rest.chars().next().is_some_and(char::is_whitespace))
}

fn directive_remainder<'a>(line: &'a str, directive: &str) -> &'a str {
    line.trim_start()
        .strip_prefix(directive)
        .unwrap_or("")
        .trim()
}

fn take_directive_body<'a>(
    lines: &mut impl Iterator<Item = &'a str>,
    outer_marker_len: usize,
) -> Vec<String> {
    let mut body = Vec::new();
    let mut in_fence = false;
    let mut nested_directives: Vec<usize> = Vec::new();

    for line in lines {
        if is_code_fence_line(line) {
            in_fence = !in_fence;
        }

        if !in_fence {
            match directive_line(line) {
                Some(DirectiveLine::Open(marker_len)) => {
                    nested_directives.push(marker_len);
                }
                Some(DirectiveLine::Close(marker_len)) => {
                    if let Some(nested_marker_len) = nested_directives.last().copied() {
                        if marker_len >= nested_marker_len {
                            nested_directives.pop();
                        }
                    } else if marker_len >= outer_marker_len {
                        break;
                    }
                }
                None => {}
            }
        }
        body.push(line.to_string());
    }

    body
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DirectiveLine {
    Open(usize),
    Close(usize),
}

fn directive_line(line: &str) -> Option<DirectiveLine> {
    let line = line.trim_start();
    let marker_len = directive_marker_len(line)?;
    let rest = line[marker_len..].trim();

    if rest.is_empty() {
        Some(DirectiveLine::Close(marker_len))
    } else {
        Some(DirectiveLine::Open(marker_len))
    }
}

fn directive_marker_len(line: &str) -> Option<usize> {
    let marker_len = line
        .trim_start()
        .chars()
        .take_while(|ch| *ch == ':')
        .count();
    (marker_len >= 3).then_some(marker_len)
}

fn is_code_fence_line(line: &str) -> bool {
    let line = line.trim_start();
    line.starts_with("```") || line.starts_with("~~~")
}

fn append_line(out: &mut String, line: &str) {
    out.push_str(line);
    out.push('\n');
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_html_attr(value: &str) -> String {
    escape_html(value).replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formatted_text::FormattedText;
    use tempfile::TempDir;

    #[test]
    fn renders_learning_item_with_markdown_body() -> Result<(), Box<dyn Error>> {
        let input = r#":::learning-item type=exercise id=ex-1 section="Sheet 1" status=partial title="Problem 1"
Show that $1+1=2$.
:::
"#;
        let config = Config::default();
        let output = preprocess_learning_blocks(input, Path::new("/tmp/page.md"), &config)?;
        assert!(output.contains(r#"class="learning-item learning-item--partial""#));
        assert!(output.contains(r#"data-learning-section="Sheet 1""#));
        assert!(output.contains("Show that $1+1=2$."));
        Ok(())
    }

    #[test]
    fn renders_learning_item_with_custom_kind_label() -> Result<(), Box<dyn Error>> {
        let input = r#":::learning-item type=review_question id=rq-1 section="Review Questions" status=todo title="1.1"
True or false?
:::
"#;
        let config = Config::default();
        let output = preprocess_learning_blocks(input, Path::new("/tmp/page.md"), &config)?;
        assert!(output.contains(r#"data-learning-type="review_question""#));
        assert!(output.contains("Review Question: 1.1"));
        Ok(())
    }

    #[test]
    fn avoids_duplicate_learning_item_kind_label() -> Result<(), Box<dyn Error>> {
        let input = r#":::learning-item type=exercise id=ex-1 section="Sheet 1" status=todo title="Exercise 3.2"
Solve it.
:::

:::learning-item type=theorem id=prop-1 section="Sheet 1" status=todo title="Proposition 3.7"
Prove it.
:::

:::learning-item type=theorem id=lemma-1 section="Sheet 1" status=todo title="Lemma 1.1.2"
Prove it.
:::

:::learning-item type=theorem id=theorem-1 section="Sheet 1" status=todo title="3.1.2"
Prove it.
:::
"#;
        let config = Config::default();
        let output = preprocess_learning_blocks(input, Path::new("/tmp/page.md"), &config)?;
        assert!(output.contains("<strong>Exercise 3.2</strong>"));
        assert!(output.contains("<strong>Proposition 3.7</strong>"));
        assert!(output.contains("<strong>Lemma 1.1.2</strong>"));
        assert!(output.contains("<strong>Theorem: 3.1.2</strong>"));
        assert!(!output.contains("Exercise: Exercise 3.2"));
        assert!(!output.contains("Theorem: Proposition 3.7"));
        assert!(!output.contains("Theorem: Lemma 1.1.2"));
        Ok(())
    }

    #[test]
    fn preserves_nested_directives_inside_learning_item() -> Result<(), Box<dyn Error>> {
        let input = r#":::learning-item type=exercise id=ex-1 section="Sheet 1" status=todo title="Problem 1"
Show the identity.

:::proof[Solution]
Use this display:

:::math
v{x} = v{y}
=> norm(v{x}) <= eps
:::
:::

Then finish the argument.
:::

Outside the item.
"#;
        let config = Config::default();
        let output = preprocess_learning_blocks(input, Path::new("/tmp/page.md"), &config)?;
        let section_start = output.find("<section").expect("learning item section");
        let section_end = output.find("</section>").expect("learning item close");
        let section = &output[section_start..section_end];
        let outside = output.find("Outside the item.").expect("outside text");

        assert!(section.contains(":::proof[Solution]"));
        assert!(section.contains(":::math"));
        assert!(section.contains("Then finish the argument."));
        assert!(outside > section_end);
        Ok(())
    }

    #[test]
    fn renders_nested_math_and_proof_inside_learning_item() -> Result<(), Box<dyn Error>> {
        let input = r#":::learning-item type=exercise id=ex-1 section="Sheet 1" status=todo title="Problem 1"
Show the identity.

:::proof[Solution]
Use this display:

:::math
v{x} = v{y}
=> norm(v{x}) <= eps
:::
:::
:::
"#;
        let mut config = Config::default();
        config.escape_markdown_in_math = false;
        config.math_shorthand = true;
        let markdown = preprocess_learning_blocks(input, Path::new("/tmp/page.md"), &config)?;
        let output = FormattedText::Markdown(markdown).to_html(&config)?;

        assert!(output.contains(r#"class="learning-item learning-item--todo""#));
        assert!(output.contains(r#"class="collapse""#));
        assert!(output.contains(r"\mathbf{x} &amp;= \mathbf{y}"));
        assert!(output.contains(r"\left\lVert \mathbf{x} \right\rVert \le \epsilon"));
        assert!(!output.contains(":::learning-item"));
        assert!(!output.contains(":::math"));
        assert!(!output.contains(":::proof"));
        Ok(())
    }

    #[test]
    fn renders_nested_math_then_proof_inside_learning_item() -> Result<(), Box<dyn Error>> {
        let input = r#":::learning-item type=exercise id=ex-2 section="Sheet 1" status=todo title="Problem 2"
Use the estimate.

:::math
norm(v{x} - v{y}) <= eps
:::

:::proof[Solution]
Choose $v{x}$ and $v{y}$ with the required bound.
:::
:::
"#;
        let mut config = Config::default();
        config.escape_markdown_in_math = false;
        config.math_shorthand = true;
        let markdown = preprocess_learning_blocks(input, Path::new("/tmp/page.md"), &config)?;
        let output = FormattedText::Markdown(markdown).to_html(&config)?;

        assert!(output.contains(r#"class="learning-item learning-item--todo""#));
        assert!(output.contains(r"\left\lVert \mathbf{x} - \mathbf{y} \right\rVert"));
        assert!(output.contains(r#"class="collapse""#));
        assert!(output.contains("Solution."));
        assert!(!output.contains(":::learning-item"));
        assert!(!output.contains(":::math"));
        assert!(!output.contains(":::proof"));
        Ok(())
    }

    #[test]
    fn renders_progress_table_from_relative_root() -> Result<(), Box<dyn Error>> {
        let temp = TempDir::new()?;
        let content = temp.path().join("content");
        let build = temp.path().join("build");
        let project = content.join("en/learning/demo");
        let sheets = project.join("sheets");
        std::fs::create_dir_all(&sheets)?;
        std::fs::write(
            sheets.join("sheet-01.md"),
            r#"# Sheet 1

:::learning-item type=exercise id=ex-1 section="Section A" status=done
Statement.
:::

:::learning-item type=theorem id=thm-1 section="Section A" status=todo
Statement.
:::
"#,
        )?;
        let progress = project.join("progress.md");
        std::fs::write(&progress, "# Progress\n")?;
        let config = Config {
            content_dir: content,
            build_dir: build,
            template_dir: temp.path().join("templates"),
            ..Default::default()
        };

        let output = preprocess_learning_blocks(
            r#":::learning-progress root="sheets" title="Demo Progress"
:::
"#,
            &progress,
            &config,
        )?;

        assert!(output.contains("Demo Progress"));
        assert!(output.contains("Section A"));
        assert!(output.contains(">1</td><td>1</td><td>1</td>"));
        assert!(output.contains("1/2"));
        Ok(())
    }

    #[test]
    fn progress_table_hides_empty_kind_columns() -> Result<(), Box<dyn Error>> {
        let temp = TempDir::new()?;
        let content = temp.path().join("content");
        let build = temp.path().join("build");
        let project = content.join("en/learning/demo");
        let sheets = project.join("sheets");
        std::fs::create_dir_all(&sheets)?;
        std::fs::write(
            sheets.join("sheet-01.md"),
            r#"# Sheet 1

:::learning-item type=exercise id=ex-1 section="Section A" status=todo
Statement.
:::
"#,
        )?;
        let progress = project.join("index.md");
        std::fs::write(&progress, "# Progress\n")?;
        let config = Config {
            content_dir: content,
            build_dir: build,
            template_dir: temp.path().join("templates"),
            ..Default::default()
        };

        let output = preprocess_learning_blocks(
            r#":::learning-progress root="sheets" title="Demo Progress"
:::
"#,
            &progress,
            &config,
        )?;

        assert!(!output.contains("<th>Theorems</th>"));
        assert!(output.contains("<th>Exercises</th>"));
        assert!(output.contains("<th>1</th><th>0</th><th>0</th><th>1</th><th>0/1</th>"));
        Ok(())
    }

    #[test]
    fn progress_table_tracks_custom_kind_columns() -> Result<(), Box<dyn Error>> {
        let temp = TempDir::new()?;
        let content = temp.path().join("content");
        let build = temp.path().join("build");
        let project = content.join("en/learning/demo");
        let sheets = project.join("sheets");
        std::fs::create_dir_all(&sheets)?;
        std::fs::write(
            sheets.join("sheet-01.md"),
            r#"# Sheet 1

:::learning-item type=review_question id=rq-1 section="Review Questions" status=done
Statement.
:::

:::learning-item type=computer_problem id=cp-1 section="Computer Problems" status=todo
Statement.
:::
"#,
        )?;
        let progress = project.join("index.md");
        std::fs::write(&progress, "# Progress\n")?;
        let config = Config {
            content_dir: content,
            build_dir: build,
            template_dir: temp.path().join("templates"),
            ..Default::default()
        };

        let output = preprocess_learning_blocks(
            r#":::learning-progress root="sheets" title="Demo Progress"
:::
"#,
            &progress,
            &config,
        )?;

        assert!(output.contains("<th>Review Questions</th>"));
        assert!(output.contains("<th>Computer Problems</th>"));
        assert!(output.contains("<th>1</th><th>1</th><th>1</th><th>0</th><th>1</th><th>1/2</th>"));
        Ok(())
    }
}
