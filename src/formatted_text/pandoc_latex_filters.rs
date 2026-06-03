use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use super::formatted_text::Theorem;

static CLEAN_LABELS_RE: OnceLock<Regex> = OnceLock::new();

fn clean_labels_regex() -> &'static Regex {
    CLEAN_LABELS_RE.get_or_init(|| {
        Regex::new(r#"<span id="([\w:-]+)" label="([\w:-]+)">\[[\w:-]+\]</span>"#)
            .expect("valid label span regex")
    })
}

pub trait PandocFilter {
    fn preprocess(&mut self, input: &str) -> Result<String, String>;
    fn postprocess(&mut self, input: &str) -> Result<String, String>;
}

pub struct EnvFilter {
    theorems: HashMap<String, Theorem>,
    theorem_labels: HashMap<String, String>,
    equation_labels: HashSet<String>,
}

impl EnvFilter {
    pub fn new(theorems: Vec<Theorem>) -> Self {
        Self {
            theorems: theorems.into_iter().map(|t| (t.name.clone(), t)).collect(),
            theorem_labels: HashMap::new(),
            equation_labels: HashSet::new(),
        }
    }

    fn generate_theorem_regex(&self) -> Regex {
        let mut pattern = r"\\label\{[\w:-]+\}|\\ref\{[\w:-]+\}".to_string();
        pattern.push_str(r"|\\begin(\{[^}]*\})+|\\end\{\w+\}");
        Regex::new(&pattern).expect("valid theorem regex")
    }

    fn clean_labels(&mut self, input: &str) -> String {
        let re = clean_labels_regex();
        let cleaned_html = re.replace_all(input, |caps: &regex::Captures| {
            let id = &caps[1];
            let label = &caps[2];

            if self.theorem_labels.contains_key(id) {
                format!(r#"<span id="{}" label="{}"></span>"#, id, label)
            } else {
                caps[0].to_string()
            }
        });

        cleaned_html.to_string()
    }

    fn write_begin_environment(
        &self,
        token: &str,
        env_name: &str,
        theorem_counter: &mut usize,
        result: &mut String,
    ) {
        if self.theorems.contains_key(env_name) {
            let theorem = &self.theorems[env_name];
            if theorem.numbered {
                *theorem_counter += 1;
            }
            result.push_str(&format!(
                "\\textbf{{{}}}. ",
                theorem.label(*theorem_counter)
            ));
        } else if env_name == "equation" {
            result.push_str(r"$$\begin{equation}");
        } else if env_name == "problem" || env_name == "solution" {
            result.push_str(format!("\\begin{{{}}}", env_name).as_str());
        } else {
            result.push_str(token);
        }
    }

    fn write_label(
        &mut self,
        token: &str,
        env_stack: &[String],
        theorem_counter: usize,
        result: &mut String,
    ) {
        if let Some(env_name) = env_stack.last() {
            let label = command_argument(token);
            if self.theorems.contains_key(env_name) {
                self.theorem_labels
                    .insert(label.to_string(), format!("{}", theorem_counter));
            } else if env_name == "equation" {
                self.equation_labels.insert(label.to_string());
            }
        }
        result.push_str(token);
    }

    fn write_reference(&self, token: &str, result: &mut String) {
        let label = command_argument(token);
        if self.theorem_labels.contains_key(label) {
            result.push_str(&format!(
                "\\href{{#{}}}{{{}}}",
                label, self.theorem_labels[label]
            ));
        } else if self.equation_labels.contains(label) {
            result.push_str(&format!("(EQREFBEGIN){}(EQREFEND)", label));
        } else {
            result.push_str(token);
        }
    }

    fn write_end_environment(
        &self,
        token: &str,
        env_name: &str,
        env_stack: &mut Vec<String>,
        result: &mut String,
    ) -> Result<(), String> {
        let last_env = env_stack.pop().unwrap();
        if env_name != last_env {
            return Err(format!(
                "Mismatched environment tags: \\begin{{{}}} and \\end{{{}}}",
                last_env, env_name
            ));
        }

        if self.theorems.contains_key(env_name) {
            result.push('\n');
        } else if env_name == "equation" {
            result.push_str(r"\end{equation}$$");
        } else {
            result.push_str(token);
        }

        Ok(())
    }
}

impl PandocFilter for EnvFilter {
    fn preprocess(&mut self, input: &str) -> Result<String, String> {
        let mut result = "".to_string();
        let theorem_re = self.generate_theorem_regex();
        let mut theorem_counter = 0;
        let mut processed = 0;
        let mut env_stack: Vec<String> = Vec::new();
        env_stack.push("document".to_string());

        for caps in theorem_re.captures_iter(input) {
            let m = caps.get(0).unwrap();
            let s = m.as_str();
            result.push_str(&input[processed..m.start()]);
            if s.starts_with(r"\begin") {
                let env_name = begin_environment_name(s);
                env_stack.push(env_name.to_string());
                self.write_begin_environment(s, env_name, &mut theorem_counter, &mut result);
            } else if s.starts_with(r"\label") {
                self.write_label(s, &env_stack, theorem_counter, &mut result);
            } else if s.starts_with(r"\ref") {
                self.write_reference(s, &mut result);
            } else if s.starts_with(r"\end") {
                let env_name = end_environment_name(s);
                self.write_end_environment(s, env_name, &mut env_stack, &mut result)?;
            } else {
                result.push_str(m.as_str());
            }
            processed = m.end();
        }
        result.push_str(&input[processed..]);
        Ok(result)
    }

    fn postprocess(&mut self, input: &str) -> Result<String, String> {
        let result = self
            .clean_labels(input)
            .replace("(EQREFBEGIN)", "\\ref{")
            .replace("(EQREFEND)", "}")
            .replace(r"$$\begin{equation}", r"\begin{equation}")
            .replace(r"\end{equation}$$", r"\end{equation}");
        Ok(result)
    }
}

fn begin_environment_name(token: &str) -> &str {
    token
        .trim_start_matches(r"\begin{")
        .split_once('}')
        .map_or("", |(name, _)| name)
}

fn end_environment_name(token: &str) -> &str {
    token.trim_start_matches(r"\end{").trim_end_matches('}')
}

fn command_argument(token: &str) -> &str {
    token
        .split_once('{')
        .and_then(|(_, rest)| rest.strip_suffix('}'))
        .unwrap_or("")
}
