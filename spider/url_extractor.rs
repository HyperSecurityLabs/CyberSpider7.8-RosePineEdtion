use anyhow::Result;
use regex::Regex;
use std::collections::HashSet;
use std::sync::Arc;
use url::Url;

use crate::SpiderConfig;

#[derive(Clone)]
pub struct UrlExtractor {
    config: Arc<SpiderConfig>,
    url_regex: Regex,
    subdomain_regex: Regex,
    s3_regex: Regex,
    js_url_regex: Regex,
}

impl UrlExtractor {
    pub fn new(config: Arc<SpiderConfig>) -> Self {
        Self {
            config,
            // Enhanced URL regex for better extraction
            url_regex: Regex::new(r"https?://[^\s<>]+").unwrap(),
            // Subdomain regex with better validation
            subdomain_regex: Regex::new(r"https?://([a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?\.)+[a-zA-Z]{2,}").unwrap(),
            // S3 bucket regex with AWS endpoint patterns
            s3_regex: Regex::new(r"[a-zA-Z0-9.-]+\.s3[.-]?(amazonaws\.com|[a-zA-Z0-9-]+\.amazonaws\.com|s3-[a-z0-9-]+\.amazonaws\.com)").unwrap(),
            // JavaScript file regex with more extensions
            js_url_regex: Regex::new(r#"["']https?://[^"']+\.(js|css|png|jpg|jpeg|gif|ico|svg|woff|woff2|ttf|eot)["']"#).unwrap(),
        }
    }

    pub fn extract_urls(&self, content: &str, base_url: &str) -> Result<Vec<String>> {
        let mut urls = HashSet::new();
        
        // Extract URLs using regex
        for captures in self.url_regex.captures_iter(content) {
            if let Some(url_match) = captures.get(0) {
                let url_str = url_match.as_str();
                if let Ok(normalized_url) = self.normalize_url(url_str, base_url) {
                    urls.insert(normalized_url);
                }
            }
        }

        // Extract JavaScript URLs if enabled
        if self.config.js_enabled {
            for captures in self.js_url_regex.captures_iter(content) {
                if let Some(url_match) = captures.get(0) {
                    let url_str = url_match.as_str().trim_matches('"').trim_matches('\'');
                    if let Ok(normalized_url) = self.normalize_url(url_str, base_url) {
                        urls.insert(normalized_url);
                    }
                }
            }
        }

        Ok(urls.into_iter().collect())
    }

    pub fn extract_subdomains(&self, content: &str) -> Result<Vec<String>> {
        let mut subdomains = HashSet::new();
        
        for captures in self.subdomain_regex.captures_iter(content) {
            if let Some(domain_match) = captures.get(0) {
                let domain = domain_match.as_str();
                if let Ok(parsed) = Url::parse(domain) {
                    if let Some(host) = parsed.host_str() {
                        let parts: Vec<&str> = host.split('.').collect();
                        if parts.len() > 2 {
                            subdomains.insert(host.to_string());
                        }
                    }
                }
            }
        }

        Ok(subdomains.into_iter().collect())
    }

    pub fn extract_s3_buckets(&self, content: &str) -> Result<Vec<String>> {
        let mut buckets = HashSet::new();
        
        for captures in self.s3_regex.captures_iter(content) {
            if let Some(bucket_match) = captures.get(0) {
                buckets.insert(bucket_match.as_str().to_string());
            }
        }

        Ok(buckets.into_iter().collect())
    }

    fn normalize_url(&self, url: &str, base_url: &str) -> Result<String> {
        // If it's already a full URL, return as is
        if url.starts_with("http://") || url.starts_with("https://") {
            return Ok(url.to_string());
        }

        // Handle relative URLs
        if let Ok(base_parsed) = Url::parse(base_url) {
            let joined = base_parsed.join(url)?;
            Ok(joined.to_string())
        } else {
            Ok(url.to_string())
        }
    }
}
