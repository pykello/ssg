const PLACEHOLDER_PREFIX: &str = "MATHSEGMENTPLACEHOLDER";

type Replacement = (&'static str, &'static str);
type WrappedFunction = (&'static str, &'static str, &'static str);

const OPERATOR_REPLACEMENTS: &[Replacement] = &[
    ("...", r"\ldots"),
    ("<=>", r"\Leftrightarrow"),
    ("=>", r"\implies"),
    ("->", r"\to"),
    ("!=", r"\ne"),
    ("<=", r"\le"),
    (">=", r"\ge"),
];

const WRAPPED_FUNCTIONS: &[WrappedFunction] = &[
    ("norm", r"\left\lVert ", r" \right\rVert"),
    ("abs", r"\left\lvert ", r" \right\rvert"),
    ("unit", r"\hat{\mathbf{", "}}"),
    ("v", r"\mathbf{", "}"),
    ("bb", r"\mathbb{", "}"),
    ("cal", r"\mathcal{", "}"),
    ("hat", r"\hat{", "}"),
];

const WORD_REPLACEMENTS: &[Replacement] = &[
    ("forall", r"\forall"),
    ("exists", r"\exists"),
    ("notin", r"\notin"),
    ("in", r"\in"),
    ("dot", r"\cdot"),
    ("cross", r"\times"),
];

const GREEK_REPLACEMENTS: &[Replacement] = &[
    ("alpha", r"\alpha"),
    ("beta", r"\beta"),
    ("gamma", r"\gamma"),
    ("delta", r"\delta"),
    ("eta", r"\eta"),
    ("theta", r"\theta"),
    ("lambda", r"\lambda"),
    ("mu", r"\mu"),
    ("xi", r"\xi"),
    ("pi", r"\pi"),
    ("rho", r"\rho"),
    ("sigma", r"\sigma"),
    ("tau", r"\tau"),
    ("phi", r"\phi"),
    ("psi", r"\psi"),
    ("omega", r"\omega"),
    ("Delta", r"\Delta"),
    ("Gamma", r"\Gamma"),
    ("Omega", r"\Omega"),
    ("Phi", r"\Phi"),
    ("Psi", r"\Psi"),
];

const SYMBOL_PREFIX_REPLACEMENTS: &[Replacement] =
    &[("eps", r"\epsilon"), ("del", r"\delta"), ("inf", r"\infty")];

const RELATION_TOKENS: &[&str] = &["<=>", "=>", "->", "!=", "<=", ">=", "=", "<", ">"];

const INDEXED_OPERATORS: &[(&str, &str)] = &[
    ("sum", r"\sum"),
    ("prod", r"\prod"),
    ("lim", r"\lim"),
    ("sup", r"\sup"),
    ("inf", r"\inf"),
    ("max", r"\max"),
    ("min", r"\min"),
    ("union", r"\bigcup"),
    ("inter", r"\bigcap"),
];

const INTEGRAL_OPERATORS: &[(&str, &str)] = &[
    ("iiint", r"\iiint"),
    ("iint", r"\iint"),
    ("oint", r"\oint"),
    ("int", r"\int"),
];

#[derive(Debug, Clone, Copy)]
struct FunctionCall {
    content_start: usize,
    content_end: usize,
    next_pos: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MathBlockMode {
    Auto,
    Plain,
    Align,
    System,
    Matrix,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MathBlockConfig {
    mode: MathBlockMode,
    tag: Option<String>,
}

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
            append_markdown_line(&mut output, line);
        } else if !in_fence {
            if let Some(config) = parse_math_block(line) {
                let body = collect_shorthand_block_body(&mut lines, &mut in_fence);
                write_math_block(&mut output, &config, &body);
            } else {
                append_markdown_line(&mut output, line);
            }
        } else {
            append_markdown_line(&mut output, line);
        }
    }

    output
}

fn append_markdown_line(output: &mut String, line: &str) {
    output.push_str(line);
    output.push('\n');
}

fn is_fence_line(line: &str) -> bool {
    let line = line.trim_start();
    line.starts_with("```") || line.starts_with("~~~")
}

fn parse_math_block(line: &str) -> Option<MathBlockConfig> {
    let trimmed_line = line.trim();
    let args = if trimmed_line == ":::math" {
        ""
    } else {
        trimmed_line.strip_prefix(":::math ")?.trim()
    };

    let mut config = MathBlockConfig {
        mode: MathBlockMode::Auto,
        tag: None,
    };

    for token in args.split_whitespace() {
        match token {
            "auto" => config.mode = MathBlockMode::Auto,
            "plain" => config.mode = MathBlockMode::Plain,
            "align" => config.mode = MathBlockMode::Align,
            "system" => config.mode = MathBlockMode::System,
            "matrix" => config.mode = MathBlockMode::Matrix,
            _ => {
                if let Some(tag) = token.strip_prefix("tag=") {
                    config.tag = Some(tag.to_string());
                } else {
                    return None;
                }
            }
        }
    }

    Some(config)
}

