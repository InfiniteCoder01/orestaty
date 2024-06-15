use super::*;
use std::path::Path;

impl OreStaty<'_> {
    /// Build an HTML page from the file using Handlebars
    pub fn build_page(&mut self, src: &Path, dst: &Path, relative_path: &Path) -> Result<(), ()> {
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

        // * Write
        if let Ok(out_dir) = self.unwrap_or_error(
            dst.parent().ok_or("No parent path"),
            format!(
                "Failed to create output directory for file {:?}",
                relative_path
            ),
        ) {
            self.unwrap_or_error(
                std::fs::create_dir_all(out_dir),
                format!(
                    "Failed to create output directory for file {:?}",
                    relative_path,
                ),
            )
            .ok();
        }

        self.unwrap_or_error(
            std::fs::write(dst, built),
            format!(
                "Failed to write built HTML page for file {:?}",
                relative_path
            ),
        )
    }
}
