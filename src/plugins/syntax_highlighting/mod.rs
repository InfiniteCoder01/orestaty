use super::*;
use handlebars::RenderErrorReason;

const PREFIX: &str = "z-";

/// Syntax highlighting theme
#[derive(Clone, Debug, Default, PartialEq)]
pub enum Theme {
    /// css-classed
    #[default]
    CSSClassed,
    /// Syntect theme
    Syntect(Box<syntect::highlighting::Theme>),
}

/// Syntax highlighting context
#[derive(Debug)]
pub struct SyntaxHighlighting {
    /// Syntax set
    pub syntax_set: syntect::parsing::SyntaxSet,
    theme: Theme,
}

impl Default for SyntaxHighlighting {
    fn default() -> Self {
        Self::new("InspiredGitHub", "".as_ref())
            .expect("Could not initialize syntax highlighting with default theme! Buggy build!")
    }
}

impl SyntaxHighlighting {
    /// Create new syntax highlighting context
    pub fn new(theme: &str, root_path: &Path) -> Option<Self> {
        use syntect::highlighting::ThemeSet;

        let theme = if theme == "css-classed" {
            Theme::CSSClassed
        } else {
            let mut theme_set = ThemeSet::load_defaults();
            let theme = if let Some(theme) = theme_set.themes.remove(theme) {
                theme
            } else {
                let path = root_path.join(theme);
                if std::fs::exists(&path).is_ok_and(|exists| exists) {
                    let theme = std::fs::File::open(path)
                        .map_err(|err| eprintln!("Failed to open theme file: {}", err))
                        .and_then(|theme_file| {
                            syntect::highlighting::ThemeSet::load_from_reader(
                                &mut std::io::BufReader::new(theme_file),
                            )
                            .map_err(|err| eprintln!("Failed to load theme file: {}", err))
                        });
                    theme.ok()
                } else {
                    eprintln!(
                        "Theme {theme:?} could not be found, as it is not built-in, nor a file. Built in themes: {}",
                        theme_set.themes.keys().map(|name|name.as_str()).chain(std::iter::once("css-classed")).collect::<Vec<_>>().join(", "),
                    );
                    None
                }?
            };
            Theme::Syntect(Box::new(theme))
        };
        Some(Self {
            syntax_set: syntect::parsing::SyntaxSet::load_defaults_newlines(),
            theme,
        })
    }

    /// Highlight a piece of code with the hint of it's syntax
    pub fn highlight(
        &self,
        code: &str,
        syntax_hint: Option<&syntect::parsing::SyntaxReference>,
    ) -> Result<String, syntect::Error> {
        let syntax = syntax_hint.unwrap_or_else(|| {
            self.syntax_set
                .find_syntax_by_first_line(code)
                .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text())
        });

        match &self.theme {
            Theme::CSSClassed => {
                let mut html_generator = syntect::html::ClassedHTMLGenerator::new_with_class_style(
                    syntax,
                    &self.syntax_set,
                    syntect::html::ClassStyle::SpacedPrefixed { prefix: PREFIX },
                );
                for line in syntect::util::LinesWithEndings::from(code) {
                    html_generator.parse_html_for_line_which_includes_newline(line)?;
                }
                Ok(format!(
                    "<pre><code>{}</code></pre>",
                    html_generator.finalize()
                ))
            }
            Theme::Syntect(theme) => {
                syntect::html::highlighted_html_for_string(code, &self.syntax_set, syntax, theme)
            }
        }
    }

    /// Highlight a piece of code with the hint of it's syntax, fallback to it without highlighting
    pub fn highlight_or_fallback(
        &self,
        code: &str,
        syntax_hint: Option<&syntect::parsing::SyntaxReference>,
    ) -> String {
        self.highlight(code, syntax_hint).unwrap_or_else(|err| {
            eprintln!("Warning: Failed to highlight: {}", err);
            format!("<pre><code>{code}</code></pre>")
        })
    }

    /// Export theme as CSS
    pub fn export_theme(&self) -> Option<String> {
        match &self.theme {
            Theme::CSSClassed => {
                eprintln!("css-classed theme can't be exported as CSS!");
                None
            }
            Theme::Syntect(theme) => syntect::html::css_for_theme_with_class_style(
                theme,
                syntect::html::ClassStyle::SpacedPrefixed { prefix: PREFIX },
            )
            .map_err(|err| eprintln!("Failed exporting CSS for the theme: {}", err))
            .ok(),
        }
    }
}

