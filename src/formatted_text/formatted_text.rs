use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::config::Config;

use super::{
    markdown_expandable::{preprocess_cards, preprocess_expandables},
    pandoc_latex_filters::{EnvFilter, PandocFilter},
    shell::run_with_timeout,
};

#[derive(Debug, Clone)]
pub enum FormattedText {
    Latex(String),
    Markdown(String),
    Html(String),
}

fn default_numbered() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theorem {
    pub name: String,
    pub label: String,

    #[serde(default = "default_numbered")]
    pub numbered: bool,
}

impl Theorem {
    pub fn label(&self, counter: usize) -> String {
        if self.numbered {
            format!("{} {}", self.label, counter)
        } else {
            self.label.clone()
        }
    }
}

impl FormattedText {
    pub fn to_html(&self, config: &Config) -> Result<String, String> {
        match self {
            FormattedText::Latex(s) => latex_to_html(s, &config.theorems),
            FormattedText::Markdown(s) => markdown_to_html(s, config),
            FormattedText::Html(s) => Ok(s.clone()),
        }
    }
}

fn latex_to_html(latex: &str, theorems: &Vec<Theorem>) -> Result<String, String> {
    let mut filters: Vec<Box<dyn PandocFilter>> = vec![Box::new(EnvFilter::new(theorems.clone()))];

    let mut preprocessed = latex.to_string();
    for filter in &mut filters {
        preprocessed = filter.preprocess(&preprocessed)?;
    }

    let pandoc_output = run_with_timeout(
        "pandoc",
        &["--from=latex", "--to=html", "--mathjax"],
        Some(&preprocessed.as_str()),
        Duration::from_secs(1),
    );

    pandoc_output.map(|output| {
        let mut postprocessed = output.to_string();
        for filter in &mut filters.iter_mut().rev() {
            match filter.postprocess(&postprocessed) {
                Ok(new_output) => postprocessed = new_output,
                Err(_) => break,
            }
        }
        postprocessed
    })
}

fn markdown_to_html(markdown: &str, config: &Config) -> Result<String, String> {
    let mut options = comrak::ComrakOptions::default();
    options.extension.tasklist = true;
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.alerts = true;
    options.parse.smart = true;
    options.render.unsafe_ = true;

    let markdown = &preprocess_expandables(markdown);
    let markdown = &preprocess_cards(markdown);

    let mut plugins = comrak::Plugins::default();
    let builder = comrak::plugins::syntect::SyntectAdapterBuilder::new()
        .theme(config.syntax_highlighter_theme.as_str());
    let adapter = builder.build();
    plugins.render.codefence_syntax_highlighter = Some(&adapter);

    let html = comrak::markdown_to_html_with_plugins(markdown, &options, &plugins);
    Ok(html)
}

#[cfg(test)]
mod test_latex_to_html {
    use super::*;

    #[test]
    fn basic_checks() {
        let result_1 = latex_to_html("latex", &vec![]);
        assert!(result_1.is_ok());
        let output_1 = result_1.unwrap();
        assert_eq!(output_1, "<p>latex</p>\n");

        let result_2 = latex_to_html("$2^5$", &vec![]);
        assert!(result_2.is_ok());
        let output_2 = result_2.unwrap();
        assert!(output_2.contains("\\(2^5\\)"));

        let result_3 = latex_to_html("$2\\", &vec![]);
        assert!(result_3.is_err());
    }

    #[test]
    fn retains_equation_blocks() {
        let input = r#"\begin{equation}\label{inequality:first}\frac{1}{x}\end{equation}"#;
        let result = latex_to_html(&input, &vec![]);
        assert!(result.is_ok());
        let output = result.unwrap();
        println!("{}", output);
        let expected_output = format!("<p><span class=\"math display\">\\[{}\\]</span></p>", input);
        assert!(output.contains(&expected_output));
    }

    #[test]
    fn retains_refs_to_equations() {
        let input = r#"
        \begin{equation}\label{inequality:first}\end{equation}
        \begin{equation}\label{inequality:second}\end{equation}
        Inequality~\ref{inequality:first} and Inequality~\ref{inequality:second}."#;
        let result = latex_to_html(&input, &vec![]);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains(r"\ref{inequality:first}"));
        assert!(output.contains(r"\ref{inequality:second}"));
    }

    #[test]
    fn test_tables() {
        let input = r#"
\begin{table}
  \begin{tabular}{|c|c|}
    \hline
    A & B \\ \hline
    1 & 2 \\ \hline
  \end{tabular}
\end{table}"#;
        let result = latex_to_html(&input, &vec![]);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("<table>"));
    }

    #[test]
    fn processes_theorems() {
        let input = r#"
        \begin{theorem}\label{lm:1}
        In a one-hour interval, at most 20 millimeters of rain can fall.
        \end{theorem}
        This is a reference \ref{lm:1}."#;
        let theorems = vec![Theorem {
            name: "theorem".to_string(),
            label: "Theorem".to_string(),
            numbered: true,
        }];
        let result = latex_to_html(&input, &theorems);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("<strong>Theorem 1</strong>. "));
        assert!(output.contains("<span id=\"lm:1\" label=\"lm:1\"></span>"));
        assert!(output.contains("<a href=\"#lm:1\">1</a>"));
    }

    #[test]
    fn ignores_unknown_environments() {
        let input = r#"\begin{solution} Something \end{solution}"#;
        let result = latex_to_html(&input, &vec![]);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output, "<p>Something</p>\n");
    }

    #[test]
    fn ignores_extra_parameters() {
        let input =
            r#"\begin{problem}{82/figs/pic.jpeg}{Game of Pebbles} We have a problem \end{problem}"#;
        let theorems = vec![Theorem {
            name: "theorem".to_string(),
            label: "Theorem".to_string(),
            numbered: true,
        }];
        let result = latex_to_html(&input, &theorems);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output, "<p>We have a problem</p>\n");
    }

    #[test]
    fn ignores_extra_parameters_empty() {
        let input = r#"\begin{problem}{}{A Problem}Some text\end{problem}"#;
        let theorems = vec![Theorem {
            name: "theorem".to_string(),
            label: "Theorem".to_string(),
            numbered: true,
        }];
        let result = latex_to_html(&input, &theorems);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output, "<p>Some text</p>\n");
    }
}

