use super::*;
use std::path::Path;

impl OreStaty<'_> {
    pub fn build_sass_source(&mut self, file: &Path, out_file: &Path) {
        let built = unwrap_or_error!(
            grass::from_path(file, &self.sass_options),
            self=>err("Failed to compile sass: {}", err),
            return
        );

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
}
