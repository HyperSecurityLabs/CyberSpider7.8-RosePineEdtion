use anyhow::Result;
use colored::*;
use futures::future::join_all;
use reqwest::Client;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use url::Url;

use crate::{SpiderConfig, DiscoveredUrl, progress::BrailleSpinner};
use super::url_extractor::UrlExtractor;
use crate::sources::ExternalSource;
use crate::media_corruption::{MediaCorruptionDetector, MediaCorruptionAttacker};

fn safe_mutex_lock<T>(mutex: &std::sync::Mutex<T>) -> Result<std::sync::MutexGuard<'_, T>>
where
    T: std::fmt::Debug,
{
    mutex.lock().map_err(|e| anyhow::anyhow!("Mutex poisoned: {:?}", e))
}

pub struct Crawler {
    client: Client,
    config: Arc<SpiderConfig>,
    url_extractor: UrlExtractor,
    wayback: crate::sources::wayback::WaybackMachine,
    commoncrawl: crate::sources::commoncrawl::CommonCrawl,
    virustotal: crate::sources::virustotal::VirusTotal,
    discovered_urls: Arc<std::sync::Mutex<Vec<DiscoveredUrl>>>,
    url_relationships: Arc<std::sync::Mutex<Vec<(String, String)>>>,
}

#[derive(Debug)]
pub struct CrawlResult {
    pub base_domain: String,
    pub discovered_urls: Vec<DiscoveredUrl>,
    pub subdomains: Vec<String>,
    pub s3_buckets: Vec<String>,
    pub total_requests: usize,
    pub successful_requests: usize,
    pub failed_requests: usize,
}