#[cfg(test)]
mod test_markdown_to_html {
    use super::*;
    use crate::content::test::get_test_config;

    #[test]
    fn test_basic_checks() {
        let config = get_test_config();
        let result_1 = markdown_to_html("markdown", &config);
        assert!(result_1.is_ok());
        let output_1 = result_1.unwrap();
        assert_eq!(output_1, "<p>markdown</p>\n");

        let result_2 = markdown_to_html("## heading\ntext\n", &config);
        assert!(result_2.is_ok());
        let output_2 = result_2.unwrap();
        assert_eq!(output_2, "<h2>heading</h2>\n<p>text</p>\n");
    }

    #[test]
    fn test_markdown_with_math() {
        let config = get_test_config();
        let result_3 = markdown_to_html("$$\n2^5\n$$", &config);
        assert!(result_3.is_ok());
        let output_3 = result_3.unwrap();
        assert!(output_3.contains("$$\n2^5\n$$"));
    }

    #[test]
    fn test_autolink() {
        let config = get_test_config();
        let result_4 = markdown_to_html("https://example.com", &config);
        assert!(result_4.is_ok());
        let output_4 = result_4.unwrap();
        assert_eq!(
            output_4,
            "<p><a href=\"https://example.com\">https://example.com</a></p>\n"
        );
    }

    #[test]
    fn test_syntax_highlighting() {
        let config = get_test_config();
        println!("{}", config.syntax_highlighter_theme);
        let result = markdown_to_html("```rust\nfn main() {}\n```", &config);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("background-color"));
    }

    #[test]
    fn test_alerts() {
        let config = get_test_config();
        let result = markdown_to_html(
            "> [!NOTE]
> Highlights information that users should take into account, even when skimming.",
            &config,
        );
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains(r#"<p class="markdown-alert-title">Note</p>"#));
    }

    #[test]
    fn test_strikethrough() {
        let config = get_test_config();
        let result = markdown_to_html("~~strikethrough~~", &config);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output, "<p><del>strikethrough</del></p>\n");
    }

    #[test]
    fn test_table() {
        let config = get_test_config();
        let result = markdown_to_html(
            "| Header 1 | Header 2 |\n| --------- | -------- |\n| Row 1    | Row 2   |",
            &config,
        );
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("<table>"));
    }

    #[test]
    fn test_custom_html() {
        let config = get_test_config();
        let result = markdown_to_html("<div class=\"custom-class\">Custom HTML</div>", &config);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output, "<div class=\"custom-class\">Custom HTML</div>\n");
    }

    #[test]
    fn test_expandables() {
        let config = get_test_config();
        let input = "Some text

:::expandable
**Proof**. [Click to Expand]

The proof text $2^4$.
::::

Some other text
        ";
        let result = markdown_to_html(input, &config);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains(r#"<p><strong>Proof</strong>. <a class="expand-link" data-bs-toggle="collapse" href='#expand-1'>Click to Expand</a></p>"#));
    }

    #[test]
    fn test_cards() {
        let config = get_test_config();
        let input = r#"Some text
:::card[example]
**Example (Condition Number)** Let $f(x) = \sqrt{x}$. Since $f'(x) = \frac{1}{2\sqrt{x}}$, we have:

$$
\kappa_{rel}(f, x) = \left\lvert \frac{x f'(x)}{f(x)} \right\rvert = \left\lvert \frac{x/(2\sqrt{x})}{\sqrt{x}} \right\rvert = \frac{1}{2}
$$

This means that a given relative change in the input causes a relative change in the output of about half as much.
::::
Some other text
        "#;
        let result = markdown_to_html(input, &config);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains(r#"<div class="card example">"#));
        assert!(output.contains(r#"<strong>Example (Condition Number)</strong>"#));
        assert!(output.contains(r#"Let $f(x) = \sqrt{x}$."#));
        assert!(output.contains(r#"<p>Some other text</p>"#));
    }
}
