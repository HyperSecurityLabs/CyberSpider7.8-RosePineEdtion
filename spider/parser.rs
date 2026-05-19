use scraper::{Html, Selector};
use std::collections::HashSet;

pub struct HtmlParser {
    title_selector: Selector,
    link_selector: Selector,
    script_selector: Selector,
    form_selector: Selector,
}

impl HtmlParser {
    pub fn new() -> Self {
        Self {
            title_selector: Selector::parse("title").unwrap(),
            link_selector: Selector::parse("a[href]").unwrap(),
            script_selector: Selector::parse("script[src]").unwrap(),
            form_selector: Selector::parse("form[action]").unwrap(),
        }
    }

    pub fn parse_title(&self, html: &str) -> Option<String> {
        let document = Html::parse_document(html);
        document.select(&self.title_selector)
            .next()
            .map(|element| element.inner_html().trim().to_string())
    }

    pub fn extract_links(&self, html: &str) -> Vec<String> {
        let mut links = HashSet::new();
        let document = Html::parse_document(html);

        // Extract links from anchor tags
        for element in document.select(&self.link_selector) {
            if let Some(href) = element.value().attr("href") {
                links.insert(href.to_string());
            }
        }

        // Extract links from script tags
        for element in document.select(&self.script_selector) {
            if let Some(src) = element.value().attr("src") {
                links.insert(src.to_string());
            }
        }

        // Extract form actions
        for element in document.select(&self.form_selector) {
            if let Some(action) = element.value().attr("action") {
                links.insert(action.to_string());
            }
        }

        links.into_iter().collect()
    }

    pub fn extract_meta_info(&self, html: &str) -> Vec<(String, String)> {
        let mut meta_info = Vec::new();
        let document = Html::parse_document(html);
        let meta_selector = Selector::parse("meta[name], meta[property]").unwrap();

        for element in document.select(&meta_selector) {
            if let (Some(name), Some(content)) = (
                element.value().attr("name").or_else(|| element.value().attr("property")),
                element.value().attr("content")
            ) {
                meta_info.push((name.to_string(), content.to_string()));
            }
        }

        meta_info
    }
}

impl Default for HtmlParser {
    fn default() -> Self {
        Self::new()
    }
}
