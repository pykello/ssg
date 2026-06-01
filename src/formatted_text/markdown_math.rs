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

pub fn protect_math(markdown: &str) -> ProtectedMath {
    let mut parser = MathProtector::new(markdown);
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
}

impl<'a> MathProtector<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            output: String::with_capacity(input.len()),
            segments: Vec::new(),
            pos: 0,
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
        let segment = normalize_math_segment(&self.input[self.pos..end]);
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

fn normalize_math_segment(segment: &str) -> String {
    let segment = strip_blockquote_markers(segment);
    unescape_markdown_operators_in_math(&segment)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protects_inline_math() {
        let protected = protect_math(r"A $x + y$ and **bold**");

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
        let protected = protect_math(r"This costs \$5 and $x$");

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
}
