#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

pub use lol_html;

/// Create a new [`lol_html::HtmlRewriter`] setup to use orestaty.
pub fn rewriter<O: lol_html::OutputSink>(output_sink: O) -> lol_html::HtmlRewriter<'static, O> {
    lol_html::HtmlRewriter::new(
        lol_html::Settings {
            element_content_handlers: vec![lol_html::text!("*", |t| {
                dbg!(t);

                Ok(())
            })],
            ..lol_html::Settings::default()
        },
        output_sink,
    )
}
