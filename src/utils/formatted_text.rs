use std::time::Duration;

use super::{
    pandoc_latex_filters::{EnvFilter, PandocFilter},
    shell::run_with_timeout,
};

#[derive(Debug, Clone)]
pub enum FormattedText {
    Latex(String),
    Markdown(String),
}

impl FormattedText {
    pub fn to_html(&self) -> Result<String, String> {
        match self {
            FormattedText::Latex(s) => latex_to_html(s, vec![]),
            FormattedText::Markdown(s) => markdown_to_html(s),
        }
    }
}

fn latex_to_html(latex: &str, theorems: Vec<(String, String)>) -> Result<String, String> {
    let mut filters: Vec<Box<dyn PandocFilter>> = vec![Box::new(EnvFilter::new(theorems))];

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

fn markdown_to_html(markdown: &str) -> Result<String, String> {
    run_with_timeout(
        "pandoc",
        &["--from=markdown", "--to=html", "--mathjax"],
        Some(markdown),
        Duration::from_secs(1),
    )
}

#[cfg(test)]
mod test_latex_to_html {
    use super::*;

    #[test]
    fn basic_checks() {
        let result_1 = latex_to_html("latex", vec![]);
        assert!(result_1.is_ok());
        let output_1 = result_1.unwrap();
        assert_eq!(output_1, "<p>latex</p>\n");

        let result_2 = latex_to_html("$2^5$", vec![]);
        assert!(result_2.is_ok());
        let output_2 = result_2.unwrap();
        assert!(output_2.contains("\\(2^5\\)"));

        let result_3 = latex_to_html("$2\\", vec![]);
        assert!(result_3.is_err());
    }

    #[test]
    fn retains_equation_blocks() {
        let input = r#"\begin{equation}\label{inequality:first}\frac{1}{x}\end{equation}"#;
        let result = latex_to_html(&input, vec![]);
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
        let result = latex_to_html(&input, vec![]);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains(r"\ref{inequality:first}"));
        assert!(output.contains(r"\ref{inequality:second}"));
    }

    #[test]
    fn processes_theorems() {
        let input = r#"
        \begin{theorem}\label{lm:1}
        In a one-hour interval, at most 20 millimeters of rain can fall.
        \end{theorem}
        This is a reference \ref{lm:1}."#;
        let result = latex_to_html(&input, vec![("theorem".to_string(), "Theorem".to_string())]);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("<strong>Theorem 1</strong>: "));
        assert!(output.contains("<span id=\"lm:1\" label=\"lm:1\"></span>"));
        assert!(output.contains("<a href=\"#lm:1\">1</a>"));
    }

    #[test]
    fn ignores_unknown_environments() {
        let input = r#"\begin{solution} Something \end{solution}"#;
        let result = latex_to_html(&input, vec![]);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output, "<p>Something</p>\n");
    }

    #[test]
    fn ignores_extra_parameters() {
        let input =
            r#"\begin{problem}{82/figs/pic.jpeg}{Game of Pebbles} We have a problem \end{problem}"#;
        let result = latex_to_html(&input, vec![("theorem".to_string(), "Theorem".to_string())]);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output, "<p>We have a problem</p>\n");
    }
}

#[test]
fn test_markdown_to_html() {
    let result_1 = markdown_to_html("markdown");
    assert!(result_1.is_ok());
    let output_1 = result_1.unwrap();
    assert_eq!(output_1, "<p>markdown</p>\n");

    let result_2 = markdown_to_html("## heading\ntext\n");
    assert!(result_2.is_ok());
    let output_2 = result_2.unwrap();
    assert_eq!(output_2, "<h2 id=\"heading\">heading</h2>\n<p>text</p>\n");

    let result_3 = markdown_to_html("$$\n2^5\n$$");
    assert!(result_3.is_ok());
    let output_3 = result_3.unwrap();
    assert!(output_3.contains("\\[\n2^5\n\\]"));
}
