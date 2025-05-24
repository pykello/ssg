use regex::Regex;

fn extract_class(line: &str) -> Option<String> {
    let re = Regex::new(r#"\[([^"]+)\]"#).unwrap();
    if let Some(caps) = re.captures(line) {
        Some(caps[1].to_string())
    } else {
        None
    }
}

pub fn preprocess_cards(markdown: &str) -> String {
    let mut out = String::new();
    let mut lines = markdown.lines();

    while let Some(line) = lines.next() {
        if line.trim_start().starts_with(":::card") {
            let class = extract_class(line).unwrap_or_else(|| "".to_string());

            out.push_str(&format!(r#"<div class="card {class}">"#, class = class));
            out.push_str("\n");
            out.push_str("\n");
            for body in &mut lines {
                if body.trim_start().starts_with(":::") {
                    break;
                }
                out.push_str(body);
                out.push('\n');
            }

            out.push_str("  </div>\n\n");
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

/// Replace `:::expandable … :::` with Bootstrap‑collapse HTML.
pub fn preprocess_expandables(markdown: &str) -> String {
    let mut out = String::new();
    let mut id_counter = 0;
    let mut lines = markdown.lines();

    while let Some(line) = lines.next() {
        if line.trim_start().starts_with(":::expandable") {
            // ── 1. Parse the heading line ────────────────────────────────
            let heading_line = lines.next().unwrap_or("").trim();
            id_counter += 1;
            let id = format!("expand-{}", id_counter);

            let re = Regex::new(r"\[([^\]]+)\]").unwrap();
            let heading_line = re
                .replace_all(heading_line, |caps: &regex::Captures| {
                    format!(
                        r#"<a class="expand-link" data-bs-toggle="collapse" href='#{id}'>{}</a>"#,
                        &caps[1],
                        id = id
                    )
                })
                .into_owned();

            // ── 2. Emit the toggle + opening wrappers ───────────────────
            out.push_str(&format!(
                r#"{heading_line}

<div class="collapse" id="{id}">
  <div class="card card-body">
"#,
                heading_line = heading_line,
                id = id
            ));

            // ── 3. Copy body lines until closing fence ──────────────────
            for body in &mut lines {
                if body.trim_start().starts_with(":::") {
                    break; // reached terminating fence
                }
                out.push_str(body);
                out.push('\n');
            }

            // ── 4. Close the wrappers ───────────────────────────────────
            out.push_str("  </div>\n</div>\n");
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
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
}
