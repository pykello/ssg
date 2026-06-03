use crate::content::Content;
use crate::content::ContentMetadata;
use crate::formatted_text::FormattedText;
use serde_json::json;
use std::collections::HashMap;
use std::error::Error;

fn choose_template(template: &Option<String>, default: &str) -> String {
    template.clone().unwrap_or_else(|| default.to_string())
}

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

fn context_with_title(metadata: &ContentMetadata) -> HashMap<String, serde_json::Value> {
    let mut context = HashMap::new();
    context.insert("title".to_string(), json!(metadata.title.clone()));
    merge_additional_context(&mut context, &metadata.context);
    context
}

fn rendered_sections(sections: &[FormattedText], config: &crate::config::Config) -> Vec<String> {
    sections
        .iter()
        .filter_map(|section| section.to_html(config).ok())
        .collect()
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
            } => render_problem(renderer, config, metadata, statement, solutions, hints),
            Content::Blog { metadata, body } => render_blog(renderer, config, metadata, body),
            Content::Page { metadata, body } => render_page(renderer, config, metadata, body),
        }
    }
}

fn render_problem(
    renderer: &crate::render::Renderer,
    config: &crate::config::Config,
    metadata: &ContentMetadata,
    statement: &FormattedText,
    solutions: &[FormattedText],
    hints: &[FormattedText],
) -> Result<String, Box<dyn Error>> {
    let mut context = context_with_title(metadata);
    context.insert(
        "problem".to_string(),
        json!({
            "title": metadata.title,
            "id": metadata.id,
            "tags": metadata.tags,
            "timestamp": metadata.timestamp,
            "statement": statement.to_html(config)?,
            "solutions": rendered_sections(solutions, config),
            "hints": rendered_sections(hints, config),
            "image": metadata.image,
        }),
    );

    renderer.render(
        &choose_template(&metadata.template, "problem.html"),
        context,
    )
}

fn render_blog(
    renderer: &crate::render::Renderer,
    config: &crate::config::Config,
    metadata: &ContentMetadata,
    body: &FormattedText,
) -> Result<String, Box<dyn Error>> {
    let mut context = context_with_title(metadata);
    context.insert(
        "blog".to_string(),
        json!({
            "title": metadata.title,
            "id": metadata.id,
            "tags": metadata.tags,
            "timestamp": metadata.timestamp,
            "body": body.to_html(config)?,
            "author": metadata.author,
        }),
    );

    renderer.render(&choose_template(&metadata.template, "blog.html"), context)
}

fn render_page(
    renderer: &crate::render::Renderer,
    config: &crate::config::Config,
    metadata: &ContentMetadata,
    body: &FormattedText,
) -> Result<String, Box<dyn Error>> {
    let mut context = context_with_title(metadata);
    context.insert(
        "page".to_string(),
        json!({
            "title": metadata.title,
            "id": metadata.id,
            "body": body.to_html(config)?,
        }),
    );

    renderer.render(&choose_template(&metadata.template, "page.html"), context)
}
