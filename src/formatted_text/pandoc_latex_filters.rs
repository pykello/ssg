use regex::Regex;
use std::collections::{HashMap, HashSet};

use super::formatted_text::Theorem;

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
        pattern.push_str(r"|\\begin(\{[^}]+\})+|\\end\{\w+\}");
        Regex::new(&pattern).unwrap()
    }

    fn clean_labels(&mut self, input: &str) -> String {
        // Regex pattern to match <span id="X" label="X">[X]</span>
        let re =
            Regex::new(r#"<span id="([\w:-]+)" label="([\w:-]+)">\[[\w:-]+\]</span>"#).unwrap();

        // Replace all matches with <span id="X" label="X"></span>
        let cleaned_html = re.replace_all(input, |caps: &regex::Captures| {
            let id = &caps[1]; // Extract id
            let label = &caps[2]; // Extract label

            if self.theorem_labels.contains_key(id) {
                format!(r#"<span id="{}" label="{}"></span>"#, id, label)
            } else {
                caps[0].to_string() // If not in labels, keep the original text
            }
        });

        cleaned_html.to_string()
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
                let env_name: &str = s
                    .trim_start_matches(r"\begin{")
                    .splitn(2, '}')
                    .next()
                    .unwrap_or("");
                env_stack.push(env_name.to_string());
                if self.theorems.contains_key(env_name) {
                    let theorem = &self.theorems[env_name];
                    if theorem.numbered {
                        theorem_counter += 1;
                    }
                    result.push_str(&format!("\\textbf{{{}}}. ", theorem.label(theorem_counter)));
                } else if env_name == "equation" {
                    result.push_str(r"$$\begin{equation}");
                } else if env_name == "problem" || env_name == "solution" {
                    // ignore extra parameters
                    result.push_str(format!("\\begin{{{}}}", env_name).as_str());
                } else {
                    result.push_str(s);
                }
            } else if s.starts_with(r"\label") {
                if let Some(env_name) = env_stack.last() {
                    if self.theorems.contains_key(env_name) {
                        let label = s.trim_start_matches(r"\label{").trim_end_matches('}');
                        self.theorem_labels
                            .insert(label.to_string(), format!("{}", theorem_counter));
                    } else if env_name == "equation" {
                        let label = s.trim_start_matches(r"\label{").trim_end_matches('}');
                        self.equation_labels.insert(label.to_string());
                    }
                    result.push_str(s);
                } else {
                    result.push_str(s);
                }
            } else if s.starts_with(r"\ref") {
                let label = s.trim_start_matches(r"\ref{").trim_end_matches('}');
                if self.theorem_labels.contains_key(label) {
                    result.push_str(&format!(
                        "\\href{{#{}}}{{{}}}",
                        label, self.theorem_labels[label]
                    ));
                } else if self.equation_labels.contains(label) {
                    result.push_str(&format!("(EQREFBEGIN){}(EQREFEND)", label));
                } else {
                    result.push_str(s);
                }
            } else if s.starts_with(r"\end") {
                let env_name = s.trim_start_matches(r"\end{").trim_end_matches('}');
                let last_env = env_stack.pop().unwrap();
                if env_name != last_env {
                    return Err(format!(
                        "Mismatched environment tags: \\begin{{{}}} and \\end{{{}}}",
                        last_env, env_name
                    ));
                }
                if self.theorems.contains_key(env_name) {
                    result.push_str("\n");
                } else if env_name == "equation" {
                    result.push_str(r"\end{equation}$$");
                } else {
                    result.push_str(s);
                }
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
