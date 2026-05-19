pub mod crawler;
pub mod parser;
pub mod url_extractor;

use anyhow::Result;
use std::time::Instant;
use colored::*;
use crate::{SpiderConfig, SpiderResult, progress::BrailleSpinner};

pub struct Spider {
    config: SpiderConfig,
}

impl Spider {
    pub fn new(config: SpiderConfig) -> Self {
        Self { config }
    }

    pub async fn run(&mut self, spinner: &BrailleSpinner) -> Result<SpiderResult> {
        let start_time = Instant::now();

        let targets = self.get_targets()?;

        if targets.is_empty() {
            return Err(anyhow::anyhow!("No valid targets specified"));
        }

        spinner.set_message("Initializing spider...");

        let target_list = targets.join(", ");
        if self.config.verbose || self.config.show_modules {
            spinner.log(&format!("{} {}",
                "[TARGET]".truecolor(246, 193, 119).bold(),
                target_list.truecolor(80, 250, 123)));
        }

        let mut crawler = crawler::Crawler::new(&self.config)
            .map_err(|e| anyhow::anyhow!("Failed to initialize crawler: {}", e))?;

        spinner.set_message("Running reconnaissance...");

        let result = crawler.crawl_targets(targets, spinner).await?;

        let duration = start_time.elapsed();

        spinner.log(&format!("{} {} URLs, {} subdomains, {} S3 buckets in {}ms",
            "[DONE]".truecolor(156, 207, 216).bold(),
            result.discovered_urls.len().to_string().truecolor(196, 111, 146),
            result.subdomains.len().to_string().truecolor(246, 193, 119),
            result.s3_buckets.len().to_string().truecolor(235, 111, 146),
            duration.as_millis().to_string().truecolor(156, 207, 216)));

        Ok(SpiderResult {
            base_domain: result.base_domain,
            discovered_urls: result.discovered_urls,
            subdomains: result.subdomains,
            s3_buckets: result.s3_buckets,
            total_requests: result.total_requests,
            successful_requests: result.successful_requests,
            failed_requests: result.failed_requests,
            duration_ms: duration.as_millis() as u64,
        })
    }

    fn get_targets(&self) -> Result<Vec<String>> {
        let mut targets = Vec::new();
        
        if let Some(site) = &self.config.site {
            targets.push(site.clone());
        }
        
        if let Some(sites_file) = &self.config.sites_file {
            let content = std::fs::read_to_string(sites_file)?;
            for line in content.lines() {
                let line = line.trim();
                if !line.is_empty() && !line.starts_with('#') {
                    targets.push(line.to_string());
                }
            }
        }
        
        Ok(targets)
    }
}
