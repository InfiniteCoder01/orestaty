use super::*;
use serde::Serialize;
use std::path::Path;

/// Markdown page metadata
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct GlobalMetadata {
    #[cfg(feature = "highlighting")]
    /// Highlight theme
    pub highlight_theme: String,
}

impl OreStaty<'_> {
    /// Render HTML template using Handlebars
    pub fn render_html<T: Serialize>(
        &mut self,
        template: &str,
        content: &str,
        global_metadata: GlobalMetadata,
        params: T,
    ) -> Result<String, ()> {
        #[derive(Debug, Serialize)]
        struct Page<T: Serialize> {
            #[serde(flatten)]
            params: T,
            content: String,
            global_metadata: GlobalMetadata,
        }

        let mut page = Page {
            params,
            content: String::new(),
            global_metadata,
        };
        let content = self.unwrap_or_error(
            self.handlebars.render_template(content, &page),
            "Failed to render page using Handlebars",
        )?;
        page.content = content;
        self.unwrap_or_error(
            self.handlebars.render(template, &page),
            format!("Failed to render page using template {:?}", template),
        )
    }

    /// Build an HTML page from the file using Handlebars
    pub fn build_page(&mut self, src: &Path, relative_path: &Path) -> Result<String, ()> {
        let source = self.unwrap_or_error(std::fs::read_to_string(src), "Failed to read file")?;
        self.render_html(
            &self.config.default_template.clone(),
            &source,
            GlobalMetadata {
                #[cfg(feature = "highlighting")]
                highlight_theme: self.config.default_highlight_theme.clone(),
            },
            serde_json::json!({
                "path": relative_path.to_string_lossy().into_owned(),
            }),
        )
    }
}
