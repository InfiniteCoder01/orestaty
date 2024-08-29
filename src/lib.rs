#![warn(missing_docs)]
#![doc = include_str!("../README.md")]
#![allow(clippy::doc_lazy_continuation)]
#![allow(clippy::result_unit_err)]

use serde::{Deserialize, Serialize};
use std::path::Path;

pub use handlebars;
pub use serde;
pub use serde_json;

pub use grass;
pub use pulldown_cmark;

/// File utilities
pub mod files;
/// Build HTML page with Markdown
pub mod markdown;
/// Build HTML page with Handlebars
pub mod page;
/// Build CSS with SASS
pub mod sass;

/// Built-in plugins
pub mod plugins;

type ArcMutex<T> = std::sync::Arc<std::sync::Mutex<T>>;

/// Generator struct, see [`Self::build`]
#[derive(Debug)]
pub struct OreStaty<'a> {
    /// Handlebars renderer
    pub handlebars: handlebars::Handlebars<'a>,
    /// Syntax highlighting context
    pub syntax_highlighting: ArcMutex<plugins::syntax_highlighting::SyntaxHighlighting>,
    /// SASS rendering options
    pub sass_options: grass::Options<'a>,
    /// Markdown (Commonmark) rendering options
    pub markdown_options: pulldown_cmark::Options,
    /// Config
    config: Config,
    errors: u32,
}

fn default_template() -> String {
    "default".to_owned()
}

fn default_markdown_template() -> String {
    "default_markdown".to_owned()
}

fn default_highlight_theme() -> String {
    "css-classed".to_owned()
}

/// Generator config
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Config {
    /// Default tempate
    #[serde(default = "default_template")]
    pub default_template: String,
    /// Default markdown template
    #[serde(default = "default_markdown_template")]
    pub default_markdown_template: String,

    /// Default highlight theme
    #[serde(default = "default_highlight_theme")]
    pub code_highlight_theme: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_template: default_template(),
            default_markdown_template: default_markdown_template(),
            code_highlight_theme: default_highlight_theme(),
        }
    }
}

impl Default for OreStaty<'_> {
    fn default() -> Self {
        Self::new(Config::default(), "".as_ref())
    }
}

impl OreStaty<'_> {
    /// Create a new generator with default parameters
    pub fn new(config: Config, root_path: &Path) -> Self {
        let mut handlebars = handlebars::Handlebars::new();
        handlebars
            .register_template_string("default", "{{{content}}}")
            .expect("Failed to register default template! Buggy build");
        Self {
            handlebars,
            syntax_highlighting: ArcMutex::new(
                plugins::syntax_highlighting::SyntaxHighlighting::new(
                    &config.code_highlight_theme,
                    root_path,
                )
                .unwrap_or_default()
                .into(),
            ),
            sass_options: grass::Options::default(),
            markdown_options: pulldown_cmark::Options::all(),
            config,
            errors: 0,
        }
    }

    /// Get config of the generator
    pub fn config(&self) -> &Config {
        &self.config
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
                let Ok((built, extension)) = self.build_file(&file.path(), &relative_path) else {
                    continue;
                };
                self.write_file(&built, &dst.with_extension(extension)).ok();
            } else {
                self.build_dir(&file.path(), &dst, &relative_path)?;
            }
        }
        Ok(())
    }

    /// Build a single source file
    pub fn build_file(
        &mut self,
        src: &Path,
        relative_path: &Path,
    ) -> Result<(String, &'static str), ()> {
        match src
            .extension()
            .map_or(std::borrow::Cow::Borrowed(""), |ext| ext.to_string_lossy())
            .as_ref()
        {
            "html" | "htm" | "hbs" => self
                .build_page(src, relative_path)
                .map(|built| (built, "html")),
            "sass" | "scss" | "css" => self
                .build_sass(src, relative_path)
                .map(|built| (built, "css")),
            "md" | "markdown" => self
                .build_markdown(src, relative_path)
                .map(|built| (built, "html")),
            ext => {
                eprintln!("Warning: {:?} file extension is unknown. Skipping {:?}; Maybe you wanted to put it into \"static\" directory?", ext, src);
                Err(())
            }
        }
    }

    /// Write a file, reporting if error
    pub fn write_file(&mut self, content: &str, dst: &Path) -> Result<(), ()> {
        if let Ok(out_dir) = self.unwrap_or_error(
            dst.parent().ok_or("No parent path"),
            format!("Failed to create output directory for file {:?}", dst),
        ) {
            self.unwrap_or_error(
                std::fs::create_dir_all(out_dir),
                format!("Failed to create output directory for file {:?}", dst,),
            )
            .ok();
        }

        self.unwrap_or_error(
            std::fs::write(dst, content),
            format!("Failed to write built HTML page for file {:?}", dst),
        )
    }
}

impl OreStaty<'_> {
    /// Load plugin helpers and templates from specified path. Set scope to an empty string if
    /// loading from root
    pub fn load_plugins(&mut self, path: &Path, scope: &str) -> Result<(), ()> {
        for file in self.unwrap_or_error(
            std::fs::read_dir(path),
            format!("Failed to read plugin directory {:?}", path),
        )? {
            let Ok(file) = self.unwrap_or_error(file, "Failed to read file") else {
                continue;
            };

            let name = file.file_name();
            let file = file.path();
            let name = file
                .file_stem()
                .unwrap_or(name.as_os_str())
                .to_string_lossy();
            let name = if scope.is_empty() {
                name.into_owned()
            } else {
                format!("{}.{}", scope, name)
            };

            if file.is_file() {
                match file
                    .extension()
                    .map_or(String::new(), |ext| ext.to_string_lossy().into_owned())
                    .as_str()
                {
                    "html" | "htm" | "hbs" => {
                        let result = self.handlebars.register_template_file(&name, &file);
                        self.unwrap_or_error(
                            result,
                            format!("Failed to register {:?} as Handlebars template", file),
                        )
                        .unwrap_or(())
                    }
                    "rhai" => {
                        let result = self
                            .handlebars
                            .register_script_helper_file(&name.replace('.', "_"), &file);
                        self.unwrap_or_error(
                            result,
                            format!("Failed to register {:?} as Handlebars template", file),
                        )
                        .unwrap_or(())
                    }
                    // ext => eprintln!(
                    //     "Warning: {:?} file extension is unknown. Skipping {:?}",
                    //     ext, file
                    // ),
                    _ => (),
                }
            } else {
                self.load_plugins(&file, &name).ok();
            }
        }
        Ok(())
    }
}
