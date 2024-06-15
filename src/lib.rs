#![warn(missing_docs)]
#![doc = include_str!("../README.md")]
#![allow(clippy::result_unit_err)]

use std::path::Path;

/// File utilities
pub mod files;
/// Build HTML page with Handlebars
pub mod page;
/// Build CSS with SASS
pub mod sass;

/// Generator struct, see [`Self::build`]
pub struct OreStaty<'a> {
    /// Handlebars renderer
    pub handlebars: handlebars::Handlebars<'a>,
    /// SASS rendering options
    pub sass_options: grass::Options<'a>,
    errors: u32,
}

impl Default for OreStaty<'_> {
    fn default() -> Self {
        Self {
            handlebars: handlebars::Handlebars::new(),
            sass_options: grass::Options::default(),
            errors: 0,
        }
    }
}

impl OreStaty<'_> {
    /// Create a new generator with default parameters
    pub fn new() -> Self {
        Self::default()
    }

    /// Report an error
    pub fn error(&mut self, message: &str) -> &mut Self {
        eprintln!("{}", message);
        self.errors += 1;
        self
    }

    /// "Unwrap" a result, report if error
    pub fn unwrap_or_error<T, E: std::fmt::Display>(
        &mut self,
        result: Result<T, E>,
        message: impl AsRef<str>,
    ) -> Result<T, ()> {
        match result {
            Ok(result) => Ok(result),
            Err(err) => {
                self.error(&format!("{}: {}", message.as_ref(), err));
                Err(())
            }
        }
    }

    /// Get the number of errors reported during build
    pub fn errors(&self) -> u32 {
        self.errors
    }
}

impl OreStaty<'_> {
    /// Build all sources in the given path, outputting to the given destination
    pub fn build(&mut self, src: &std::path::Path, dst: &std::path::Path) {
        self.build_dir(src, dst, Path::new("")).unwrap();
    }

    /// Build all sources in the given directory, with it's relative path specified
    pub fn build_dir(&mut self, src: &Path, dst: &Path, relative_path: &Path) -> Result<(), ()> {
        for file in self.unwrap_or_error(
            std::fs::read_dir(src),
            format!("Failed to read directory {:?}", src),
        )? {
            let Ok(file) = self.unwrap_or_error(file, "Failed to read file") else {
                continue;
            };
            let name = std::path::PathBuf::from(file.file_name());
            let dst = dst.join(&name);
            let relative_path = relative_path.join(&name);

            if file.path().is_file() {
                match file
                    .path()
                    .extension()
                    .map_or(std::borrow::Cow::Borrowed(""), |ext| ext.to_string_lossy())
                    .as_ref()
                {
                    "html" | "htm" | "hbs" => self.build_page(&file.path(), &dst.with_extension("html"), &relative_path).unwrap_or(()),
                    "sass" | "scss" | "css" => self.build_sass(&file.path(), &dst.with_extension("css"), &relative_path).unwrap_or(()),
                    ext => eprintln!("Warning: {:?} file extension is unknown. Skipping {:?}; Maybe you wanted to put it into \"static\" directory?", ext, file.path()),
                }
            } else {
                self.build_dir(&file.path(), &dst, &relative_path)?;
            }
        }
        Ok(())
    }
}
