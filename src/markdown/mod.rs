use super::*;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Markdown page metadata
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Metadata {
    /// Page title
    pub title: Option<String>,
    /// Page template
    pub template: Option<String>,
}

impl OreStaty<'_> {
    /// Build a markdown page from the file using pulldown-cmark
    pub fn build_markdown(&mut self, src: &Path, relative_path: &Path) -> Result<String, ()> {
        // * Read
        let source = self.unwrap_or_error(std::fs::read_to_string(src), "Failed to read file")?;
        let parser = pulldown_cmark::Parser::new_ext(&source, self.markdown_options);

        // * Build
        let mut content = String::new();
        let metadata = {
            let mut accumulating = false;
            let mut metadata = String::new();
            pulldown_cmark::html::push_html(
                &mut content,
                parser.into_iter().map(|event| {
                    use pulldown_cmark::{Event, MetadataBlockKind::YamlStyle, Tag, TagEnd};
                    match &event {
                        Event::Start(Tag::MetadataBlock(YamlStyle)) => accumulating = true,
                        Event::End(TagEnd::MetadataBlock(YamlStyle)) => accumulating = false,
                        Event::Text(text) => {
                            if accumulating {
                                metadata.push_str(text);
                            }
                        }
                        _ => (),
                    }
                    event
                }),
            );
            self.unwrap_or_error(
                serde_yml::from_str::<Metadata>(&metadata),
                "Invalid metadata format",
            )
            .unwrap_or_default()
        };

        self.render_html(
            &self.config.default_markdown_template.clone(),
            &content,
            &serde_json::json!({
                "metadata": metadata,
                "path":relative_path,
            }),
        )
    }

    /// Register default markdown templates
    pub fn register_default_markdown_templates(&mut self) {
        self.handlebars
            .register_template_string("default_markdown", include_str!("../templates/markdown_template.html"))
            .expect("Failed to register default markdown template! Buggy build");
    }
}
