const PLACEHOLDER_PREFIX: &str = "MATHSEGMENTPLACEHOLDER";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtectedMath {
    markdown: String,
    segments: Vec<String>,
}

impl ProtectedMath {
    pub fn markdown(&self) -> &str {
        &self.markdown
    }

    pub fn restore(&self, html: &str) -> String {
        let mut restored = html.to_string();
        for (idx, segment) in self.segments.iter().enumerate() {
            restored = restored.replace(&placeholder(idx), segment);
        }
        restored
    }
}

pub fn protect_math(markdown: &str, expand_shorthand: bool) -> ProtectedMath {
    let mut parser = MathProtector::new(markdown, expand_shorthand);
    parser.protect();
    ProtectedMath {
        markdown: parser.output,
        segments: parser.segments,
    }
}

fn placeholder(index: usize) -> String {
    format!("{PLACEHOLDER_PREFIX}{index:06}")
}

struct MathProtector<'a> {
    input: &'a str,
    output: String,
    segments: Vec<String>,
    pos: usize,
    expand_shorthand: bool,
}

impl<'a> MathProtector<'a> {
    fn new(input: &'a str, expand_shorthand: bool) -> Self {
        Self {
            input,
            output: String::with_capacity(input.len()),
            segments: Vec::new(),
            pos: 0,
            expand_shorthand,
        }
    }

    fn protect(&mut self) {
        while self.pos < self.input.len() {
            if self.starts_unescaped("$$") {
                if let Some(end) = self.find_math_end("$$", self.pos + 2) {
                    self.push_segment(end + 2);
                    continue;
                }
            } else if self.starts_unescaped("$") {
                if let Some(end) = self.find_math_end("$", self.pos + 1) {
                    self.push_segment(end + 1);
                    continue;
                }
            }

            self.push_next_char();
        }
    }

    fn starts_unescaped(&self, delimiter: &str) -> bool {
        self.input[self.pos..].starts_with(delimiter) && !self.is_escaped(self.pos)
    }

    fn is_escaped(&self, pos: usize) -> bool {
        let mut slash_count = 0;
        for ch in self.input[..pos].chars().rev() {
            if ch == '\\' {
                slash_count += 1;
            } else {
                break;
            }
        }
        slash_count % 2 == 1
    }

    fn find_math_end(&self, delimiter: &str, start: usize) -> Option<usize> {
        let mut search_pos = start;
        while search_pos < self.input.len() {
            let relative = self.input[search_pos..].find(delimiter)?;
            let end = search_pos + relative;
            if !self.is_escaped(end) {
                return Some(end);
            }
            search_pos = end + delimiter.len();
        }
        None
    }

    fn push_segment(&mut self, end: usize) {
        let segment = normalize_math_segment(&self.input[self.pos..end], self.expand_shorthand);
        let placeholder = placeholder(self.segments.len());
        self.output.push_str(&placeholder);
        self.segments.push(segment);
        self.pos = end;
    }

    fn push_next_char(&mut self) {
        let ch = self.input[self.pos..]
            .chars()
            .next()
            .expect("pos is always on a char boundary");
        self.output.push(ch);
        self.pos += ch.len_utf8();
    }
}

fn normalize_math_segment(segment: &str, expand_shorthand: bool) -> String {
    let segment = strip_blockquote_markers(segment);
    let segment = unescape_markdown_operators_in_math(&segment);
    if expand_shorthand {
        expand_math_shorthand(&segment)
    } else {
        segment
    }
}

