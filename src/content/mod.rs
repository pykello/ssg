#![allow(dead_code)]

#[allow(clippy::module_inception)]
mod content;
mod metadata;
mod problem;
pub mod test;

pub use content::{content_url, Content};
pub use metadata::{ContentKind, ContentMetadata};
