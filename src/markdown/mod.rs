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
    #[cfg(feature = "highlighting")]
    /// Highlight theme
    pub highlight_theme: Option<String>,
}

impl OreStaty<'_> {
    /// Build a markdown page from the file using pulldown-cmark
    pub fn build_markdown(&mut self, src: &Path, relative_path: &Path) -> Result<String, ()> {
        // * Read
        let source = self.unwrap_or_error(std::fs::read_to_string(src), "Failed to read file")?;
        let events = pulldown_cmark::Parser::new_ext(&source, self.markdown_options);
        #[allow(unused_mut)]
        let mut global_metadata = page::GlobalMetadata {
            #[cfg(feature = "highlighting")]
            highlight_theme: self.config.default_highlight_theme.clone(),
        };

        // * Build
        let mut content = String::new();
        let (events, metadata) = {
            let mut accumulating = false;
            let mut metadata = String::new();
            let events = events
                .into_iter()
                .inspect(|event| {
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

        #[cfg(feature = "highlighting")]
        if let Some(theme) = &metadata.highlight_theme {
            global_metadata.highlight_theme = theme.clone();
        }

        #[cfg(feature = "highlighting")]
        let events = highlight_pulldown::highlight_with_theme(
            events.into_iter(),
            &global_metadata.highlight_theme,
        )
        .unwrap();
        pulldown_cmark::html::push_html(&mut content, events.into_iter());
        self.render_html(
            &self.config.default_markdown_template.clone(),
            &content,
            global_metadata,
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
