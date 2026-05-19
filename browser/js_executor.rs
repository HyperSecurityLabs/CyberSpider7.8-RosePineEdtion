use anyhow::Result;
use regex::Regex;
use crate::browser::BrowserEngine;

pub struct JsExecutor {
    url_regex: Regex,
    api_regex: Regex,
    form_regex: Regex,
}

impl JsExecutor {
    pub fn new() -> Self {
        let url_regex = Regex::new(r#"[\"'](https?://[^\"']+)[\"']"#).unwrap();
        let api_regex = Regex::new(r#"/api/v?[0-9]?/[a-zA-Z0-9/_-]+"#).unwrap();
        let form_regex = Regex::new(r#"action\s*=\s*[\"']([^\"']+)[\"']"#).unwrap();

        Self {
            url_regex,
            api_regex,
            form_regex,
        }
    }

    pub async fn extract_dynamic_urls<B: BrowserEngine>(&self, browser: &B) -> Result<Vec<String>> {
        let mut urls = Vec::new();
        
        let scripts = vec![
            "Array.from(document.querySelectorAll('a')).map(a => a.href).join('\\n')",
            "Array.from(document.querySelectorAll('script[src]')).map(s => s.src).join('\\n')",
            "Array.from(document.querySelectorAll('link[href]')).map(l => l.href).join('\\n')",
            "Array.from(document.querySelectorAll('img[src]')).map(i => i.src).join('\\n')",
            "Object.keys(window).filter(k => typeof window[k] === 'function').join('\\n')",
        ];

        for script in scripts {
            if let Ok(result) = browser.execute_script(script).await {
                for line in result.lines() {
                    if let Some(url_match) = self.url_regex.find(line) {
                        urls.push(url_match.as_str().trim_matches('\'').trim_matches('"').to_string());
                    }
                }
            }
        }

        urls.sort();
        urls.dedup();
        Ok(urls)
    }

    pub async fn extract_api_endpoints<B: BrowserEngine>(&self, browser: &B) -> Result<Vec<String>> {
        let mut endpoints = Vec::new();
        
        let scripts = vec![
            "Object.getOwnPropertyNames(window).filter(n => n.includes('api') || n.includes('fetch')).join('\\n')",
            "Array.from(document.querySelectorAll('[data-api]')).map(e => e.dataset.api).join('\\n')",
        ];

        for script in scripts {
            if let Ok(result) = browser.execute_script(script).await {
                for endpoint in self.api_regex.find_iter(&result) {
                    endpoints.push(endpoint.as_str().to_string());
                }
            }
        }

        endpoints.sort();
        endpoints.dedup();
        Ok(endpoints)
    }

    pub async fn extract_form_actions<B: BrowserEngine>(&self, browser: &B) -> Result<Vec<String>> {
        let mut actions = Vec::new();
        
        let script = "Array.from(document.querySelectorAll('form')).map(f => f.action || f.getAttribute('action')).join('\\n')";
        
        if let Ok(result) = browser.execute_script(script).await {
            for action in self.form_regex.find_iter(&result) {
                actions.push(action.as_str().to_string());
            }
        }

        actions.sort();
        actions.dedup();
        Ok(actions)
    }

    pub async fn get_page_metadata<B: BrowserEngine>(&self, browser: &B) -> Result<PageMetadata> {
        let title = browser.get_title().await.unwrap_or_default();
        let source = browser.get_page_source().await?;
        
        let description_script = r#"
            (function() {
                let desc = document.querySelector('meta[name="description"]');
                return desc ? desc.content : '';
            })()
        "#;
        
        let description = browser.execute_script(description_script)
            .await
            .unwrap_or_default()
            .trim_matches('"')
            .to_string();

        let keywords_script = r#"
            (function() {
                let kw = document.querySelector('meta[name="keywords"]');
                return kw ? kw.content : '';
            })()
        "#;
        
        let keywords = browser.execute_script(keywords_script)
            .await
            .unwrap_or_default()
            .trim_matches('"')
            .to_string();

        Ok(PageMetadata {
            title,
            description,
            keywords,
            source_length: source.len(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct PageMetadata {
    pub title: String,
    pub description: String,
    pub keywords: String,
    pub source_length: usize,
}