fn collect_shorthand_block_body<'a>(
    lines: &mut impl Iterator<Item = &'a str>,
    in_fence: &mut bool,
) -> Vec<String> {
    let mut body = Vec::new();

    for body_line in lines.by_ref() {
        if is_fence_line(body_line) {
            *in_fence = !*in_fence;
        }

        let trimmed = body_line.trim();
        if is_shorthand_block_close(trimmed) {
            break;
        }
        if !trimmed.is_empty() {
            body.push(trimmed.to_string());
        }
    }

    body
}

fn is_shorthand_block_close(trimmed_line: &str) -> bool {
    trimmed_line == ":::" || trimmed_line == "::::"
}

fn write_math_block(output: &mut String, config: &MathBlockConfig, body: &[String]) {
    let (body, inline_tag) = extract_math_block_tag(body);
    let tag = config.tag.as_deref().or(inline_tag.as_deref());

    if config.mode == MathBlockMode::System {
        write_system_math_block(output, &body, tag);
        return;
    }
    if config.mode == MathBlockMode::Matrix {
        write_matrix_math_block(output, &body, tag);
        return;
    }

    if let Some((prefix, rows)) = parse_block_cases(&body) {
        output.push_str("$$\n");
        if !prefix.is_empty() {
            output.push_str(&prefix);
            output.push(' ');
        }
        output.push_str("\\begin{cases}\n");
        output.push_str(&join_case_rows(rows));
        output.push_str("\n\\end{cases}");
        push_optional_tag(output, tag);
        output.push_str("\n$$\n");
        return;
    }

    match config.mode {
        MathBlockMode::Plain => write_plain_math_block(output, &body, tag),
        MathBlockMode::Align => write_aligned_math_block(output, &body, tag),
        MathBlockMode::Auto => {
            let (rows, should_align) = auto_align_rows(&body);
            if should_align {
                write_aligned_rows(output, &rows, tag);
            } else {
                write_plain_math_block(output, &body, tag);
            }
        }
        MathBlockMode::System | MathBlockMode::Matrix => unreachable!("handled above"),
    }
}

fn extract_math_block_tag(rows: &[String]) -> (Vec<String>, Option<String>) {
    let mut tag = None;
    let mut rows = rows.to_vec();

    for row in &mut rows {
        if let Some(pos) = find_top_level_token(row, "#tag") {
            let extracted = row[pos + "#tag".len()..].trim();
            if !extracted.is_empty() {
                tag = Some(extracted.to_string());
                *row = row[..pos].trim_end().to_string();
                break;
            }
        }
    }

    rows.retain(|row| !row.trim().is_empty());
    (rows, tag)
}

fn parse_block_cases(rows: &[String]) -> Option<(String, &[String])> {
    let first = rows.first()?;
    let marker = find_top_level_token(first, "cases:")?;
    if rows.len() < 2 {
        return None;
    }

    Some((first[..marker].trim_end().to_string(), &rows[1..]))
}

fn write_plain_math_block(output: &mut String, rows: &[String], tag: Option<&str>) {
    output.push_str("$$\n");
    output.push_str(&rows.join("\n"));
    push_optional_tag(output, tag);
    output.push_str("\n$$\n");
}

fn write_aligned_math_block(output: &mut String, rows: &[String], tag: Option<&str>) {
    let (rows, _) = auto_align_rows(rows);
    write_aligned_rows(output, &rows, tag);
}

fn write_aligned_rows(output: &mut String, rows: &[String], tag: Option<&str>) {
    output.push_str("$$\n\\begin{aligned}\n");
    output.push_str(&join_math_rows(rows));
    push_optional_tag(output, tag);
    output.push_str("\n\\end{aligned}\n$$\n");
}

fn write_system_math_block(output: &mut String, rows: &[String], tag: Option<&str>) {
    let (rows, _) = auto_align_rows(rows);
    output.push_str("$$\n\\left\\{\\begin{aligned}\n");
    output.push_str(&join_math_rows(&rows));
    push_optional_tag(output, tag);
    output.push_str("\n\\end{aligned}\\right.\n$$\n");
}

fn write_matrix_math_block(output: &mut String, rows: &[String], tag: Option<&str>) {
    output.push_str("$$\n\\begin{bmatrix}\n");
    output.push_str(&format_matrix_rows(rows));
    output.push_str("\n\\end{bmatrix}");
    push_optional_tag(output, tag);
    output.push_str("\n$$\n");
}

fn push_optional_tag(output: &mut String, tag: Option<&str>) {
    if let Some(tag) = tag {
        output.push_str(" \\tag{");
        output.push_str(tag);
        output.push('}');
    }
}

fn auto_align_rows(rows: &[String]) -> (Vec<String>, bool) {
    let mut should_align = false;
    let rows: Vec<String> = rows
        .iter()
        .map(|row| {
            let (row, aligned) = auto_align_row(row);
            should_align |= aligned;
            row
        })
        .collect();

    let should_align = should_align && rows.len() > 1;
    (rows, should_align)
}

