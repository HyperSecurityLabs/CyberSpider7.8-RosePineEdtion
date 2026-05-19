use anyhow::Result;
use reqwest::Client;
use std::collections::HashSet;

use crate::sources::ExternalSource;

pub struct WaybackMachine {
    client: Client,
}

impl WaybackMachine {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("CyberSpider/7.8.0pro")
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    async fn fetch_raw_urls(&self, domain: &str) -> Result<Vec<String>> {
        let url = format!(
            "http://web.archive.org/cdx/search/cdx?url={}/*&output=json&collapse=urlkey",
            domain
        );

        let response = self.client.get(&url).send().await?;
        let text = response.text().await?;

        let lines: Vec<&str> = text.lines().collect();
        if lines.is_empty() {
            return Ok(Vec::new());
        }

        // Skip header line
        let mut urls = HashSet::new();
        for line in lines.iter().skip(1) {
            if let Some(url) = line.split(',').nth(2) {
                urls.insert(url.to_string());
            }
        }

        Ok(urls.into_iter().collect())
    }
}

#[async_trait::async_trait]
impl ExternalSource for WaybackMachine {
    async fn fetch_urls(&self, domain: &str) -> Result<Vec<String>> {
        self.fetch_raw_urls(domain).await
    }

    fn source_name(&self) -> &'static str {
        "wayback_machine"
    }
}
