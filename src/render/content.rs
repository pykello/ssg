use crate::content::Content;
use serde_json::json;
use std::collections::HashMap;
use std::error::Error;

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
                // Convert FormattedText to HTML strings
                let problem_html = statement.to_html(&config)?;
                let solution_htmls: Vec<String> = solutions
                    .iter()
                    .filter_map(|s| s.to_html(&config).ok())
                    .collect();
                let hint_htmls: Vec<String> = hints
                    .iter()
                    .filter_map(|h| h.to_html(&config).ok())
                    .collect();

                let mut context = HashMap::new();
                let template = if let Some(template) = &metadata.template {
                    template.clone()
                } else {
                    "problem.html".to_string()
                };
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

                if let Some(additional_context) = metadata.context.clone() {
                    for (key, value) in additional_context {
                        context.insert(key, json!(value));
                    }
                }

                // Render the problem template
                renderer.render(&template, context).map_err(|e| e.into())
            }
            Content::Blog { metadata, body } => {
                let body_html = body.to_html(&config)?;
                let mut context = HashMap::new();
                let template = if let Some(template) = &metadata.template {
                    template.clone()
                } else {
                    "blog.html".to_string()
                };
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

                if let Some(additional_context) = metadata.context.clone() {
                    for (key, value) in additional_context {
                        context.insert(key, json!(value));
                    }
                }

                // Render the blog template
                renderer.render(&template, context).map_err(|e| e.into())
            }
            Content::Page { metadata, body } => {
                let body_html = body.to_html(&config)?;
                let mut context = HashMap::new();
                let template = if let Some(template) = &metadata.template {
                    template.clone()
                } else {
                    "page.html".to_string()
                };
                context.insert(
                    "page".to_string(),
                    json!({
                        "title": metadata.title,
                        "id": metadata.id,
                        "body": body_html,
                    }),
                );
                context.insert("title".to_string(), json!(metadata.title.clone()));

                if let Some(additional_context) = metadata.context.clone() {
                    for (key, value) in additional_context {
                        context.insert(key, json!(value));
                    }
                }

                // Render the page template - simpler than blog template
                renderer.render(&template, context).map_err(|e| e.into())
            }
        }
    }
}
