#[warn(missing_docs)]
#[doc = include_str!("../README.md")]

pub mod page;

pub struct OreStaty<'a> {
    handlebars: handlebars::Handlebars<'a>,
    errors: u32,
}

impl Default for OreStaty<'_> {
    fn default() -> Self {
        Self {
            handlebars: handlebars::Handlebars::new(),
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
        generator.build_sources(&src_path, output, std::path::Path::new(""));
    }

    // * Copy static
    let static_path = path.join("static");
    if static_path.exists() {
        if let Err(err) = copy_dir_all(static_path, output) {
            eprintln!("Failed to copy static: {}", err);
        }
    }
}

// * Utils
fn copy_dir_all(
    src: impl AsRef<std::path::Path>,
    dst: impl AsRef<std::path::Path>,
) -> std::io::Result<()> {
    std::fs::create_dir_all(&dst)?;
    for entry in std::fs::read_dir(src)? {
        let copy = || -> std::io::Result<()> {
            let entry = entry?;
            let ty = entry.file_type()?;
            if ty.is_dir() {
                copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
            } else {
                std::fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
            }
            Ok(())
        };
        if let Err(err) = copy() {
            eprintln!("Failed to copy asset: {}", err);
            continue;
        };
    }
    Ok(())
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