fn auto_align_row(row: &str) -> (String, bool) {
    if find_top_level_char(row, '&').is_some() {
        return (row.to_string(), true);
    }

    if let Some(pos) = find_top_level_relation(row) {
        let (prefix, suffix) = row.split_at(pos);
        (format!("{prefix}&{suffix}"), true)
    } else {
        (row.to_string(), false)
    }
}

fn find_top_level_relation(input: &str) -> Option<usize> {
    find_top_level_token_from(input, RELATION_TOKENS)
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
            let row = format_case_row(row).unwrap_or_else(|| row.clone());
            if idx + 1 == rows.len() || row.ends_with(r"\\") {
                row
            } else {
                format!("{row} \\\\")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_matrix_rows(rows: &[String]) -> String {
    rows.iter()
        .map(|row| {
            split_top_level_char(row, ',')
                .into_iter()
                .map(str::trim)
                .collect::<Vec<_>>()
                .join(" & ")
        })
        .collect::<Vec<_>>()
        .join(" \\\\\n")
}

fn format_case_row(row: &str) -> Option<String> {
    let split = find_top_level_char(row, '|')?;
    let (value, condition) = row.split_at(split);
    Some(format!("{} & {}", value.trim(), condition[1..].trim()))
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
        after_marker
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

    output = expand_cases(&output);
    output = expand_indexed_operators(&output);
    output = expand_integrals(&output);
    output = expand_derivatives(&output);
    output = expand_norm_variants(&output);
    output = expand_vector_helpers(&output);
    output = expand_topology_helpers(&output);
    output = expand_matrix_helpers(&output);
    output = expand_form_helpers(&output);

    for &(from, to) in OPERATOR_REPLACEMENTS {
        output = output.replace(from, to);
    }

    for &function in WRAPPED_FUNCTIONS {
        output = expand_wrapped_function(&output, function);
    }

    output = expand_set(&output);
    output = expand_lim(&output);
    output = expand_plain_delimiters(&output);

    for &(from, to) in WORD_REPLACEMENTS {
        output = replace_word(&output, from, to);
    }
    for &(from, to) in GREEK_REPLACEMENTS {
        output = replace_word(&output, from, to);
    }
    for &(from, to) in SYMBOL_PREFIX_REPLACEMENTS {
        output = replace_symbol_prefix(&output, from, to);
    }

    output
}

fn expand_cases(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut pos = 0;
    let pattern = "cases(";

    while pos < input.len() {
        if let Some(call) = find_function_call(input, pos, pattern, '(', ')') {
            if let Some(rows) =
                format_inline_cases_content(&input[call.content_start..call.content_end])
            {
                output.push_str(r"\begin{cases}");
                output.push_str(&rows);
                output.push_str(r"\end{cases}");
                pos = call.next_pos;
                continue;
            }
        }

        copy_current_char(input, &mut output, &mut pos);
    }

    output
}

fn format_inline_cases_content(content: &str) -> Option<String> {
    let rows = split_top_level_char(content, ';')
        .into_iter()
        .map(|arm| format_case_row(arm.trim()))
        .collect::<Option<Vec<_>>>()?;

    if rows.is_empty() {
        None
    } else {
        Some(join_math_rows(&rows))
    }
}

fn expand_indexed_operators(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut pos = 0;

    while pos < input.len() {
        if let Some((call, latex)) = find_named_bracket_call(input, pos, INDEXED_OPERATORS) {
            let binder = &input[call.content_start..call.content_end];
            output.push_str(latex);
            output.push_str(&format_operator_binder(binder));

            if let Some((body, next_pos)) = following_paren_content(input, call.next_pos) {
                output.push(' ');
                output.push_str(body);
                pos = next_pos;
            } else {
                pos = call.next_pos;
            }
            continue;
        }

        copy_current_char(input, &mut output, &mut pos);
    }

    output
}

fn expand_integrals(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut pos = 0;

    while pos < input.len() {
        if let Some((call, latex)) = find_named_bracket_call(input, pos, INTEGRAL_OPERATORS) {
            output.push_str(latex);
            output.push_str(&format_integral_bound(
                &input[call.content_start..call.content_end],
            ));

            if let Some((body, next_pos)) = following_paren_content(input, call.next_pos) {
                output.push(' ');
                output.push_str(&format_integral_body(body));
                pos = next_pos;
            } else {
                pos = call.next_pos;
            }
            continue;
        }

        copy_current_char(input, &mut output, &mut pos);
    }

    output
}

fn expand_derivatives(input: &str) -> String {
    let output = expand_indexed_derivative(input, "dd", "d");
    let output = expand_indexed_derivative(&output, "pd", r"\partial");
    let output = expand_paren_functions(&output, &["pd2"], |_, content| {
        let args = split_top_level_char(content, ',');
        if args.len() == 3 {
            Some(format!(
                r"\frac{{\partial^2 {}}}{{\partial {} \partial {}}}",
                args[0].trim(),
                args[1].trim(),
                args[2].trim()
            ))
        } else {
            None
        }
    });
    let output = expand_plain_derivative(&output, "dd", "d");
    let output = expand_plain_derivative(&output, "pd", r"\partial");
    expand_paren_functions(
        &output,
        &["grad", "curl", "div", "hess", "jac"],
        |name, content| {
            let arg = content.trim();
            if arg.is_empty() {
                return None;
            }
            Some(match name {
                "grad" => format!(r"\nabla {arg}"),
                "curl" => format!(r"\operatorname{{curl}} {arg}"),
                "div" => format!(r"\operatorname{{div}} {arg}"),
                "hess" => format!("H_{{{arg}}}"),
                "jac" => format!("J_{{{arg}}}"),
                _ => unreachable!(),
            })
        },
    )
}

fn expand_norm_variants(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut pos = 0;

    while pos < input.len() {
        if let Some(call) = find_function_call(input, pos, "norm[", '[', ']') {
            if let Some((body, next_pos)) = following_paren_content(input, call.next_pos) {
                output.push_str(r"\left\lVert ");
                output.push_str(body);
                output.push_str(r" \right\rVert_{");
                output.push_str(&input[call.content_start..call.content_end]);
                output.push('}');
                pos = next_pos;
                continue;
            }
        }

        copy_current_char(input, &mut output, &mut pos);
    }

    output
}

fn expand_vector_helpers(input: &str) -> String {
    expand_paren_functions(
        input,
        &["tuple", "cross", "dist", "dot", "seq", "ip"],
        |name, content| {
            let args = split_top_level_char(content, ',');
            Some(match name {
                "ip" if args.len() == 2 => {
                    format!(r"\left\langle {}, {} \right\rangle", args[0], args[1])
                }
                "dot" if args.len() == 2 => format!("{} \\cdot {}", args[0], args[1]),
                "cross" if args.len() == 2 => format!("{} \\times {}", args[0], args[1]),
                "dist" if args.len() == 2 => format!("d({}, {})", args[0], args[1]),
                "tuple" => format!("({})", args.join(", ")),
                "seq" if args.len() == 1 => format!(r"\{{{}\}}_{{n \ge 1}}", args[0]),
                _ => return None,
            })
        },
    )
}

fn expand_topology_helpers(input: &str) -> String {
    expand_paren_functions(
        input,
        &[
            "closedball",
            "openball",
            "interior",
            "ball",
            "comp",
            "pre",
            "img",
            "bd",
            "cl",
        ],
        |name, content| {
            let args = split_top_level_char(content, ',');
            Some(match name {
                "img" if args.len() == 2 => format!("{}({})", args[0], args[1]),
                "pre" if args.len() == 2 => format!("{}^{{-1}}({})", args[0], args[1]),
                "comp" if args.len() == 1 => format!("{{{}}}^c", args[0]),
                "cl" if args.len() == 1 => format!(r"\overline{{{}}}", args[0]),
                "interior" if args.len() == 1 => format!("{{{}}}^\\circ", args[0]),
                "bd" if args.len() == 1 => format!(r"\partial {}", args[0]),
                "ball" | "openball" if args.len() == 2 => {
                    format!("B_{{{}}}({})", args[1], args[0])
                }
                "closedball" if args.len() == 2 => {
                    format!(r"\overline{{B}}_{{{}}}({})", args[1], args[0])
                }
                _ => return None,
            })
        },
    )
}

fn expand_matrix_helpers(input: &str) -> String {
    expand_paren_functions(
        input,
        &["detmat", "pmat", "bmat", "mat"],
        |name, content| {
            let rows = split_top_level_char(content, ';')
                .into_iter()
                .map(str::to_string)
                .collect::<Vec<_>>();
            let env = match name {
                "pmat" => "pmatrix",
                "detmat" => "vmatrix",
                "mat" | "bmat" => "bmatrix",
                _ => unreachable!(),
            };
            Some(format!(
                "\\begin{{{env}}}{}\\end{{{env}}}",
                format_matrix_rows(&rows)
            ))
        },
    )
}

fn expand_form_helpers(input: &str) -> String {
    expand_paren_functions(
        input,
        &["boundary", "wedge", "chain", "form", "pull", "ext"],
        |name, content| {
            let args = split_top_level_char(content, ',');
            Some(match name {
                "wedge" if !args.is_empty() => args.join(r" \wedge "),
                "ext" if args.len() == 1 => format!("d{{{}}}", args[0]),
                "pull" if args.len() == 2 => format!("{}^*{}", args[0], args[1]),
                "form" if args.len() == 1 => format!(r"\lambda_{{{}}}", args[0]),
                "boundary" if args.len() == 1 => format!(r"\partial {}", args[0]),
                "chain" if args.len() == 1 => args[0].to_string(),
                _ => return None,
            })
        },
    )
}

fn find_named_bracket_call(
    input: &str,
    pos: usize,
    names: &[(&'static str, &'static str)],
) -> Option<(FunctionCall, &'static str)> {
    for &(name, latex) in names {
        let pattern = format!("{name}[");
        if let Some(call) = find_function_call(input, pos, &pattern, '[', ']') {
            return Some((call, latex));
        }
    }
    None
}

fn following_paren_content(input: &str, pos: usize) -> Option<(&str, usize)> {
    if pos >= input.len() || !input[pos..].starts_with('(') || is_escaped(input, pos) {
        return None;
    }
    let end = find_matching(input, pos, '(', ')')?;
    Some((&input[pos + 1..end], end + 1))
}

fn format_operator_binder(binder: &str) -> String {
    let binder = binder.trim();
    if binder.is_empty() {
        return String::new();
    }

    if let Some(split) = find_top_level_token(binder, "..") {
        let (lower, upper) = binder.split_at(split);
        return format!("_{{{}}}^{{{}}}", lower.trim(), upper[2..].trim());
    }

    if let Some(split) = find_top_level_token(binder, "->") {
        let (from, to) = binder.split_at(split);
        return format!("_{{{} \\to {}}}", from.trim(), to[2..].trim());
    }

    if let Some(split) = find_top_level_token(binder, " in ") {
        let (value, set) = binder.split_at(split);
        return format!("_{{{} \\in {}}}", value.trim(), set[4..].trim());
    }

    format!("_{{{}}}", format_math_phrase(binder))
}

fn format_integral_bound(bound: &str) -> String {
    let bound = bound.trim();
    if bound.is_empty() {
        return String::new();
    }

    if let Some(split) = find_top_level_token(bound, "..") {
        let (lower, upper) = bound.split_at(split);
        return format!("_{{{}}}^{{{}}}", lower.trim(), upper[2..].trim());
    }

    format!("_{{{}}}", format_math_phrase(bound))
}

fn format_integral_body(content: &str) -> String {
    let args = split_top_level_char(content, ',');
    if args.is_empty() {
        return String::new();
    }

    let mut output = args[0].trim().to_string();
    for differential in args.iter().skip(1) {
        output.push_str(r"\,d");
        output.push_str(differential.trim());
    }
    output
}

fn format_math_phrase(input: &str) -> String {
    let trimmed = input.trim();
    if let Some(rest) = trimmed.strip_prefix("boundary ") {
        format!(r"\partial {}", rest.trim())
    } else {
        trimmed.to_string()
    }
}

fn expand_indexed_derivative(input: &str, name: &str, symbol: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut pos = 0;
    let pattern = format!("{name}[");

    while pos < input.len() {
        if let Some(call) = find_function_call(input, pos, &pattern, '[', ']') {
            if let Some((content, next_pos)) = following_paren_content(input, call.next_pos) {
                let args = split_top_level_char(content, ',');
                if args.len() == 2 {
                    let order = input[call.content_start..call.content_end].trim();
                    output.push_str(&format_power_derivative(symbol, order, args[0], args[1]));
                    pos = next_pos;
                    continue;
                }
            }
        }

        copy_current_char(input, &mut output, &mut pos);
    }

    output
}

fn expand_plain_derivative(input: &str, name: &str, symbol: &str) -> String {
    let pattern = [name];
    expand_paren_functions(input, &pattern, |_, content| {
        let args = split_top_level_char(content, ',');
        if args.len() == 2 {
            Some(format!(
                r"\frac{{{} {}}}{{{} {}}}",
                symbol,
                args[0].trim(),
                symbol,
                args[1].trim()
            ))
        } else {
            None
        }
    })
}

fn format_power_derivative(symbol: &str, order: &str, value: &str, variable: &str) -> String {
    format!(
        r"\frac{{{}^{} {}}}{{{} {}^{}}}",
        symbol,
        order,
        value.trim(),
        symbol,
        variable.trim(),
        order
    )
}

fn expand_paren_functions<F>(input: &str, names: &[&str], formatter: F) -> String
where
    F: Fn(&str, &str) -> Option<String>,
{
    let mut output = String::with_capacity(input.len());
    let mut pos = 0;

    while pos < input.len() {
        let mut expanded = None;
        for name in names {
            let pattern = format!("{name}(");
            if let Some(call) = find_function_call(input, pos, &pattern, '(', ')') {
                let content = &input[call.content_start..call.content_end];
                if let Some(rendered) = formatter(name, content) {
                    expanded = Some((rendered, call.next_pos));
                    break;
                }
            }
        }

        if let Some((rendered, next_pos)) = expanded {
            output.push_str(&rendered);
            pos = next_pos;
        } else {
            copy_current_char(input, &mut output, &mut pos);
        }
    }

    output
}

fn expand_set(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut pos = 0;
    let brace_pattern = "set{";
    let paren_pattern = "set(";

    while pos < input.len() {
        let call = find_function_call(input, pos, brace_pattern, '{', '}')
            .or_else(|| find_function_call(input, pos, paren_pattern, '(', ')'));

        if let Some(call) = call {
            output.push_str(r"\left\{");
            output.push_str(&format_set_content(
                &input[call.content_start..call.content_end],
            ));
            output.push_str(r"\right\}");
            pos = call.next_pos;
            continue;
        }

        copy_current_char(input, &mut output, &mut pos);
    }

    output
}

fn format_set_content(content: &str) -> String {
    if let Some(split) = find_top_level_char(content, '|') {
        let (value, condition) = content.split_at(split);
        format!("{} \\;\\middle|\\; {}", value.trim(), condition[1..].trim())
    } else if let Some(split) = find_top_level_char(content, ':') {
        let (value, condition) = content.split_at(split);
        format!("{} \\;\\middle|\\; {}", value.trim(), condition[1..].trim())
    } else {
        content.to_string()
    }
}

fn find_top_level_char(input: &str, target: char) -> Option<usize> {
    find_top_level_token(input, &target.to_string())
}

fn find_top_level_token(input: &str, target: &str) -> Option<usize> {
    find_top_level_token_from(input, &[target])
}

fn find_top_level_token_from(input: &str, targets: &[&str]) -> Option<usize> {
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
            _ if paren_depth == 0 && brace_depth == 0 && bracket_depth == 0 => {
                if targets
                    .iter()
                    .any(|target| input[idx..].starts_with(target))
                {
                    return Some(idx);
                }
            }
            _ => {}
        }
    }

    None
}

