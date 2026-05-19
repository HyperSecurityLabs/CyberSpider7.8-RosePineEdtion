use anyhow::Result;
use regex::Regex;
use reqwest::Client;

use crate::detectors::Detector;

pub struct S3BucketDetector {
    client: Client,
    s3_regex: Regex,
}

impl S3BucketDetector {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("CyberSpider/7.8.0pro")
            .build()
            .expect("Failed to create HTTP client");

        let s3_regex = Regex::new(
            r"[a-zA-Z0-9.-]+\.s3[.-]?(amazonaws\.com|[a-zA-Z0-9-]+\.amazonaws\.com)"
        ).unwrap();

        Self {
            client,
            s3_regex,
        }
    }

    async fn verify_bucket(&self, bucket_url: &str) -> bool {
        // Simple verification by checking if the bucket responds
        if let Ok(response) = self.client.head(bucket_url).send().await {
            response.status().is_success() || response.status() == 403
        } else {
            false
        }
    }
}

#[async_trait::async_trait]
impl Detector for S3BucketDetector {
    async fn detect(&self, content: &str, _base_url: &str) -> Result<Vec<String>> {
        let mut buckets = Vec::new();

        for captures in self.s3_regex.captures_iter(content) {
            if let Some(bucket_match) = captures.get(0) {
                let bucket_url = bucket_match.as_str();
                
                // Verify bucket exists
                if self.verify_bucket(bucket_url).await {
                    buckets.push(bucket_url.to_string());
                }
            }
        }

        Ok(buckets)
    }

    fn detector_name(&self) -> &'static str {
        "s3_buckets"
    }
}
