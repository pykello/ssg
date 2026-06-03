use regex::Regex;
use std::sync::OnceLock;

static CARD_CLASS_RE: OnceLock<Regex> = OnceLock::new();
static EXPAND_LINK_RE: OnceLock<Regex> = OnceLock::new();

fn card_class_regex() -> &'static Regex {
    CARD_CLASS_RE.get_or_init(|| Regex::new(r#"\[([^"]+)\]"#).expect("valid card class regex"))
}

fn expand_link_regex() -> &'static Regex {
    EXPAND_LINK_RE.get_or_init(|| Regex::new(r"\[([^\]]+)\]").expect("valid expand link regex"))
}

fn extract_class(line: &str) -> Option<String> {
    card_class_regex()
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

fn is_directive_close(line: &str) -> bool {
    line.trim_start().starts_with(":::")
}

fn copy_directive_body<'a>(lines: &mut impl Iterator<Item = &'a str>, out: &mut String) {
    let mut in_fence = false;
    for body in lines {
        if is_fence_line(body) {
            in_fence = !in_fence;
        }
        if !in_fence && is_directive_close(body) {
            break;
        }
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
        } else if !in_fence && line.trim_start().starts_with(":::card") {
            let class = extract_class(line).unwrap_or_default();

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

pub fn preprocess_expandables(markdown: &str) -> String {
    let mut out = String::new();
    let mut id_counter = 0;
    let mut lines = markdown.lines();
    let mut in_fence = false;

    while let Some(line) = lines.next() {
        if is_fence_line(line) {
            in_fence = !in_fence;
            append_line(&mut out, line);
        } else if !in_fence && line.trim_start().starts_with(":::expandable") {
            let heading_line = lines.next().unwrap_or("").trim();
            id_counter += 1;
            let id = format!("expand-{}", id_counter);
            let heading_line = render_expandable_heading(heading_line, &id);

            out.push_str(&format!(
                r#"{heading_line}

<div class="collapse" id="{id}">
  <div class="card card-body">
"#,
                heading_line = heading_line,
                id = id
            ));

            copy_directive_body(&mut lines, &mut out);
            out.push_str("  </div>\n</div>\n");
        } else {
            append_line(&mut out, line);
        }
    }
    out
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
}
