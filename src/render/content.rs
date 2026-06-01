use crate::content::Content;
use serde_json::json;
use std::collections::HashMap;
use std::error::Error;

/// Choose template name, falling back to the provided default.
fn choose_template(template: &Option<String>, default: &str) -> String {
    template.clone().unwrap_or_else(|| default.to_string())
}

/// Merge per-item `context:` from metadata into the render context (if present).
fn merge_additional_context(
    context: &mut HashMap<String, serde_json::Value>,
    additional: &Option<HashMap<String, serde_yaml::Value>>,
) {
    if let Some(add) = additional.clone() {
        for (key, value) in add {
            context.insert(key, json!(value));
        }
    }
}

impl Content {
    pub fn render_html(
        &self,
        renderer: &crate::render::Renderer,
        config: &crate::config::Config,
    ) -> Result<String, Box<dyn Error>> {
        match self {
            Content::Problem {
                metadata,
                statement,
                solutions,
                hints,
            } => {
                let problem_html = statement.to_html(config)?;
                let solution_htmls: Vec<String> = solutions
                    .iter()
                    .filter_map(|s| s.to_html(config).ok())
                    .collect();
                let hint_htmls: Vec<String> = hints
                    .iter()
                    .filter_map(|h| h.to_html(config).ok())
                    .collect();

                let mut context = HashMap::new();
                let template = choose_template(&metadata.template, "problem.html");
                context.insert(
                    "problem".to_string(),
                    json!({
                        "title": metadata.title,
                        "id": metadata.id,
                        "tags": metadata.tags,
                        "timestamp": metadata.timestamp,
                        "statement": problem_html,
                        "solutions": solution_htmls,
                        "hints": hint_htmls,
                        "image": metadata.image,
                    }),
                );
                context.insert("title".to_string(), json!(metadata.title.clone()));
                merge_additional_context(&mut context, &metadata.context);

                renderer.render(&template, context)
            }
            Content::Blog { metadata, body } => {
                let body_html = body.to_html(config)?;

                let mut context = HashMap::new();
                let template = choose_template(&metadata.template, "blog.html");
                context.insert(
                    "blog".to_string(),
                    json!({
                        "title": metadata.title,
                        "id": metadata.id,
                        "tags": metadata.tags,
                        "timestamp": metadata.timestamp,
                        "body": body_html,
                        "author": metadata.author,
                    }),
                );
                context.insert("title".to_string(), json!(metadata.title.clone()));
                merge_additional_context(&mut context, &metadata.context);

                renderer.render(&template, context)
            }
            Content::Page { metadata, body } => {
                let body_html = body.to_html(config)?;

                let mut context = HashMap::new();
                let template = choose_template(&metadata.template, "page.html");
                context.insert(
                    "page".to_string(),
                    json!({
                        "title": metadata.title,
                        "id": metadata.id,
                        "body": body_html,
                    }),
                );
                context.insert("title".to_string(), json!(metadata.title.clone()));
                merge_additional_context(&mut context, &metadata.context);

                renderer.render(&template, context)
            }
        }
    }
}
