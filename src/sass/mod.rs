use super::*;
use std::path::Path;

impl OreStaty<'_> {
    /// Build an HTML page from the file using Handlebars
    pub fn build_sass(
        &mut self,
        src: &Path,
        relative_path: &Path,
    ) -> Result<String, ()> {
        self.unwrap_or_error(
            grass::from_path(src, &self.sass_options),
            format!("Failed to render {:?} using SASS", relative_path),
        )
    }
}
