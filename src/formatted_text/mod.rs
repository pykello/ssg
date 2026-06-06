#[allow(clippy::module_inception)]
mod formatted_text;
mod geomdsl;
mod learning;
mod markdown_expandable;
mod markdown_math;
mod pandoc_latex_filters;
mod shell;

pub use formatted_text::FormattedText;
pub use formatted_text::Theorem;
pub use geomdsl::preprocess_geomdsl_blocks;
pub use learning::preprocess_learning_blocks;
