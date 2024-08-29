use super::*;

/// Syntax highlighting
pub mod syntax_highlighting;

impl OreStaty<'_> {
    /// Register built-in plugins
    pub fn register_builtin_plugins(&mut self) {
        self.handlebars.register_helper(
            "highlight",
            Box::new(syntax_highlighting::HighlightHelper(
                self.syntax_highlighting.clone(),
            )),
        );
    }
}
