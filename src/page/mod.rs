use super::*;
use serde::Serialize;
use std::path::Path;

impl OreStaty<'_> {
    /// Render HTML template using Handlebars
    pub fn render_html<T: Serialize>(
        &mut self,
        template: &str,
        content: &str,
        params: T,
    ) -> Result<String, ()> {
        #[derive(Debug, Serialize)]
        struct Page<'a, T: Serialize> {
            #[serde(flatten)]
            params: T,
            content: &'a str,
        }

        let content = self.unwrap_or_error(
            self.handlebars.render_template(content, &params),
            "Failed to render page using Handlebars",
        )?;
        self.unwrap_or_error(
            self.handlebars.render(
                template,
                &Page {
                    params,
                    content: &content,
                },
            ),
            format!("Failed to render page using template {:?}", template),
        )
    }

    /// Build an HTML page from the file using Handlebars
    pub fn build_page(&mut self, src: &Path, relative_path: &Path) -> Result<String, ()> {
        let source = self.unwrap_or_error(std::fs::read_to_string(src), "Failed to read file")?;
        self.render_html(
            &self.config.default_template.clone(),
            &source,
            serde_json::json!({
                "path": relative_path.to_string_lossy().into_owned(),
            }),
        )
    }
}
