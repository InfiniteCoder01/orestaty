use super::*;
use std::path::Path;

impl OreStaty<'_> {
    /// Build an HTML page from the file using Handlebars
    pub fn build_sass(&mut self, src: &Path, dst: &Path, relative_path: &Path) -> Result<(), ()> {
        // * Build
        let built = self.unwrap_or_error(
            grass::from_path(src, &self.sass_options),
            format!("Failed to render {:?} using SASS", relative_path),
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
                "Failed to write built CSS for file {:?}",
                relative_path
            ),
        )
    }
}
