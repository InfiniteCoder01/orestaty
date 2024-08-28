use super::*;
use handlebars::{
    Context, Handlebars, Helper, HelperResult, Output, RenderContext, RenderErrorReason,
};

/// Highlight helper
pub fn highlight(
    helper: &Helper,
    _: &Handlebars,
    ctx: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let metadata: page::GlobalMetadata = ctx
        .data()
        .get("global_metadata")
        .and_then(|metadata| serde_json::value::from_value(metadata.clone()).ok())
        .ok_or(RenderErrorReason::Other(
            "Failed to get global metadata".to_owned(),
        ))?;
    let code = helper
        .template()
        .and_then(|template| template.elements.first())
        .and_then(|element| {
            if let handlebars::template::TemplateElement::RawString(code) = element {
                Some(code)
            } else {
                None
            }
        })
        .ok_or(RenderErrorReason::ParamNotFoundForName(
            "highlight",
            "\"code\"".to_owned(),
        ))?;

    let ss = syntect::parsing::SyntaxSet::load_defaults_newlines();
    let ts = syntect::highlighting::ThemeSet::load_defaults();

    let syntax = if let Some(syntax) = helper.param(0) {
        let syntax = syntax
            .value()
            .as_str()
            .ok_or(RenderErrorReason::InvalidParamType("string"))?;
        ss.find_syntax_by_name(syntax).ok_or_else(|| {
            RenderErrorReason::Other(format!(
                "Unsupported syntax {}. Supported syntaxes: {}",
                syntax,
                ss.syntaxes()
                    .iter()
                    .map(|syntax| syntax.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        })?
    } else {
        ss.find_syntax_by_first_line(code)
            .unwrap_or(ss.find_syntax_plain_text())
    };

    let theme = &ts.themes.get(&metadata.highlight_theme).ok_or_else(|| {
        RenderErrorReason::Other(format!(
            "Unsupported theme {}. Supported themes: {}",
            metadata.highlight_theme,
            ts.themes
                .keys()
                .map(|name| name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ))
    })?;
    out.write(&syntect::html::highlighted_html_for_string(code, &ss, syntax, theme).unwrap())?;
    Ok(())
}
