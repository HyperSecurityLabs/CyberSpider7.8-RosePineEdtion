use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::sources::ExternalSource;

#[derive(Debug, Serialize, Deserialize)]
struct VirusTotalResponse {
    data: Option<VirusTotalData>,
    response_code: i32,
    verbose_msg: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct VirusTotalData {
    id: String,
    detected_urls: Option<Vec<DetectedUrl>>,
    subdomains: Option<Vec<String>>,
    resolutions: Option<Vec<Resolution>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DetectedUrl {
    url: String,
    positives: i32,
    total: i32,
    scan_date: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Resolution {
    ip_address: String,
    last_resolved: String,
}

pub struct VirusTotal {
    client: Client,
    api_key: Option<String>,
}

impl VirusTotal {
    pub fn new(api_key: Option<String>) -> Self {
        let client = Client::builder()
            .user_agent("CyberSpider/7.8.0pro")
            .build()
            .expect("Failed to create HTTP client");

        Self { client, api_key }
    }

    async fn fetch_domain_urls(&self, domain: &str) -> Result<Vec<String>> {
        if let Some(api_key) = &self.api_key {
            let mut all_urls = std::collections::HashSet::new();
            
            // Fetch domain report
            let domain_url = format!(
                "https://www.virustotal.com/vtapi/v2/domain/report?apikey={}&domain={}",
                api_key, domain
            );

            let response = self.client.get(&domain_url).send().await?;
            let text = response.text().await?;
            
            if let Ok(vt_response) = serde_json::from_str::<VirusTotalResponse>(&text) {
                if vt_response.response_code == 1 {
                    if let Some(data) = vt_response.data {
                        // Add detected URLs
                        if let Some(detected_urls) = data.detected_urls {
                            for url_info in detected_urls {
                                all_urls.insert(url_info.url);
                            }
                        }
                        
                        // Add subdomains (convert to full URLs)
                        if let Some(subdomains) = data.subdomains {
                            for subdomain in subdomains {
                                all_urls.insert(format!("https://{}", subdomain));
                                all_urls.insert(format!("http://{}", subdomain));
                            }
                        }
                    }
                }
            }
            
            // Also try to fetch URLs using the search API
            let search_url = format!(
                "https://www.virustotal.com/vtapi/v2/domain/search?apikey={}&query={}",
                api_key, domain
            );
            
            if let Ok(search_response) = self.client.get(&search_url).send().await {
                if let Ok(search_text) = search_response.text().await {
                    if let Ok(search_result) = serde_json::from_str::<VirusTotalResponse>(&search_text) {
                        if search_result.response_code == 1 {
                            if let Some(search_data) = search_result.data {
                                if let Some(detected_urls) = search_data.detected_urls {
                                    for url_info in detected_urls {
                                        all_urls.insert(url_info.url);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            Ok(all_urls.into_iter().collect())
        } else {
            // No API key provided, return empty results
            Ok(Vec::new())
        }
    }
}

#[async_trait::async_trait]
impl ExternalSource for VirusTotal {
    async fn fetch_urls(&self, domain: &str) -> Result<Vec<String>> {
        self.fetch_domain_urls(domain).await
    }

    fn source_name(&self) -> &'static str {
        "virustotal"
    }
}
