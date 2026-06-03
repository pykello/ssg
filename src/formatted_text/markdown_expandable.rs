use regex::Regex;
use std::sync::OnceLock;

static BRACKET_ARG_RE: OnceLock<Regex> = OnceLock::new();
static EXPAND_LINK_RE: OnceLock<Regex> = OnceLock::new();

fn bracket_arg_regex() -> &'static Regex {
    BRACKET_ARG_RE.get_or_init(|| Regex::new(r"\[([^\]]+)\]").expect("valid bracket arg regex"))
}

fn expand_link_regex() -> &'static Regex {
    EXPAND_LINK_RE.get_or_init(|| Regex::new(r"\[([^\]]+)\]").expect("valid expand link regex"))
}

fn extract_bracket_arg(line: &str) -> Option<String> {
    bracket_arg_regex()
        .captures(line)
        .map(|caps| caps[1].to_string())
}

fn is_fence_line(line: &str) -> bool {
    let line = line.trim_start();
    line.starts_with("```") || line.starts_with("~~~")
}

fn append_line(out: &mut String, line: &str) {
    out.push_str(line);
    out.push('\n');
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

fn is_directive_close(line: &str) -> bool {
    line.trim_start().starts_with(":::")
}

fn take_directive_body<'a>(lines: &mut impl Iterator<Item = &'a str>) -> Vec<&'a str> {
    let mut body = Vec::new();
    let mut in_fence = false;

    for line in lines {
        if is_fence_line(line) {
            in_fence = !in_fence;
        }
        if !in_fence && is_directive_close(line) {
            break;
        }
        body.push(line);
    }

    body
}

fn copy_directive_body<'a>(lines: &mut impl Iterator<Item = &'a str>, out: &mut String) {
    for body in take_directive_body(lines) {
        append_line(out, body);
    }
}