impl Crawler {
    pub fn new(config: &SpiderConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout))
            .user_agent("CyberSpider/7.8.0pro")
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to create HTTP client: {}", e))?;

        let config = Arc::new(config.clone());
        let url_extractor = UrlExtractor::new(config.clone());

        let wayback = crate::sources::wayback::WaybackMachine::new();
        let commoncrawl = crate::sources::commoncrawl::CommonCrawl::new();
        let virustotal = crate::sources::virustotal::VirusTotal::new(None);

        Ok(Self {
            client,
            config,
            url_extractor,
            wayback,
            commoncrawl,
            virustotal,
            discovered_urls: Arc::new(std::sync::Mutex::new(Vec::new())),
            url_relationships: Arc::new(std::sync::Mutex::new(Vec::new())),
        })
    }

    pub async fn crawl_targets(&mut self, targets: Vec<String>, spinner: &BrailleSpinner) -> Result<CrawlResult> {
        let mut all_urls = HashSet::new();
        let mut processed_urls = HashSet::new();
        let mut subdomains = HashSet::new();
        let mut s3_buckets = HashSet::new();
        let mut total_requests = 0;
        let mut successful_requests = 0;
        let mut failed_requests = 0;

        let url_tag = "[URL]".truecolor(156, 207, 216).bold();
        let status_tag = "[+]".truecolor(156, 207, 216).bold();
        let err_tag = "[-]".truecolor(235, 111, 146).bold();
        let wayback_tag = "[WAYBACK]".truecolor(196, 167, 231).bold();
        let commoncrawl_tag = "[COMMONCRAWL]".truecolor(49, 116, 143).bold();
        let virustotal_tag = "[VIRUSTOTAL]".truecolor(196, 167, 231).bold();

        for target in targets {
            let base_domain = self.extract_base_domain(&target)?;

            all_urls.insert(target.clone());

            for depth in 0..=self.config.depth {
                if depth == 0 {
                    if self.config.verbose || self.config.show_modules {
                        spinner.log(&format!("🎯 Targeting {}", target.truecolor(80, 250, 123)));
                    }
                    continue;
                }

                let urls_to_process: Vec<String> = all_urls.iter()
                    .filter(|url| !processed_urls.contains(*url))
                    .cloned()
                    .collect();

                if urls_to_process.is_empty() {
                    spinner.set_message(&format!("Depth {}: no new URLs", depth));
                    continue;
                }

                spinner.set_message(&format!("Depth {} — {} URLs to process", depth, urls_to_process.len()));

                if self.config.verbose || self.config.show_modules {
                    spinner.log(&format!("{} Depth {}: processing {} URLs ({} discovered, {} done)",
                        status_tag, depth, urls_to_process.len(), all_urls.len(), processed_urls.len()));
                }

                let semaphore = Arc::new(Semaphore::new(self.config.concurrent));
                let processed_count = Arc::new(std::sync::Mutex::new(0usize));
                let total_discovered_arc = Arc::new(std::sync::Mutex::new(all_urls.len()));

                let tasks: Vec<_> = urls_to_process.clone().into_iter().map(|url| {
                    let client = self.client.clone();
                    let config = self.config.clone();
                    let semaphore = semaphore.clone();
                    let url_extractor = self.url_extractor.clone();
                    let discovered_urls = self.discovered_urls.clone();
                    let url_relationships = self.url_relationships.clone();
                    let processed_count = processed_count.clone();

                    async move {
                        let _permit = semaphore.acquire().await
                            .map_err(|e| anyhow::anyhow!("Failed to acquire semaphore permit: {}", e))?;

                        if config.delay > 0 {
                            tokio::time::sleep(Duration::from_secs(config.delay)).await;
                        }

                        let result = Self::crawl_url(&client, &url, &url_extractor, &config, discovered_urls, url_relationships).await;

                        {
                            let mut processed = safe_mutex_lock(&processed_count)?;
                            *processed += 1;
                        }

                        result
                    }
                }).collect();

                let results = join_all(tasks).await;

                for (url, result) in urls_to_process.iter().zip(results) {
                    processed_urls.insert(url.clone());
                    total_requests += 1;

                    match result {
                        Ok((discovered, new_subdomains, new_s3_buckets)) => {
                            successful_requests += 1;

                            let initial_len = all_urls.len();
                            for u in &discovered {
                                if !all_urls.contains(u) {
                                    all_urls.insert(u.clone());
                                }
                            }
                            let new_urls_count = all_urls.len() - initial_len;

                            {
                                let mut total = safe_mutex_lock(&total_discovered_arc)?;
                                *total += new_urls_count;
                            }

                            subdomains.extend(new_subdomains);
                            s3_buckets.extend(new_s3_buckets);

                            if self.config.verbose || self.config.show_modules {
                                spinner.log(&format!("{} {}",
                                    url_tag,
                                    url.truecolor(80, 250, 123)));
                                if new_urls_count > 0 {
                                    spinner.log(&format!("  {} Found {} URLs",
                                        status_tag,
                                        new_urls_count.to_string().truecolor(156, 207, 216)));
                                }
                            }
                        }
                        Err(e) => {
                            failed_requests += 1;
                            if self.config.verbose {
                                spinner.log(&format!("{} {} — {}",
                                    err_tag,
                                    url.truecolor(80, 250, 123),
                                    e.to_string().truecolor(235, 111, 146)));
                            }
                        }
                    }
                }
            }

            if self.config.other_sources_enabled {
                let domain = url::Url::parse(&base_domain)
                    .map(|u| u.host_str().unwrap_or(&base_domain).to_string())
                    .unwrap_or_else(|_| base_domain.clone());

                if self.config.verbose || self.config.show_modules {
                    spinner.log(&format!("{} Fetching external sources for {}",
                        wayback_tag, domain.truecolor(80, 250, 123)));
                }

                if let Ok(wayback_urls) = self.wayback.fetch_urls(&domain).await {
                    let initial_len = all_urls.len();
                    for url in &wayback_urls {
                        if !all_urls.contains(url) {
                            all_urls.insert(url.clone());
                        }
                    }
                    let new_urls_count = all_urls.len() - initial_len;
                    if self.config.verbose || self.config.show_modules {
                        spinner.log(&format!("{} {} new unique URLs from Wayback Machine",
                            wayback_tag,
                            new_urls_count.to_string().truecolor(156, 207, 216)));
                    }
                } else if self.config.verbose {
                    spinner.log(&format!("{} Failed to fetch from Wayback Machine",
                        wayback_tag));
                }

                if let Ok(commoncrawl_urls) = self.commoncrawl.fetch_urls(&domain).await {
                    let initial_len = all_urls.len();
                    for url in &commoncrawl_urls {
                        if !all_urls.contains(url) {
                            all_urls.insert(url.clone());
                        }
                    }
                    let new_urls_count = all_urls.len() - initial_len;
                    if self.config.verbose || self.config.show_modules {
                        spinner.log(&format!("{} {} new unique URLs from Common Crawl",
                            commoncrawl_tag,
                            new_urls_count.to_string().truecolor(156, 207, 216)));
                    }
                } else if self.config.verbose {
                    spinner.log(&format!("{} Failed to fetch from Common Crawl",
                        commoncrawl_tag));
                }

                if let Ok(virustotal_urls) = self.virustotal.fetch_urls(&domain).await {
                    let initial_len = all_urls.len();
                    for url in &virustotal_urls {
                        if !all_urls.contains(url) {
                            all_urls.insert(url.clone());
                        }
                    }
                    let new_urls_count = all_urls.len() - initial_len;
                    if self.config.verbose || self.config.show_modules {
                        spinner.log(&format!("{} {} new unique URLs from VirusTotal",
                            virustotal_tag,
                            new_urls_count.to_string().truecolor(156, 207, 216)));
                    }
                } else if self.config.verbose {
                    spinner.log(&format!("{} Failed to fetch from VirusTotal (no API key?)",
                        virustotal_tag));
                }

                spinner.set_message(&format!("{} unique URLs after external sources", all_urls.len()));
            }

            let discovered_urls_final: Vec<DiscoveredUrl> = {
                let discovered = safe_mutex_lock(&self.discovered_urls)?;
                discovered.clone()
            };

            spinner.log(&format!("{} Crawl complete — {} URLs, {} subdomains, {} S3 buckets",
                status_tag,
                all_urls.len().to_string().truecolor(156, 207, 216),
                subdomains.len().to_string().truecolor(246, 193, 119),
                s3_buckets.len().to_string().truecolor(235, 111, 146)));

            return Ok(CrawlResult {
                base_domain,
                discovered_urls: discovered_urls_final,
                subdomains: subdomains.into_iter().collect(),
                s3_buckets: s3_buckets.into_iter().collect(),
                total_requests,
                successful_requests,
                failed_requests,
            });
        }

        Err(anyhow::anyhow!("No targets to crawl"))
    }

    async fn crawl_url(
        client: &Client,
        url: &str,
        url_extractor: &UrlExtractor,
        config: &SpiderConfig,
        discovered_urls: Arc<std::sync::Mutex<Vec<DiscoveredUrl>>>,
        url_relationships: Arc<std::sync::Mutex<Vec<(String, String)>>>,
    ) -> Result<(Vec<String>, Vec<String>, Vec<String>)> {
        if config.verbose {
            println!("Crawling: {}", url);
        }

        let response = match client.get(url).send().await {
            Ok(resp) => {
                if config.verbose {
                    println!("Response: {} - {} ({})",
                        url,
                        resp.status(),
                        resp.status().canonical_reason().unwrap_or("Unknown"));
                }
                resp
            }
            Err(e) => {
                if config.verbose {
                    println!("Failed to fetch {}: {}", url, e);
                }
                return Err(e.into());
            }
        };

        let status_code = response.status().as_u16();
        let content_type = response.headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|ct| ct.to_str().ok())
            .map(|ct| ct.to_string());

        let content = match response.text().await {
            Ok(text) => {
                if config.verbose {
                    println!("Retrieved {} bytes from {}", text.len(), url);
                }
                text
            }
            Err(e) => {
                if config.verbose {
                    println!("Failed to read response body from {}: {}", url, e);
                }
                return Err(e.into());
            }
        };

        let title = if content_type.as_ref().map_or(false, |ct| ct.contains("text/html")) {
            extract_title_from_html(&content)
        } else {
            None
        };

        let discovered_urls_list = url_extractor.extract_urls(&content, url)?;
        let subdomains = url_extractor.extract_subdomains(&content)?;
        let s3_buckets = url_extractor.extract_s3_buckets(&content)?;

        {
            let mut discovered = safe_mutex_lock(&discovered_urls)?;
            discovered.push(DiscoveredUrl {
                url: url.to_string(),
                source: "crawl".to_string(),
                status_code: Some(status_code),
                content_type: content_type.clone(),
                title,
                method: "GET".to_string(),
            });
        }

        {
            let mut relationships = safe_mutex_lock(&url_relationships)?;
            for discovered_url in &discovered_urls_list {
                relationships.push((url.to_string(), discovered_url.clone()));
            }
        }

        let url_tag = "[URL]".truecolor(156, 207, 216).bold();
        let sub_tag = "[SUBDOMAIN]".truecolor(246, 193, 119).bold();
        let s3_tag = "[S3]".truecolor(196, 111, 146).bold();
        let media_tag = "[MEDIA]".truecolor(196, 111, 146).bold();
        let extract_tag = "[EXTRACT]".truecolor(196, 111, 146).bold();

        if config.verbose || config.show_modules {

            println!("{} {} URLs, {} {}, {} {} from {}",
                extract_tag,
                discovered_urls_list.len().to_string().truecolor(156, 207, 216),
                subdomains.len().to_string().truecolor(246, 193, 119),
                sub_tag,
                s3_buckets.len().to_string().truecolor(196, 111, 146),
                s3_tag,
                url.truecolor(80, 250, 123));

            if !discovered_urls_list.is_empty() {
                println!("   {} {} URLs:", url_tag, discovered_urls_list.len());
                for (i, discovered_url) in discovered_urls_list.iter().take(5).enumerate() {
                    println!("     {}. {}", i + 1, discovered_url.truecolor(80, 250, 123));
                }
                if discovered_urls_list.len() > 5 {
                    println!("     ... and {} more", discovered_urls_list.len() - 5);
                }
            }
        }

        if is_media_extension(url) && config.media_check {
            let detector = MediaCorruptionDetector::new();
            if let Ok(findings) = detector.check_url(url).await {
                for finding in &findings {
                    println!("{} {} - {}",
                        media_tag,
                        finding.url.truecolor(80, 250, 123),
                        finding.description.truecolor(235, 111, 146));
                    if let Some(rec) = &finding.recommendation {
                        println!("  {}", rec.truecolor(156, 207, 216));
                    }
                }
            }

            // Run active media corruption attack
            if config.deep_scan || config.media_check {
                let attacker = MediaCorruptionAttacker::new();
                let result = attacker.corrupt_url(url).await;
                let attack_tag = "[ATTACK]".truecolor(235, 111, 146).bold();
                if result.success {
                    println!("{} {} via {} — {}",
                        attack_tag,
                        result.url.truecolor(80, 250, 123),
                        result.method.truecolor(246, 193, 119),
                        result.detail.truecolor(156, 207, 216));
                } else if config.verbose {
                    println!("{} {} — {}",
                        attack_tag,
                        result.url.truecolor(80, 250, 123),
                        result.detail.truecolor(235, 111, 146));
                }
            }
        }

        Ok((discovered_urls_list, subdomains, s3_buckets))
    }

    fn extract_base_domain(&self, url: &str) -> Result<String> {
        let parsed_url = Url::parse(url)?;
        Ok(parsed_url.host_str().unwrap_or("unknown").to_string())
    }
}

fn is_media_extension(url: &str) -> bool {
    let lower = url.to_lowercase();
    let without_query = lower.split(&['?', '#'][..]).next().unwrap_or(&lower);
    if let Some(dot_pos) = without_query.rfind('.') {
        let ext = &without_query[dot_pos + 1..];
        matches!(ext, "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" | "svg"
            | "ico" | "mp4" | "webm" | "mp3" | "wav" | "ogg"
            | "pdf" | "zip" | "tiff" | "tif" | "mov" | "avi")
    } else {
        false
    }
}

fn extract_title_from_html(html: &str) -> Option<String> {
    let title_regex = regex::Regex::new(r"<title[^>]*>(.*?)</title>").ok()?;
    if let Some(captures) = title_regex.captures(html) {
        if let Some(title_match) = captures.get(1) {
            let title = title_match.as_str().trim();
            if !title.is_empty() {
                let decoded = title
                    .replace("&lt;", "<")
                    .replace("&gt;", ">")
                    .replace("&amp;", "&")
                    .replace("&quot;", "\"")
                    .replace("&#39;", "'");
                return Some(decoded);
            }
        }
    }
    None
}
