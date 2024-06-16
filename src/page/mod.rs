use super::*;
use std::path::Path;

impl OreStaty<'_> {
    /// Render HTML template using Handlebars
    pub fn render_html(&mut self, src: &str, relative_path: &Path) -> Result<String, ()> {
        self.unwrap_or_error(
            self.handlebars.render_template(
                src,
                &serde_json::json!({"path": relative_path.to_string_lossy()}),
            ),
            format!("Failed to render {:?} using Handlebars", relative_path),
        )
    }

    /// Build an HTML page from the file using Handlebars
    pub fn build_page(&mut self, src: &Path, relative_path: &Path) -> Result<String, ()> {
        let source = self.unwrap_or_error(std::fs::read_to_string(src), "Failed to read file")?;
        self.render_html(&source, relative_path)
    }
}
