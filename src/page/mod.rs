use super::*;
use serde::Serialize;
use std::path::Path;

/// Information about the processed page that gets sent to template as `page`
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct PageData<'a> {
    /// Path of the page
    pub path: &'a std::path::Path,
}

impl OreStaty<'_> {
    /// Render HTML template using Handlebars
    pub fn render_html<T: Serialize>(
        &mut self,
        template: &str,
        content: &str,
        page_data: PageData,
        params: T,
    ) -> Result<String, ()> {
        #[derive(Debug, Serialize)]
        struct Page<'a, T: Serialize> {
            #[serde(flatten)]
            params: T,
            page: PageData<'a>,
        }

        #[derive(Debug, Serialize)]
        struct PageWithContent<'a, T: Serialize> {
            #[serde(flatten)]
            page: Page<'a, T>,
            content: String,
        }

        let page = Page {
            params,
            page: page_data,
        };
        let content = self.unwrap_or_error(
            self.handlebars.render_template(content, &page),
            "Failed to render page using Handlebars",
        )?;
        let page = PageWithContent { page, content };
        self.unwrap_or_error(
            self.handlebars.render(template, &page),
            format!("Failed to render page using template {:?}", template),
        )
    }

    /// Build an HTML page from the file using Handlebars
    pub fn build_page(&mut self, src: &Path, relative_path: &Path) -> Result<String, ()> {
        let source = self.unwrap_or_error(std::fs::read_to_string(src), "Failed to read file")?;
        self.render_html(
            &self.config.default_template.clone(),
            &source,
            PageData {
                path: relative_path,
            },
            (),
        )
    }
}
