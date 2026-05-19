use anyhow::Result;
use regex::Regex;

use crate::detectors::Detector;

pub struct SubdomainDetector {
    subdomain_regex: Regex,
}

impl SubdomainDetector {
    pub fn new() -> Self {
        let subdomain_regex = Regex::new(
            r"https?://([a-zA-Z0-9-]+\.){2,}[a-zA-Z]{2,}"
        ).unwrap();

        Self {
            subdomain_regex,
        }
    }

    fn extract_subdomain(&self, url: &str) -> Option<String> {
        if let Ok(parsed) = url::Url::parse(url) {
            if let Some(host) = parsed.host_str() {
                let parts: Vec<&str> = host.split('.').collect();
                if parts.len() > 2 {
                    return Some(host.to_string());
                }
            }
        }
        None
    }
}

#[async_trait::async_trait]
impl Detector for SubdomainDetector {
    async fn detect(&self, content: &str, _base_url: &str) -> Result<Vec<String>> {
        let mut subdomains = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for captures in self.subdomain_regex.captures_iter(content) {
            if let Some(url_match) = captures.get(0) {
                let url = url_match.as_str();
                
                if let Some(subdomain) = self.extract_subdomain(url) {
                    if seen.insert(subdomain.clone()) {
                        subdomains.push(subdomain);
                    }
                }
            }
        }

        Ok(subdomains)
    }

    fn detector_name(&self) -> &'static str {
        "subdomains"
    }
}