fn split_top_level_char(input: &str, target: char) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut rest = input;

    while let Some(pos) = find_top_level_char(rest, target) {
        parts.push(rest[..pos].trim());
        let next = pos + target.len_utf8();
        start += next;
        rest = &input[start..];
    }

    parts.push(rest.trim());
    parts
}

fn expand_plain_delimiters(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut pos = 0;
    let mut paren_depth = 0;
    let mut bracket_depth = 0;
    let matched_delimiters = matched_plain_delimiter_positions(input);

    while pos < input.len() {
        let ch = current_char(input, pos);
        match ch {
            '(' if should_expand_open_paren(input, pos, &output)
                && matched_delimiters.contains(&pos) =>
            {
                output.push_str(r"\left(");
                paren_depth += 1;
            }
            ')' if should_expand_close_delimiter(input, pos, &output) => {
                if paren_depth > 0 {
                    output.push_str(r"\right)");
                    paren_depth -= 1;
                } else {
                    output.push(ch);
                }
            }
            '[' if should_expand_open_bracket(input, pos, &output)
                && matched_delimiters.contains(&pos) =>
            {
                output.push_str(r"\left[");
                bracket_depth += 1;
            }
            ']' if should_expand_close_delimiter(input, pos, &output) => {
                if bracket_depth > 0 {
                    output.push_str(r"\right]");
                    bracket_depth -= 1;
                } else {
                    output.push(ch);
                }
            }
            _ => output.push(ch),
        }
        pos += ch.len_utf8();
    }

    output
}

