use std::collections::HashMap;
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
    pub fn new(config: &Config) -> Self {
        let templates_path = config.template_dir.join("**/*.html");
        let mut tera = match Tera::new(&templates_path.to_string_lossy()) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Error parsing templates: {}", e);
                std::process::exit(1);
            }
        };

        let translations = match &config.translation_dir {
            Some(translations_root) => {
                let translations_file = translations_root.join(&format!("{}.csv", config.language));
                match load_translations(&translations_file) {
                    Ok(t) => t,
                    Err(e) => {
                        eprintln!("Error loading translations: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            None => HashMap::new(),
        };

        tera.register_function("translate", translate_to_tera(translations));

        let mut default_context = Context::new();
        default_context.insert("text_direction", &config.text_direction);
        default_context.insert("language", &config.language);

        if let Some(context) = &config.context {
            for (key, value) in context {
                default_context.insert(key, &value);
            }
        }

        Self {
            tera,
            default_context,
        }
    }

    pub fn render(
        &self,
        template_name: &str,
        custom_context: HashMap<String, Value>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut context = self.default_context.clone();
        for (key, value) in custom_context {
            context.insert(key, &value);
        }

        match self.tera.render(template_name, &context) {
            Ok(s) => Ok(s),
            Err(e) => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Error rendering template: {:#?}", e),
            ))),
        }
    }
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

        // Split by first comma (CSV format)
        if let Some(pos) = line.find(',') {
            let key = line[..pos].trim().to_string();
            let value = line[(pos + 1)..].trim().to_string();
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