pub fn preprocess_math_shorthand_blocks(markdown: &str) -> String {
    let mut output = String::with_capacity(markdown.len());
    let mut lines = markdown.lines();
    let mut in_fence = false;

    while let Some(line) = lines.next() {
        if is_fence_line(line) {
            in_fence = !in_fence;
            output.push_str(line);
            output.push('\n');
        } else if !in_fence && line.trim() == ":::align" {
            let mut body = Vec::new();
            for body_line in lines.by_ref() {
                if is_fence_line(body_line) {
                    in_fence = !in_fence;
                }
                if body_line.trim() == ":::" || body_line.trim() == "::::" {
                    break;
                }
                if !body_line.trim().is_empty() {
                    body.push(body_line.trim().to_string());
                }
            }
            output.push_str("$$\n\\begin{align*}\n");
            output.push_str(&join_math_rows(&body));
            output.push_str("\n\\end{align*}\n$$\n");
        } else if !in_fence && line.trim().starts_with(":::cases ") {
            let expr = line
                .trim()
                .strip_prefix(":::cases ")
                .expect("checked cases prefix");
            let mut body = Vec::new();
            for body_line in lines.by_ref() {
                if is_fence_line(body_line) {
                    in_fence = !in_fence;
                }
                if body_line.trim() == ":::" || body_line.trim() == "::::" {
                    break;
                }
                if !body_line.trim().is_empty() {
                    body.push(body_line.trim().to_string());
                }
            }
            output.push_str("$$\n");
            output.push_str(expr.trim());
            output.push_str(" = \\begin{cases}\n");
            output.push_str(&join_case_rows(&body));
            output.push_str("\n\\end{cases}\n$$\n");
        } else {
            output.push_str(line);
            output.push('\n');
        }
    }

    output
}

fn is_fence_line(line: &str) -> bool {
    let line = line.trim_start();
    line.starts_with("```") || line.starts_with("~~~")
}

