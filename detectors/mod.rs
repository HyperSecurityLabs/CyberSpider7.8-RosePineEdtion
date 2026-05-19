pub mod s3_buckets;
pub mod subdomains;

use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Detector {
    async fn detect(&self, content: &str, base_url: &str) -> Result<Vec<String>>;
    fn detector_name(&self) -> &'static str;
}