pub fn preprocess_cards(markdown: &str) -> String {
    let mut out = String::new();
    let mut lines = markdown.lines();
    let mut in_fence = false;

    while let Some(line) = lines.next() {
        if is_fence_line(line) {
            in_fence = !in_fence;
            append_line(&mut out, line);
        } else if !in_fence && starts_directive(line, ":::card") {
            let class = extract_bracket_arg(line).unwrap_or_default();

            out.push_str(&format!(r#"<div class="card {class}">"#, class = class));
            out.push('\n');
            out.push('\n');
            copy_directive_body(&mut lines, &mut out);
            out.push_str("  </div>\n\n");
        } else {
            append_line(&mut out, line);
        }
    }
    out
}

pub fn preprocess_semantic_cards(markdown: &str) -> String {
    let mut out = String::new();
    let mut lines = markdown.lines();
    let mut in_fence = false;

    while let Some(line) = lines.next() {
        if is_fence_line(line) {
            in_fence = !in_fence;
            append_line(&mut out, line);
        } else if !in_fence && starts_directive(line, ":::aside") {
            write_semantic_card(&mut out, "aside", &mut lines);
        } else if !in_fence && starts_directive(line, ":::remark") {
            write_semantic_card(&mut out, "remark", &mut lines);
        } else {
            append_line(&mut out, line);
        }
    }

    out
}

fn write_semantic_card<'a>(
    out: &mut String,
    class: &str,
    lines: &mut impl Iterator<Item = &'a str>,
) {
    out.push_str(&format!(r#"<aside class="card {class}">"#));
    out.push('\n');
    out.push('\n');
    copy_directive_body(lines, out);
    out.push_str("  </aside>\n\n");
}

pub fn preprocess_expandables(markdown: &str) -> String {
    let mut out = String::new();
    let mut id_counter = 0;
    let mut lines = markdown.lines();
    let mut in_fence = false;

    while let Some(line) = lines.next() {
        if is_fence_line(line) {
            in_fence = !in_fence;
            append_line(&mut out, line);
        } else if !in_fence && starts_directive(line, ":::expandable") {
            let heading_line = lines.next().unwrap_or("").trim();
            id_counter += 1;
            let id = format!("expand-{}", id_counter);
            write_expandable_block(&mut out, &id, heading_line, &mut lines);
        } else if !in_fence && starts_directive(line, ":::proof") {
            id_counter += 1;
            let id = format!("expand-{}", id_counter);
            let title = extract_bracket_arg(line).unwrap_or_else(|| "Proof".to_string());
            let heading_line = format!("**{}** [Click to Expand]", punctuate_title(&title));
            write_expandable_block(&mut out, &id, &heading_line, &mut lines);
        } else {
            append_line(&mut out, line);
        }
    }
    out
}

fn write_expandable_block<'a>(
    out: &mut String,
    id: &str,
    heading_line: &str,
    lines: &mut impl Iterator<Item = &'a str>,
) {
    let heading_line = render_expandable_heading(heading_line, id);

    out.push_str(&format!(
        r#"{heading_line}

<div class="collapse" id="{id}">
  <div class="card card-body">
"#,
        heading_line = heading_line,
        id = id
    ));

    copy_directive_body(lines, out);
    out.push_str("  </div>\n</div>\n");
}

fn punctuate_title(title: &str) -> String {
    let title = title.trim();
    if title.ends_with(['.', ':', '?', '!']) {
        title.to_string()
    } else {
        format!("{title}.")
    }
}

fn render_expandable_heading(heading_line: &str, id: &str) -> String {
    expand_link_regex()
        .replace_all(heading_line, |caps: &regex::Captures| {
            format!(
                r#"<a class="expand-link" data-bs-toggle="collapse" href='#{id}'>{}</a>"#,
                &caps[1],
                id = id
            )
        })
        .into_owned()
}

pub fn preprocess_figures(markdown: &str) -> String {
    let mut out = String::new();
    let mut lines = markdown.lines();
    let mut in_fence = false;

    while let Some(line) = lines.next() {
        if is_fence_line(line) {
            in_fence = !in_fence;
            append_line(&mut out, line);
        } else if !in_fence && starts_directive(line, ":::figure") {
            let body = take_directive_body(&mut lines);
            out.push_str(&render_figure(
                directive_remainder(line, ":::figure"),
                &body,
            ));
        } else {
            append_line(&mut out, line);
        }
    }

    out
}

#[derive(Debug, Default)]
struct FigureConfig {
    source: Option<String>,
    id: Option<String>,
    class: Option<String>,
    width: Option<String>,
    ratio: Option<String>,
    alt: Option<String>,
    caption: Option<String>,
}

fn render_figure(header: &str, body: &[&str]) -> String {
    let config = parse_figure_config(header, body);

    if let Some(id) = config.id.as_deref() {
        render_figure_container(id, &config)
    } else if let Some(source) = config.source.as_deref() {
        render_figure_image(source, &config)
    } else {
        String::new()
    }
}

fn parse_figure_config(header: &str, body: &[&str]) -> FigureConfig {
    let mut config = FigureConfig::default();

    for token in header.split_whitespace() {
        if let Some((key, value)) = token.split_once('=') {
            config.set(key, value);
        } else if config.source.is_none() {
            config.source = Some(token.to_string());
        }
    }

    for line in body {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some((key, value)) = line.split_once(':') {
            config.set(key.trim(), value.trim());
        }
    }

    config
}

impl FigureConfig {
    fn set(&mut self, key: &str, value: &str) {
        match key {
            "src" | "source" => self.source = Some(value.to_string()),
            "id" => self.id = Some(value.to_string()),
            "class" => self.class = Some(value.to_string()),
            "width" => self.width = Some(value.to_string()),
            "ratio" => self.ratio = Some(value.to_string()),
            "alt" => self.alt = Some(value.to_string()),
            "caption" => self.caption = Some(value.to_string()),
            _ => {}
        }
    }
}

fn render_figure_container(id: &str, config: &FigureConfig) -> String {
    let width = config.width.as_deref().unwrap_or("480");
    let ratio = config.ratio.as_deref().unwrap_or("1 / 1");
    let class_attr = config
        .class
        .as_deref()
        .map(|class| format!(r#" class="{}""#, escape_html(class)))
        .unwrap_or_default();
    let div = format!(
        r#"<div id="{}"{class_attr} style="width:90%; max-width: {}px; aspect-ratio: {}; margin: 20px auto;"></div>"#,
        escape_html(id),
        escape_html(width),
        escape_html(ratio),
        class_attr = class_attr,
    );

    wrap_figure_if_captioned(div, config.caption.as_deref())
}

fn render_figure_image(source: &str, config: &FigureConfig) -> String {
    let alt = config.alt.as_deref().unwrap_or_default();
    let class_attr = config
        .class
        .as_deref()
        .map(|class| format!(r#" class="{}""#, escape_html(class)))
        .unwrap_or_default();
    let style_attr = config
        .width
        .as_deref()
        .map(|width| {
            format!(
                r#" style="width:90%; max-width: {}px; height: auto; margin: 0 auto;""#,
                escape_html(width)
            )
        })
        .unwrap_or_default();
    let img = format!(
        r#"<img src="{}" alt="{}"{class_attr}{style_attr}>"#,
        escape_html(source),
        escape_html(alt),
        class_attr = class_attr,
        style_attr = style_attr,
    );

    wrap_figure_if_captioned(img, config.caption.as_deref())
}

fn wrap_figure_if_captioned(content: String, caption: Option<&str>) -> String {
    if let Some(caption) = caption {
        format!(
            r#"<figure class="figure-block" style="margin: 1.5rem auto; text-align: center;">
  {content}
  <figcaption>{}</figcaption>
</figure>
"#,
            escape_html(caption)
        )
    } else {
        format!("{content}\n")
    }
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
mod test_markdown_expandable {
    use super::*;

    #[test]
    fn test_preprocess_expandables() {
        let input = r#"
:::expandable
**Heading** [Click to Expand]
Some text

More text
::::

:::expandable
**Heading 2** ([Expand])
Some more
"#;
        let out = preprocess_expandables(input);
        assert!(out.contains(r#"**Heading** <a class="expand-link" data-bs-toggle="collapse" href='#expand-1'>Click to Expand</a>"#));
        assert!(out.contains(r#"**Heading 2** (<a class="expand-link" data-bs-toggle="collapse" href='#expand-2'>Expand</a>)"#));
    }

    #[test]
    fn leaves_expandable_marker_inside_code_fence() {
        let input = r#"```markdown
:::expandable
**Heading** [Click]
:::
```
"#;
        let out = preprocess_expandables(input);

        assert!(out.contains(":::expandable"));
        assert!(!out.contains(r#"class="collapse""#));
    }

    #[test]
    fn leaves_closing_marker_inside_expandable_code_fence() {
        let input = r#":::expandable
**Heading** [Click]

```markdown
:::
```

After code
:::
"#;
        let out = preprocess_expandables(input);

        assert!(out.contains("After code"));
        assert!(out.contains(":::\n```"));
    }

    #[test]
    fn preprocesses_proof_blocks() {
        let input = r#"
:::proof
Let x = y.
:::

:::proof[Proof of 4]
Custom proof.
:::
"#;
        let out = preprocess_expandables(input);

        assert!(out.contains(r#"**Proof.** <a class="expand-link" data-bs-toggle="collapse" href='#expand-1'>Click to Expand</a>"#));
        assert!(out.contains("Let x = y."));
        assert!(out.contains(r#"**Proof of 4.** <a class="expand-link" data-bs-toggle="collapse" href='#expand-2'>Click to Expand</a>"#));
        assert!(out.contains("Custom proof."));
    }

    #[test]
    fn leaves_proof_marker_inside_code_fence() {
        let input = r#"```markdown
:::proof
body
:::
```
"#;
        let out = preprocess_expandables(input);

        assert!(out.contains(":::proof"));
        assert!(!out.contains(r#"class="collapse""#));
    }
}

#[cfg(test)]
mod test_markdown_card {
    use super::*;

    #[test]
    fn test_preprocess_cards() {
        let input = r#"
:::card[example]
Some code here
More code here
::::
"#;
        let out = preprocess_cards(input);
        assert!(out.contains(r#"<div class="card example">"#));
        assert!(out.contains(r#"Some code here"#));
        assert!(out.contains(r#"More code here"#));
    }

    #[test]
    fn test_preprocess_cards_no_class() {
        let input = r#"
:::card
Some code here
More code here
::::
"#;
        let out = preprocess_cards(input);
        assert!(out.contains(r#"<div class="card ">"#));
        assert!(out.contains(r#"Some code here"#));
        assert!(out.contains(r#"More code here"#));
    }

    #[test]
    fn leaves_card_marker_inside_code_fence() {
        let input = r#"```markdown
:::card[example]
body
:::
```
"#;
        let out = preprocess_cards(input);

        assert!(out.contains(":::card[example]"));
        assert!(!out.contains(r#"<div class="card example">"#));
    }

    #[test]
    fn leaves_closing_marker_inside_card_code_fence() {
        let input = r#":::card[example]
```markdown
:::
```
After code
:::
"#;
        let out = preprocess_cards(input);

        assert!(out.contains("After code"));
        assert!(out.contains(":::\n```"));
    }

    #[test]
    fn preprocesses_semantic_cards() {
        let input = r#"
:::aside
Side note.
:::

:::remark
Remark body.
:::
"#;
        let out = preprocess_semantic_cards(input);

        assert!(out.contains(r#"<aside class="card aside">"#));
        assert!(out.contains("Side note."));
        assert!(out.contains(r#"<aside class="card remark">"#));
        assert!(out.contains("Remark body."));
    }

    #[test]
    fn leaves_semantic_card_marker_inside_code_fence() {
        let input = r#"```markdown
:::aside
body
:::
```
"#;
        let out = preprocess_semantic_cards(input);

        assert!(out.contains(":::aside"));
        assert!(!out.contains(r#"<aside class="card aside">"#));
    }
}

#[cfg(test)]
mod test_markdown_figures {
    use super::*;

    #[test]
    fn preprocesses_image_figures() {
        let input = r#"
:::figure fig.png
alt: Diagram
caption: Important diagram
width: 360
:::
"#;
        let out = preprocess_figures(input);

        assert!(out.contains(r#"<figure class="figure-block""#));
        assert!(out.contains(r#"<img src="fig.png" alt="Diagram" style="width:90%; max-width: 360px; height: auto; margin: 0 auto;">"#));
        assert!(out.contains("<figcaption>Important diagram</figcaption>"));
    }

    #[test]
    fn preprocesses_image_figures_without_forced_width() {
        let input = r#"
:::figure fig.png
alt: Diagram
:::
"#;
        let out = preprocess_figures(input);

        assert!(out.contains(r#"<img src="fig.png" alt="Diagram">"#));
        assert!(!out.contains("max-width"));
    }

    #[test]
    fn preprocesses_container_figures() {
        let input = r#"
:::figure id=fig12 width=360 ratio=4/3 class=jxgbox
:::
"#;
        let out = preprocess_figures(input);

        assert!(out.contains(r#"<div id="fig12" class="jxgbox" style="width:90%; max-width: 360px; aspect-ratio: 4/3; margin: 20px auto;"></div>"#));
        assert!(!out.contains("<figure"));
    }

    #[test]
    fn leaves_figure_marker_inside_code_fence() {
        let input = r#"```markdown
:::figure fig.png
:::
```
"#;
        let out = preprocess_figures(input);

        assert!(out.contains(":::figure fig.png"));
        assert!(!out.contains("<img"));
    }
}
