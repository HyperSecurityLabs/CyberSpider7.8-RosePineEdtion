use anyhow::Result;
use crate::{SpiderResult, output::OutputFormatter};

pub struct TextFormatter;

impl OutputFormatter for TextFormatter {
    fn format(&self, result: &SpiderResult) -> Result<String> {
        let mut output = String::new();
        
        output.push_str(&format!("=== Spider Results for {} ===\n\n", result.base_domain));
        
        // Statistics
        output.push_str("Statistics:\n");
        output.push_str(&format!("  Total Requests: {}\n", result.total_requests));
        output.push_str(&format!("  Successful: {}\n", result.successful_requests));
        output.push_str(&format!("  Failed: {}\n", result.failed_requests));
        output.push_str(&format!("  Duration: {}ms\n\n", result.duration_ms));
        
        // Discovered URLs
        output.push_str(&format!("Discovered URLs ({}):\n", result.discovered_urls.len()));
        for url in &result.discovered_urls {
            output.push_str(&format!("  {} [{}]\n", url.url, url.source));
        }
        output.push_str("\n");
        
        // Subdomains
        if !result.subdomains.is_empty() {
            output.push_str(&format!("Subdomains ({}):\n", result.subdomains.len()));
            for subdomain in &result.subdomains {
                output.push_str(&format!("  {}\n", subdomain));
            }
            output.push_str("\n");
        }
        
        // S3 Buckets
        if !result.s3_buckets.is_empty() {
            output.push_str(&format!("S3 Buckets ({}):\n", result.s3_buckets.len()));
            for bucket in &result.s3_buckets {
                output.push_str(&format!("  {}\n", bucket));
            }
            output.push_str("\n");
        }
        
        Ok(output)
    }
}
