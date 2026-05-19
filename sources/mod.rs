pub mod wayback;
pub mod commoncrawl;
pub mod virustotal;

use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait ExternalSource {
    async fn fetch_urls(&self, domain: &str) -> Result<Vec<String>>;
    fn source_name(&self) -> &'static str;
}
