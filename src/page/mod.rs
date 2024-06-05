use super::*;
use std::path::Path;

impl OreStaty<'_> {
    /// Build an HTML page from the file
    pub fn build_source(&mut self, file: &Path, out_file: &Path, path: &Path) {
        // * Read
        macro_rules! source {
            () => {
                unwrap_or_error!(
                    std::fs::read_to_string(file),
                    self=>err("Failed to read file: {}", err),
                    return
                )
            }
        }

        // * Build
        let built = match file.extension() {
            Some(ext) if ext == "html" || ext == "hbs" => {
                let source = source!();
                self.build_html(&source, path).unwrap_or(source)
            }
            _ => {
                eprintln!("Warning: {:?} is not a source file, skipping.", file);
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

    /// Build an HTML page from handlebars source
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
