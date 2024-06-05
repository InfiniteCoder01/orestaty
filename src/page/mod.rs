use std::path::Path;

use super::*;

impl OreStaty<'_> {
    /// Build all sources in source_dir/path and put them into out_dir/path
    pub fn build_sources(&mut self, source_dir: &Path, out_dir: &Path, path: &Path) {
        for file in unwrap_or_error!(
            std::fs::read_dir(source_dir.join(path)),
            self=>err("Failed to read directory {:?}: {}", source_dir, err),
            return
        ) {
            let file = unwrap_or_error!(
                file,
                self=>err("Failed to read file: {}", err),
                continue
            );
            let name = std::path::PathBuf::from(file.file_name());
            let path = path.join(&name);

            if file.path().is_file() {
                self.build_source(
                    &file.path(),
                    &out_dir.join(path.with_extension("html")),
                    &path,
                );
            } else {
                self.build_sources(source_dir, out_dir, &path);
            }
        }
    }

    /// Build the page from the file
    pub fn build_source(&mut self, source_file: &Path, out_file: &Path, path: &Path) {
        // * Read
        let source = unwrap_or_error!(
            std::fs::read_to_string(source_file),
            self=>err("Failed to read file: {}", err),
            return
        );

        // * Build
        let built = match source_file.extension() {
            Some(ext) if ext == "html" || ext == "hbs" => {
                self.build_html(&source, path).unwrap_or(source)
            }
            _ => {
                eprintln!("Warning: {:?} is not a source file, skipping.", source_file);
                return;
            }
        };

        // * Write
        let out_dir = unwrap_or_error!(
            out_file.parent().ok_or(()), self=>_err("Failed to create output directory: No parent path"),
            return
        );
        unwrap_or_error!(
            std::fs::create_dir_all(out_dir),
            self=>err("Failed to create output directory: {}", err),
            ()
        );
        unwrap_or_error!(
            std::fs::write(out_file, built),
            self=>err("Failed to write built HTML page: {}", err),
            ()
        );
    }

    /// Build the HTML page from the file
    pub fn build_html(&mut self, src: &str, path: &Path) -> Option<String> {
        Some(unwrap_or_error!(
            self
                .handlebars
                .render_template(src, &serde_json::json!({"path": path.to_string_lossy()})),
            self=>err("Failed to render HTML: {}", err),
            return None
        ))
    }
}
