use super::*;

#[cfg(feature = "highlighting")]
/// Syntax highlighting
pub mod highlighting;

impl OreStaty<'_> {
    /// Register built-in plugins
    pub fn register_builtin_plugins(&mut self) {
        #[cfg(feature = "highlighting")]
        self.handlebars
            .register_helper("highlight", Box::new(highlighting::highlight));
    }
}
