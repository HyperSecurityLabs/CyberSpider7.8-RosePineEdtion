use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::sources::ExternalSource;

#[derive(Debug, Serialize, Deserialize)]
struct CommonCrawlIndex {
    id: String,
    name: String,
    cdx_api: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CommonCrawlRecord {
    url: String,
    timestamp: String,
    status: String,
    mime: String,
}

pub struct CommonCrawl {
    client: Client,
}

impl CommonCrawl {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("CyberSpider/7.8.0pro")
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    async fn fetch_index_urls(&self, domain: &str) -> Result<Vec<String>> {
        // Get available Common Crawl indexes
        let indexes_url = "http://index.commoncrawl.org/collinfo.json";
        let response = self.client.get(indexes_url).send().await?;
        let indexes: Vec<CommonCrawlIndex> = response.json().await?;
        
        let mut all_urls = HashSet::new();
        
        // Search the most recent index
        if let Some(latest_index) = indexes.first() {
            let search_url = format!(
                "{}?url=*.{}/*&output=json&limit=1000",
                latest_index.cdx_api, domain
            );
            
            let search_response = self.client.get(&search_url).send().await?;
            let text = search_response.text().await?;
            
            // Parse CDX format lines
            for line in text.lines() {
                if line.is_empty() || line.starts_with('[') {
                    continue;
                }
                
                if let Ok(record) = serde_json::from_str::<CommonCrawlRecord>(line) {
                    all_urls.insert(record.url);
                }
            }
        }
        
        Ok(all_urls.into_iter().collect())
    }
}

#[async_trait::async_trait]
impl ExternalSource for CommonCrawl {
    async fn fetch_urls(&self, domain: &str) -> Result<Vec<String>> {
        self.fetch_index_urls(domain).await
    }

    fn source_name(&self) -> &'static str {
        "common_crawl"
    }
}
