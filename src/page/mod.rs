use super::*;
use std::path::Path;

impl OreStaty<'_> {
    /// Build an HTML page from the file using Handlebars
    pub fn build_page(&mut self, src: &Path, relative_path: &Path) -> Result<String, ()> {
        // * Read
        let source = self.unwrap_or_error(std::fs::read_to_string(src), "Failed to read file")?;

        // * Build
        let built = self.unwrap_or_error(
            self.handlebars.render_template(
                &source,
                &serde_json::json!({"path": relative_path.to_string_lossy()}),
            ),
            format!("Failed to render {:?} using Handlebars", relative_path),
        )?;

        Ok(built)
    }
}
