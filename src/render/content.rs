use crate::content::Content;
use std::error::Error;

impl Content {
    pub fn render_html(
        &self,
        renderer: &crate::render::Renderer,
    ) -> Result<String, Box<dyn Error>> {
        match self {
            Content::Problem(metadata, problem, solutions, hints) => {
                // Convert FormattedText to HTML strings
                let problem_html = problem.to_html()?;
                let solution_htmls: Vec<String> =
                    solutions.iter().filter_map(|s| s.to_html().ok()).collect();
                let hint_htmls: Vec<String> =
                    hints.iter().filter_map(|h| h.to_html().ok()).collect();

                // Create a context for the template
                use serde_json::json;
                use std::collections::HashMap;

                let mut context = HashMap::new();
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

                // Render the problem template
                renderer
                    .render("problem.html", context)
                    .map_err(|e| e.into())
            }
            Content::Blog(metadata, body) => {
                // Convert body to HTML
                let body_html = body.to_html()?;

                // Create a context for the template
                use serde_json::json;
                use std::collections::HashMap;

                let mut context = HashMap::new();
                context.insert(
                    "blog".to_string(),
                    json!({
                        "title": metadata.title,
                        "id": metadata.id,
                        "tags": metadata.tags,
                        "timestamp": metadata.timestamp,
                        "body": body_html,
                    }),
                );

                // Render the blog template
                renderer.render("blog.html", context).map_err(|e| e.into())
            }
            Content::Page(metadata, body) => {
                // Convert body to HTML
                let body_html = body.to_html()?;

                // Create a context for the template
                use serde_json::json;
                use std::collections::HashMap;

                let mut context = HashMap::new();
                context.insert(
                    "page".to_string(),
                    json!({
                        "title": metadata.title,
                        "id": metadata.id,
                        "body": body_html,
                    }),
                );

                // Render the page template - simpler than blog template
                renderer.render("page.html", context).map_err(|e| e.into())
            }
        }
    }
}
