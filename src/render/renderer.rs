use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use tera::{Context, Function, Tera, Value};

use crate::config::Config;

pub struct Renderer {
    tera: Tera,
    default_context: Context,
}

impl Renderer {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let mut tera = load_templates(config)?;
        let translations = load_configured_translations(config)?;
        tera.register_function("translate", translate_to_tera(translations));

        Ok(Self {
            tera,
            default_context: build_default_context(config),
        })
    }

    pub fn render(
        &self,
        template_name: &str,
        custom_context: HashMap<String, Value>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut context = self.default_context.clone();
        merge_render_context(&mut context, custom_context);

        match self.tera.render(template_name, &context) {
            Ok(s) => Ok(s),
            Err(e) => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Error rendering template: {:#?}", e),
            ))),
        }
    }
}

fn load_templates(config: &Config) -> Result<Tera, Box<dyn Error>> {
    let templates_path = config.template_dir.join("**/*.html");
    Tera::new(&templates_path.to_string_lossy()).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Error parsing templates: {}", e),
        )
        .into()
    })
}

fn load_configured_translations(
    config: &Config,
) -> Result<HashMap<String, String>, Box<dyn Error>> {
    match &config.translations_csv {
        Some(translations_file) => load_translations(translations_file).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Error loading translations: {}", e),
            )
            .into()
        }),
        None => Ok(HashMap::new()),
    }
}

fn build_default_context(config: &Config) -> Context {
    let mut context = Context::new();
    context.insert("text_direction", &config.text_direction);
    context.insert("language", &config.language);

    if let Some(extra_context) = &config.context {
        for (key, value) in extra_context {
            context.insert(key, value);
        }
    }

    context
}

fn merge_render_context(context: &mut Context, custom_context: HashMap<String, Value>) {
    for (key, value) in custom_context {
        context.insert(key, &value);
    }
}

fn strip_csv_quotes(s: &str) -> String {
    let mut s = s.trim();
    if s.starts_with('"') {
        s = &s[1..];
    }
    if s.ends_with('"') {
        s = &s[..s.len() - 1];
    }
    s.to_string()
}

fn load_translations(path: &Path) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut translations = HashMap::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() || line.starts_with("#") {
            continue; // Skip empty lines and comments
        }

        if let Some(pos) = line.find(',') {
            let key = strip_csv_quotes(&line[..pos]);
            let value = strip_csv_quotes(&line[(pos + 1)..]);
            translations.insert(key, value);
        }
    }

    Ok(translations)
}

fn translate_to_tera(translations: HashMap<String, String>) -> impl Function {
    Box::new(
        move |args: &HashMap<String, Value>| -> tera::Result<Value> {
            let key = args
                .get("key")
                .and_then(|v| v.as_str())
                .ok_or_else(|| tera::Error::msg("Missing or invalid key for translation"))?;

            let translation = match translations.get(key) {
                Some(translation) => translation.to_string(),
                None => key.to_string(),
            };

            Ok(Value::String(translation))
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn new_returns_error_for_missing_templates() {
        let temp_dir = tempdir().unwrap();
        let template_dir = temp_dir.path().join("templates");
        fs::create_dir_all(&template_dir).unwrap();
        fs::write(template_dir.join("bad.html"), "{{").unwrap();
        let config = Config {
            template_dir,
            ..Default::default()
        };

        let err = match Renderer::new(&config) {
            Ok(_) => panic!("missing templates should return an error"),
            Err(err) => err,
        };

        assert!(err.to_string().contains("Error parsing templates"));
    }

    #[test]
    fn new_returns_error_for_missing_translations() {
        let temp_dir = tempdir().unwrap();
        let template_dir = temp_dir.path().join("templates");
        fs::create_dir_all(&template_dir).unwrap();
        fs::write(template_dir.join("page.html"), "{{ title }}").unwrap();
        let config = Config {
            template_dir,
            translations_csv: Some(temp_dir.path().join("missing.csv")),
            ..Default::default()
        };

        let err = match Renderer::new(&config) {
            Ok(_) => panic!("missing translations should return an error"),
            Err(err) => err,
        };

        assert!(err.to_string().contains("Error loading translations"));
    }

    #[test]
    fn new_builds_renderer_for_valid_templates() -> Result<(), Box<dyn Error>> {
        let temp_dir = tempdir()?;
        let template_dir = temp_dir.path().join("templates");
        fs::create_dir_all(&template_dir)?;
        fs::write(template_dir.join("page.html"), "{{ title }}")?;
        let config = Config {
            template_dir,
            ..Default::default()
        };

        let renderer = Renderer::new(&config)?;
        let mut context = HashMap::new();
        context.insert("title".to_string(), Value::String("Hello".to_string()));

        assert_eq!(renderer.render("page.html", context)?, "Hello");

        Ok(())
    }
}