/// Highlight helper
pub struct HighlightHelper(pub ArcMutex<SyntaxHighlighting>);

impl handlebars::HelperDef for HighlightHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        helper: &handlebars::Helper<'rc>,
        _: &'reg handlebars::Handlebars<'reg>,
        _: &'rc handlebars::Context,
        _: &mut handlebars::RenderContext<'reg, 'rc>,
        output: &mut dyn handlebars::Output,
    ) -> handlebars::HelperResult {
        let code = helper
            .template()
            .and_then(|template| template.elements.first())
            .and_then(|element| {
                if let handlebars::template::TemplateElement::RawString(code) = element {
                    Some(unindent::unindent(code.trim()))
                } else {
                    None
                }
            })
            .ok_or(RenderErrorReason::ParamNotFoundForName(
                "highlight",
                "\"code\"".to_owned(),
            ))?;

        let syntax_highlighing = self.0.try_lock().unwrap();
        let syntax_hint = if let Some(syntax) = helper.param(0) {
            let syntax = syntax
                .value()
                .as_str()
                .ok_or(RenderErrorReason::InvalidParamType("string"))?;
            Some(
                syntax_highlighing
                    .syntax_set
                    .find_syntax_by_token(syntax)
                    .ok_or_else(|| {
                        RenderErrorReason::Other(format!(
                            "Unsupported syntax {}. Supported syntaxes: {}",
                            syntax,
                            syntax_highlighing
                                .syntax_set
                                .syntaxes()
                                .iter()
                                .map(|syntax| syntax.name.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        ))
                    })?,
            )
        } else {
            None
        };

        output.write(&syntax_highlighing.highlight_or_fallback(&code, syntax_hint))?;
        Ok(())
    }
}

impl SyntaxHighlighting {
    /// Highlight all code in this markdown event stream
    pub fn highlight_markdown<'a, 'e>(
        &'a self,
        events: impl IntoIterator<Item = pulldown_cmark::Event<'e>> + 'a,
    ) -> impl Iterator<Item = pulldown_cmark::Event<'e>> + 'a {
        use pulldown_cmark::{Event, Tag, TagEnd};

        struct HighlightedCode<'a> {
            code: String,
            syntax_hint: Option<&'a syntect::parsing::SyntaxReference>,
        }
        let mut highlighted_code = None;

        events.into_iter().filter_map(move |event| match event {
            pulldown_cmark::Event::Start(Tag::CodeBlock(kind)) => {
                use pulldown_cmark::CodeBlockKind;
                let syntax_hint = match kind {
                    CodeBlockKind::Fenced(lang) => self.syntax_set.find_syntax_by_token(&lang),
                    CodeBlockKind::Indented => None,
                };
                highlighted_code = Some(HighlightedCode {
                    code: String::new(),
                    syntax_hint,
                });
                None
            }
            Event::End(TagEnd::CodeBlock) => {
                let Some(highlighted_code) = highlighted_code.take() else {
                    unreachable!("Code block ends without starting! Buggy build!");
                };
                Some(Event::Html(pulldown_cmark::CowStr::from(
                    self.highlight_or_fallback(
                        &highlighted_code.code,
                        highlighted_code.syntax_hint,
                    ),
                )))
            }
            Event::Text(text) => {
                if let Some(highlighted_code) = &mut highlighted_code {
                    highlighted_code.code.push_str(&text);
                    None
                } else {
                    Some(Event::Text(text))
                }
            }
            event => Some(event),
        })
    }
}