fn join_math_rows(rows: &[String]) -> String {
    rows.iter()
        .enumerate()
        .map(|(idx, row)| {
            if idx + 1 == rows.len() || row.ends_with(r"\\") {
                row.clone()
            } else {
                format!("{row} \\\\")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn join_case_rows(rows: &[String]) -> String {
    rows.iter()
        .enumerate()
        .map(|(idx, row)| {
            let row = row
                .split_once('|')
                .map(|(value, condition)| format!("{} & {}", value.trim(), condition.trim()))
                .unwrap_or_else(|| row.clone());
            if idx + 1 == rows.len() || row.ends_with(r"\\") {
                row
            } else {
                format!("{row} \\\\")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn strip_blockquote_markers(segment: &str) -> String {
    let mut output = String::with_capacity(segment.len());
    for (idx, line) in segment.split_inclusive('\n').enumerate() {
        if idx == 0 {
            output.push_str(line);
        } else {
            output.push_str(strip_blockquote_marker(line));
        }
    }
    output
}

fn strip_blockquote_marker(line: &str) -> &str {
    let leading_spaces = line
        .char_indices()
        .take_while(|(_, ch)| *ch == ' ')
        .last()
        .map_or(0, |(idx, _)| idx + 1);
    let rest = &line[leading_spaces..];

    if let Some(after_marker) = rest.strip_prefix('>') {
        let after_marker = after_marker.strip_prefix(' ').unwrap_or(after_marker);
        &after_marker[..]
    } else {
        line
    }
}

fn unescape_markdown_operators_in_math(segment: &str) -> String {
    let mut output = String::with_capacity(segment.len());
    let mut chars = segment.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.peek() {
                Some('+') | Some('=') => {
                    output.push(chars.next().expect("peeked char exists"));
                }
                _ => output.push(ch),
            }
        } else {
            output.push(ch);
        }
    }

    output
}

fn expand_math_shorthand(segment: &str) -> String {
    let mut output = segment.to_string();

    for (from, to) in [
        ("<=>", r"\Leftrightarrow"),
        ("=>", r"\implies"),
        ("->", r"\to"),
        ("!=", r"\ne"),
        ("<=", r"\le"),
        (">=", r"\ge"),
    ] {
        output = output.replace(from, to);
    }

    for (name, open, close) in [
        ("norm", r"\left\lVert ", r" \right\rVert"),
        ("abs", r"\left\lvert ", r" \right\rvert"),
        ("unit", r"\hat{\mathbf{", "}}"),
        ("v", r"\mathbf{", "}"),
        ("bb", r"\mathbb{", "}"),
        ("cal", r"\mathcal{", "}"),
        ("hat", r"\hat{", "}"),
    ] {
        output = expand_wrapped_function(&output, name, open, close);
    }

    output = expand_set(&output);
    output = expand_lim(&output);
    output = expand_plain_delimiters(&output);

    for (from, to) in [("forall", r"\forall"), ("exists", r"\exists")] {
        output = replace_word(&output, from, to);
    }
    for (from, to) in [("eps", r"\epsilon"), ("del", r"\delta"), ("inf", r"\infty")] {
        output = replace_symbol_prefix(&output, from, to);
    }

    output
}

fn expand_set(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut pos = 0;
    let brace_pattern = "set{";
    let paren_pattern = "set(";

    while pos < input.len() {
        let rest = &input[pos..];
        if starts_identifier_function(input, pos, brace_pattern) {
            if let Some(end) = find_matching(input, pos + brace_pattern.len() - 1, '{', '}') {
                output.push_str(r"\left\{");
                output.push_str(&format_set_content(&input[pos + brace_pattern.len()..end]));
                output.push_str(r"\right\}");
                pos = end + 1;
                continue;
            }
        } else if starts_identifier_function(input, pos, paren_pattern) {
            if let Some(end) = find_matching(input, pos + paren_pattern.len() - 1, '(', ')') {
                output.push_str(r"\left\{");
                output.push_str(&format_set_content(&input[pos + paren_pattern.len()..end]));
                output.push_str(r"\right\}");
                pos = end + 1;
                continue;
            }
        }

        let ch = rest.chars().next().expect("pos is on a char boundary");
        output.push(ch);
        pos += ch.len_utf8();
    }

    output
}

fn format_set_content(content: &str) -> String {
    if let Some(split) = find_top_level_char(content, '|') {
        let (value, condition) = content.split_at(split);
        format!("{} \\;\\middle|\\; {}", value.trim(), condition[1..].trim())
    } else {
        content.to_string()
    }
}

fn find_top_level_char(input: &str, target: char) -> Option<usize> {
    let mut paren_depth = 0;
    let mut brace_depth = 0;
    let mut bracket_depth = 0;

    for (idx, ch) in input.char_indices() {
        if is_escaped(input, idx) {
            continue;
        }
        match ch {
            '(' => paren_depth += 1,
            ')' if paren_depth > 0 => paren_depth -= 1,
            '{' => brace_depth += 1,
            '}' if brace_depth > 0 => brace_depth -= 1,
            '[' => bracket_depth += 1,
            ']' if bracket_depth > 0 => bracket_depth -= 1,
            _ if ch == target && paren_depth == 0 && brace_depth == 0 && bracket_depth == 0 => {
                return Some(idx);
            }
            _ => {}
        }
    }

    None
}

fn expand_plain_delimiters(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut pos = 0;
    let mut paren_depth = 0;
    let mut bracket_depth = 0;

    while pos < input.len() {
        let ch = input[pos..]
            .chars()
            .next()
            .expect("pos is on a char boundary");
        if ch == '(' && !is_escaped(input, pos) && !output.trim_end().ends_with(r"\left") {
            output.push_str(r"\left(");
            paren_depth += 1;
        } else if ch == ')' && !is_escaped(input, pos) && !output.trim_end().ends_with(r"\right") {
            if paren_depth > 0 {
                output.push_str(r"\right)");
                paren_depth -= 1;
            } else {
                output.push(ch);
            }
        } else if ch == '['
            && !is_escaped(input, pos)
            && !output.trim_end().ends_with(r"\left")
            && !output.trim_end().ends_with(r"\\")
            && !previous_output_token_is_latex_command(&output)
        {
            output.push_str(r"\left[");
            bracket_depth += 1;
        } else if ch == ']' && !is_escaped(input, pos) && !output.trim_end().ends_with(r"\right") {
            if bracket_depth > 0 {
                output.push_str(r"\right]");
                bracket_depth -= 1;
            } else {
                output.push(ch);
            }
        } else {
            output.push(ch);
        }
        pos += ch.len_utf8();
    }

    output
}

fn previous_output_token_is_latex_command(output: &str) -> bool {
    let trimmed = output.trim_end();
    let mut command_len = 0;
    for ch in trimmed.chars().rev() {
        if ch.is_ascii_alphabetic() {
            command_len += 1;
        } else {
            break;
        }
    }
    if command_len == 0 || command_len == trimmed.len() {
        return false;
    }
    trimmed[..trimmed.len() - command_len].ends_with('\\')
}

fn expand_wrapped_function(input: &str, name: &str, open: &str, close: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut pos = 0;
    let brace_pattern = format!("{name}{{");
    let paren_pattern = format!("{name}(");

    while pos < input.len() {
        let rest = &input[pos..];
        if starts_identifier_function(input, pos, &brace_pattern) {
            if let Some(end) = find_matching(input, pos + brace_pattern.len() - 1, '{', '}') {
                output.push_str(open);
                output.push_str(&input[pos + brace_pattern.len()..end]);
                output.push_str(close);
                pos = end + 1;
                continue;
            }
        } else if starts_identifier_function(input, pos, &paren_pattern) {
            if let Some(end) = find_matching(input, pos + paren_pattern.len() - 1, '(', ')') {
                output.push_str(open);
                output.push_str(&input[pos + paren_pattern.len()..end]);
                output.push_str(close);
                pos = end + 1;
                continue;
            }
        }

        let ch = rest.chars().next().expect("pos is on a char boundary");
        output.push(ch);
        pos += ch.len_utf8();
    }

    output
}

fn expand_lim(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut pos = 0;
    let pattern = "lim[";

    while pos < input.len() {
        if starts_identifier_function(input, pos, pattern) {
            if let Some(end) = find_matching(input, pos + pattern.len() - 1, '[', ']') {
                output.push_str(r"\lim_{");
                output.push_str(&input[pos + pattern.len()..end]);
                output.push('}');
                pos = end + 1;
                continue;
            }
        }

        let ch = input[pos..]
            .chars()
            .next()
            .expect("pos is on a char boundary");
        output.push(ch);
        pos += ch.len_utf8();
    }

    output
}

fn starts_identifier_function(input: &str, pos: usize, pattern: &str) -> bool {
    input[pos..].starts_with(pattern)
        && !is_escaped(input, pos)
        && !previous_char(input, pos).is_some_and(is_identifier_char)
}

fn is_escaped(input: &str, pos: usize) -> bool {
    let mut slash_count = 0;
    for ch in input[..pos].chars().rev() {
        if ch == '\\' {
            slash_count += 1;
        } else {
            break;
        }
    }
    slash_count % 2 == 1
}

fn previous_char(input: &str, pos: usize) -> Option<char> {
    input[..pos].chars().next_back()
}

fn is_identifier_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

fn find_matching(input: &str, open_pos: usize, open: char, close: char) -> Option<usize> {
    let mut depth = 0;
    for (offset, ch) in input[open_pos..].char_indices() {
        if ch == open {
            depth += 1;
        } else if ch == close {
            depth -= 1;
            if depth == 0 {
                return Some(open_pos + offset);
            }
        }
    }
    None
}

fn replace_word(input: &str, from: &str, to: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut pos = 0;

    while pos < input.len() {
        if input[pos..].starts_with(from)
            && !is_escaped(input, pos)
            && !previous_char(input, pos).is_some_and(is_identifier_char)
            && !input[pos + from.len()..]
                .chars()
                .next()
                .is_some_and(is_identifier_char)
        {
            output.push_str(to);
            pos += from.len();
        } else {
            let ch = input[pos..]
                .chars()
                .next()
                .expect("pos is on a char boundary");
            output.push(ch);
            pos += ch.len_utf8();
        }
    }

    output
}

fn replace_symbol_prefix(input: &str, from: &str, to: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut pos = 0;

    while pos < input.len() {
        if input[pos..].starts_with(from)
            && !is_escaped(input, pos)
            && !previous_char(input, pos).is_some_and(is_identifier_char)
            && !input[pos + from.len()..]
                .chars()
                .next()
                .is_some_and(|ch| ch.is_ascii_alphanumeric())
        {
            output.push_str(to);
            pos += from.len();
        } else {
            let ch = input[pos..]
                .chars()
                .next()
                .expect("pos is on a char boundary");
            output.push(ch);
            pos += ch.len_utf8();
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protects_inline_math() {
        let protected = protect_math(r"A $x + y$ and **bold**", false);

        assert_eq!(
            protected.markdown(),
            "A MATHSEGMENTPLACEHOLDER000000 and **bold**"
        );
        assert_eq!(
            protected.restore("<p>A MATHSEGMENTPLACEHOLDER000000 and <strong>bold</strong></p>\n"),
            "<p>A $x + y$ and <strong>bold</strong></p>\n"
        );
    }

    #[test]
    fn keeps_escaped_dollar_literals() {
        let protected = protect_math(r"This costs \$5 and $x$", false);

        assert_eq!(
            protected.markdown(),
            r"This costs \$5 and MATHSEGMENTPLACEHOLDER000000"
        );
        assert_eq!(
            protected.restore("<p>This costs $5 and MATHSEGMENTPLACEHOLDER000000</p>\n"),
            "<p>This costs $5 and $x$</p>\n"
        );
    }

    #[test]
    fn unescapes_markdown_operators_inside_math() {
        let protected = protect_math(
            r"$$
a \= b \+ c
$$",
            false,
        );

        assert_eq!(protected.segments[0], "$$\na = b + c\n$$");
    }

    #[test]
    fn strips_blockquote_markers_from_display_math_segments() {
        let protected = protect_math(
            r#"> [!NOTE]
> Intro.
>
> $$
> f(x) = \begin{cases}
> 1 & x \ne 0 \\
> 0 & x = 0
> \end{cases}
> $$
>
> Done."#,
            false,
        );

        assert!(protected
            .markdown()
            .contains("> MATHSEGMENTPLACEHOLDER000000"));
        assert_eq!(
            protected.segments[0],
            r#"$$
f(x) = \begin{cases}
1 & x \ne 0 \\
0 & x = 0
\end{cases}
$$"#
        );
    }

    #[test]
    fn expands_math_shorthand_inside_math_segments() {
        let protected = protect_math(
            r"$norm(v{x} - v{y}) <= eps => lim[x -> 0] (f(x) + 1) != inf$",
            true,
        );

        assert_eq!(
            protected.segments[0],
            r"$\left\lVert \mathbf{x} - \mathbf{y} \right\rVert \le \epsilon \implies \lim_{x \to 0} \left(f\left(x\right) + 1\right) \ne \infty$"
        );
    }

    #[test]
    fn does_not_double_existing_left_right_parentheses() {
        let protected = protect_math(r"$\left(x + y\right) + (a + b)$", true);

        assert_eq!(
            protected.segments[0],
            r"$\left(x + y\right) + \left(a + b\right)$"
        );
    }

    #[test]
    fn expands_additional_math_shorthand_forms() {
        let protected = protect_math(
            r"$A[0] + \sqrt[n] + \\[1em] + unit{n} + eps_0 + del_a + inf_n + set(v{x} in bb{R} | norm(v{x}) <= 1)$",
            true,
        );

        assert_eq!(
            protected.segments[0],
            r"$A\left[0\right] + \sqrt[n] + \\[1em] + \hat{\mathbf{n}} + \epsilon_0 + \delta_a + \infty_n + \left\{\mathbf{x} in \mathbb{R} \;\middle|\; \left\lVert \mathbf{x} \right\rVert \le 1\right\}$"
        );
    }

    #[test]
    fn leaves_shorthand_untouched_when_disabled() {
        let protected = protect_math(r"$norm(v{x}) <= eps$", false);

        assert_eq!(protected.segments[0], r"$norm(v{x}) <= eps$");
    }

    #[test]
    fn does_not_rewrite_escaped_or_embedded_words() {
        let protected = protect_math(r"$\norm(v{x}) + epsilon + myeps + eps$", true);

        assert_eq!(
            protected.segments[0],
            r"$\norm\left(\mathbf{x}\right) + epsilon + myeps + \epsilon$"
        );
    }

    #[test]
    fn preprocesses_align_blocks() {
        let markdown = preprocess_math_shorthand_blocks(
            r#"before
:::align
a &= b
&= c
:::
after"#,
        );

        assert_eq!(
            markdown,
            "before\n$$\n\\begin{align*}\na &= b \\\\\n&= c\n\\end{align*}\n$$\nafter\n"
        );
    }

    #[test]
    fn preprocesses_cases_blocks() {
        let markdown = preprocess_math_shorthand_blocks(
            r#":::cases f(x)
1 | x != 0
0 | x = 0
:::"#,
        );

        assert_eq!(
            markdown,
            "$$\nf(x) = \\begin{cases}\n1 & x != 0 \\\\\n0 & x = 0\n\\end{cases}\n$$\n"
        );
    }

    #[test]
    fn leaves_shorthand_blocks_inside_code_fences() {
        let markdown = preprocess_math_shorthand_blocks(
            r#"```text
:::align
a &= b
:::
```"#,
        );

        assert_eq!(markdown, "```text\n:::align\na &= b\n:::\n```\n");
    }
}
