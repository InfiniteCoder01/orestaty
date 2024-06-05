#[warn(missing_docs)]
#[doc = include_str!("../README.md")]

pub mod files;
pub mod page;
pub mod sass;

pub struct OreStaty<'a> {
    pub handlebars: handlebars::Handlebars<'a>,
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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn error(&mut self, message: &str) -> &mut Self {
        eprintln!("{}", message);
        self.errors += 1;
        self
    }

    pub fn errors(&self) -> u32 {
        self.errors
    }
}

pub fn build(generator: &mut OreStaty, path: &std::path::Path, output: &std::path::Path) {
    // * Build src
    let src_path = path.join("src");
    if src_path.exists() {
        files::walk_recursively(generator, &src_path, std::path::Path::new(""), &mut |generator, file, path| {
            generator.build_source(file, &output.join(path).with_extension("html"), path);
        });
    }

    // * Build sass/scss
    let sass_path = path.join("sass");
    if sass_path.exists() {
        files::walk_recursively(generator, &sass_path, std::path::Path::new(""), &mut |generator, file, path| {
            generator.build_sass_source(file, &output.join(path).with_extension("css"));
        });
    }

    // * Copy static
    let static_path = path.join("static");
    if static_path.exists() {
        if let Err(err) = files::copy_recursively(static_path, output) {
            eprintln!("Failed to copy static: {}", err);
        }
    }
}

#[macro_export]
macro_rules! unwrap_or_error {
    ($expr:expr, $self:expr => $err:ident ($($args:tt)+), $action:expr) => {
        match $expr {
            Ok(expr) => expr,
            Err($err) => {
                $self.error(&format!($($args)+));
                $action
            }
        }
    };
}