fn matched_plain_delimiter_positions(input: &str) -> std::collections::HashSet<usize> {
    let mut matched = std::collections::HashSet::new();
    let mut stack = Vec::new();

    for (idx, ch) in input.char_indices() {
        if is_escaped(input, idx) {
            continue;
        }

        match ch {
            '(' | '[' => stack.push((ch, idx)),
            ')' | ']' => {
                if let Some((open, open_idx)) = stack.pop() {
                    if plain_delimiters_match(open, ch) {
                        matched.insert(open_idx);
                        matched.insert(idx);
                    }
                }
            }
            _ => {}
        }
    }

    matched
}

fn plain_delimiters_match(open: char, close: char) -> bool {
    matches!((open, close), ('(', ')') | ('[', ']'))
}

fn should_expand_open_paren(input: &str, pos: usize, output: &str) -> bool {
    !is_escaped(input, pos) && !output.trim_end().ends_with(r"\left")
}

fn should_expand_close_delimiter(input: &str, pos: usize, output: &str) -> bool {
    !is_escaped(input, pos) && !output.trim_end().ends_with(r"\right")
}

fn should_expand_open_bracket(input: &str, pos: usize, output: &str) -> bool {
    should_expand_open_paren(input, pos, output)
        && !output.trim_end().ends_with(r"\\")
        && !previous_output_token_is_latex_command(output)
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

fn expand_wrapped_function(input: &str, function: WrappedFunction) -> String {
    let mut output = String::with_capacity(input.len());
    let mut pos = 0;
    let (name, open, close) = function;
    let brace_pattern = format!("{name}{{");
    let paren_pattern = format!("{name}(");

    while pos < input.len() {
        let call = find_function_call(input, pos, &brace_pattern, '{', '}')
            .or_else(|| find_function_call(input, pos, &paren_pattern, '(', ')'));

        if let Some(call) = call {
            output.push_str(open);
            output.push_str(&input[call.content_start..call.content_end]);
            output.push_str(close);
            pos = call.next_pos;
            continue;
        }

        copy_current_char(input, &mut output, &mut pos);
    }

    output
}

fn expand_lim(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut pos = 0;
    let pattern = "lim[";

    while pos < input.len() {
        if let Some(call) = find_function_call(input, pos, pattern, '[', ']') {
            output.push_str(r"\lim_{");
            output.push_str(&input[call.content_start..call.content_end]);
            output.push('}');
            pos = call.next_pos;
            continue;
        }

        copy_current_char(input, &mut output, &mut pos);
    }

    output
}

fn find_function_call(
    input: &str,
    pos: usize,
    pattern: &str,
    open: char,
    close: char,
) -> Option<FunctionCall> {
    if !starts_identifier_function(input, pos, pattern) {
        return None;
    }

    let open_pos = pos + pattern.len() - 1;
    let end = find_matching(input, open_pos, open, close)?;
    Some(FunctionCall {
        content_start: pos + pattern.len(),
        content_end: end,
        next_pos: end + 1,
    })
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
    replace_token(input, from, to, is_identifier_char)
}

fn replace_symbol_prefix(input: &str, from: &str, to: &str) -> String {
    replace_token(input, from, to, |ch| ch.is_ascii_alphanumeric())
}

fn replace_token<F>(input: &str, from: &str, to: &str, disallowed_next: F) -> String
where
    F: Fn(char) -> bool,
{
    let mut output = String::with_capacity(input.len());
    let mut pos = 0;

    while pos < input.len() {
        if can_replace_token(input, pos, from, &disallowed_next) {
            output.push_str(to);
            pos += from.len();
        } else {
            copy_current_char(input, &mut output, &mut pos);
        }
    }

    output
}

fn can_replace_token<F>(input: &str, pos: usize, from: &str, disallowed_next: &F) -> bool
where
    F: Fn(char) -> bool,
{
    input[pos..].starts_with(from)
        && !is_escaped(input, pos)
        && previous_char(input, pos) != Some('\\')
        && !previous_char(input, pos).is_some_and(is_identifier_char)
        && !input[pos + from.len()..]
            .chars()
            .next()
            .is_some_and(disallowed_next)
}

fn current_char(input: &str, pos: usize) -> char {
    input[pos..]
        .chars()
        .next()
        .expect("pos is on a char boundary")
}

fn copy_current_char(input: &str, output: &mut String, pos: &mut usize) {
    let ch = current_char(input, *pos);
    output.push(ch);
    *pos += ch.len_utf8();
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
    fn leaves_mixed_interval_delimiters_unscaled() {
        let protected = protect_math(r"$[0, 1) \cup (2, 3]$", true);

        assert_eq!(protected.segments[0], r"$[0, 1) \cup (2, 3]$");
    }

    #[test]
    fn expands_additional_math_shorthand_forms() {
        let protected = protect_math(
            r"$A[0] + \sqrt[n] + \\[1em] + unit{n} + eps_0 + del_a + inf_n + set(v{x} in bb{R} | norm(v{x}) <= 1)$",
            true,
        );

        assert_eq!(
            protected.segments[0],
            r"$A\left[0\right] + \sqrt[n] + \\[1em] + \hat{\mathbf{n}} + \epsilon_0 + \delta_a + \infty_n + \left\{\mathbf{x} \in \mathbb{R} \;\middle|\; \left\lVert \mathbf{x} \right\rVert \le 1\right\}$"
        );
    }

    #[test]
    fn expands_indexed_operator_shorthand() {
        let protected = protect_math(
            r"$sum[i=1..n](a_i) + prod[i=1..n](b_i) + lim[x -> a](f(x)) + sup[x in A](g(x)) + inf[x in A](g(x)) + union[a in A](X_a)$",
            true,
        );

        assert_eq!(
            protected.segments[0],
            r"$\sum_{i=1}^{n} a_i + \prod_{i=1}^{n} b_i + \lim_{x \to a} f\left(x\right) + \sup_{x \in A} g\left(x\right) + \inf_{x \in A} g\left(x\right) + \bigcup_{a \in A} X_a$"
        );
    }

    #[test]
    fn expands_integral_shorthand() {
        let protected = protect_math(
            r"$int[a..b](f(x), x) + iint[D](g(x,y), x, y) + int[boundary Phi](omega)$",
            true,
        );

        assert_eq!(
            protected.segments[0],
            r"$\int_{a}^{b} f\left(x\right)\,dx + \iint_{D} g\left(x,y\right)\,dx\,dy + \int_{\partial \Phi} \omega$"
        );
    }

    #[test]
    fn expands_derivative_and_vector_analysis_shorthand() {
        let protected = protect_math(
            r"$dd(f, x) + dd[n](f, x) + pd(f, x_i) + pd2(f, x, y) + grad(f) + div(F) + curl(F)$",
            true,
        );

        assert_eq!(
            protected.segments[0],
            r"$\frac{d f}{d x} + \frac{d^n f}{d x^n} + \frac{\partial f}{\partial x_i} + \frac{\partial^2 f}{\partial x \partial y} + \nabla f + \operatorname{div} F + \operatorname{curl} F$"
        );
    }

    #[test]
    fn expands_vector_and_topology_helpers() {
        let protected = protect_math(
            r"$norm[2](x) + ip(x,y) + dot(x,y) + cross(x,y) + dist(x,y) + tuple(x_1, ..., x_n) + seq(x_n) + cl(A) + interior(A) + bd(A) + pre(f,B) + ball(x,r)$",
            true,
        );

        assert_eq!(
            protected.segments[0],
            r"$\left\lVert x \right\rVert_{2} + \left\langle x, y \right\rangle + x \cdot y + x \times y + d\left(x, y\right) + \left(x_1, \ldots, x_n\right) + \{x_n\}_{n \ge 1} + \overline{A} + {A}^\circ + \partial A + f^{-1}\left(B\right) + B_{r}\left(x\right)$"
        );
    }

    #[test]
    fn expands_matrix_and_form_helpers() {
        let protected = protect_math(
            r"$bmat(a,b;c,d) + wedge(dx, dy, dz) + ext(omega) + pull(T, omega) + form(F) + boundary(Phi)$",
            true,
        );

        assert_eq!(
            protected.segments[0],
            "$\\begin{bmatrix}a & b \\\\\nc & d\\end{bmatrix} + dx \\wedge dy \\wedge dz + d{\\omega} + T^*\\omega + \\lambda_{F} + \\partial \\Phi$"
        );
    }

    #[test]
    fn expands_inline_cases() {
        let protected = protect_math(r"$abs(x) = cases(x | x >= 0; -x | x < 0)$", true);

        assert_eq!(
            protected.segments[0],
            "$\\left\\lvert x \\right\\rvert = \\begin{cases}x & x \\ge 0 \\\\\n-x & x < 0\\end{cases}$"
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
    fn preprocesses_plain_math_blocks() {
        let markdown = preprocess_math_shorthand_blocks(
            r#"before
:::math
x^2 + y^2 = r^2
:::
after"#,
        );

        assert_eq!(markdown, "before\n$$\nx^2 + y^2 = r^2\n$$\nafter\n");
    }

    #[test]
    fn preprocesses_auto_aligned_math_blocks() {
        let markdown = preprocess_math_shorthand_blocks(
            r#":::math
S_n = sum[k=1..n] a_k
= a_1 + ... + a_n
<= n max[k] abs(a_k)
:::"#,
        );

        assert_eq!(
            markdown,
            "$$\n\\begin{aligned}\nS_n &= sum[k=1..n] a_k \\\\\n&= a_1 + ... + a_n \\\\\n&<= n max[k] abs(a_k)\n\\end{aligned}\n$$\n"
        );
    }

    #[test]
    fn preprocesses_explicit_aligned_math_blocks() {
        let markdown = preprocess_math_shorthand_blocks(
            r#":::math
f(x) &= x^2
g(x) &= x^3
:::"#,
        );

        assert_eq!(
            markdown,
            "$$\n\\begin{aligned}\nf(x) &= x^2 \\\\\ng(x) &= x^3\n\\end{aligned}\n$$\n"
        );
    }

    #[test]
    fn preprocesses_plain_mode_math_blocks() {
        let markdown = preprocess_math_shorthand_blocks(
            r#":::math plain
a = b
c = d
:::"#,
        );

        assert_eq!(markdown, "$$\na = b\nc = d\n$$\n");
    }

    #[test]
    fn preprocesses_math_block_cases() {
        let markdown = preprocess_math_shorthand_blocks(
            r#":::math
f(x) = cases:
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
    fn preprocesses_system_math_blocks_with_tags() {
        let markdown = preprocess_math_shorthand_blocks(
            r#":::math system tag=2.1
2x + 3y = 1
x - y = 0
:::"#,
        );

        assert_eq!(
            markdown,
            "$$\n\\left\\{\\begin{aligned}\n2x + 3y &= 1 \\\\\nx - y &= 0 \\tag{2.1}\n\\end{aligned}\\right.\n$$\n"
        );
    }

    #[test]
    fn preprocesses_matrix_math_blocks_with_inline_tags() {
        let markdown = preprocess_math_shorthand_blocks(
            r#":::math matrix
a, b
c, d #tag A
:::"#,
        );

        assert_eq!(
            markdown,
            "$$\n\\begin{bmatrix}\na & b \\\\\nc & d\n\\end{bmatrix} \\tag{A}\n$$\n"
        );
    }

    #[test]
    fn leaves_shorthand_blocks_inside_code_fences() {
        let markdown = preprocess_math_shorthand_blocks(
            r#"```text
:::math
x = y
:::
```"#,
        );

        assert_eq!(markdown, "```text\n:::math\nx = y\n:::\n```\n");
    }
}
