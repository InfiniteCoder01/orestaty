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
        let events = pulldown_cmark::Parser::new_ext(&source, self.markdown_options);

        // * Build
        let mut content = String::new();
        let (events, metadata) = {
            let mut accumulating_metadata = false;
            let mut metadata = String::new();
            let events = events
                .into_iter()
                .inspect(|event| {
                    use pulldown_cmark::{Event, MetadataBlockKind::YamlStyle, Tag, TagEnd};
                    match &event {
                        Event::Start(Tag::MetadataBlock(YamlStyle)) => accumulating_metadata = true,
                        Event::End(TagEnd::MetadataBlock(YamlStyle)) => {
                            accumulating_metadata = false
                        }
                        Event::Text(text) => {
                            if accumulating_metadata {
                                metadata.push_str(text);
                            }
                        }
                        _ => (),
                    }
                })
                .collect::<Vec<_>>();
            (
                events,
                self.unwrap_or_error(
                    serde_yml::from_str::<Metadata>(&metadata),
                    "Invalid metadata format",
                )
                .unwrap_or_default(),
            )
        };

        let syntax_highlighting = self.syntax_highlighting.try_lock().unwrap();
        let events = syntax_highlighting.highlight_markdown(events);
        pulldown_cmark::html::push_html(&mut content, events);
        drop(syntax_highlighting);
        self.render_html(
            &self.config.default_markdown_template.clone(),
            &content,
            page::PageData {
                path: relative_path,
            },
            serde_json::json!({
                "metadata": metadata,
                "path": relative_path,
            }),
        )
    }

    /// Register default markdown template
    pub fn register_default_markdown_template(&mut self) {
        self.handlebars
            .register_template_string(
                "default_markdown",
                include_str!("../templates/markdown_template.html"),
            )
            .expect("Failed to register default markdown template! Buggy build");
    }
}
