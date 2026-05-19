pub mod rotator;

use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proxy {
    pub url: String,
    pub proxy_type: ProxyType,
    pub username: Option<String>,
    pub password: Option<String>,
    pub country: Option<String>,
    pub response_time: Option<u64>,
    pub success_rate: f64,
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProxyType {
    Http,
    Https,
    Socks4,
    Socks5,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub enabled: bool,
    pub rotation_strategy: RotationStrategy,
    pub health_check_interval: u64,
    pub max_failures: u32,
    pub timeout_seconds: u64,
    pub exclude_domains: Vec<String>,
    pub include_domains: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RotationStrategy {
    RoundRobin,
    Random,
    LeastUsed,
    Fastest,
    Weighted,
}

#[async_trait]
pub trait ProxyManager {
    async fn add_proxy(&mut self, proxy: Proxy) -> Result<()>;
    async fn remove_proxy(&mut self, url: &str) -> Result<bool>;
    async fn get_proxy(&mut self, target_url: &str) -> Result<Option<Proxy>>;
    async fn mark_proxy_success(&mut self, proxy_url: &str, response_time: u64) -> Result<()>;
    async fn mark_proxy_failure(&mut self, proxy_url: &str) -> Result<()>;
    async fn health_check_all(&mut self) -> Result<Vec<ProxyHealth>>;
    async fn get_stats(&self) -> ProxyStats;
    async fn reset_stats(&mut self) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyHealth {
    pub proxy_url: String,
    pub is_healthy: bool,
    pub response_time: u64,
    pub error: Option<String>,
    pub checked_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyStats {
    pub total_proxies: usize,
    pub active_proxies: usize,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time: f64,
    pub success_rate: f64,
    pub last_rotation: Option<chrono::DateTime<chrono::Utc>>,
}

pub struct ProxyPool {
    proxies: HashMap<String, Proxy>,
    config: ProxyConfig,
    rotation_state: RotationState,
    _client: Client,
}

impl ProxyPool {
    pub fn new(config: ProxyConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .unwrap();

        Self {
            proxies: HashMap::new(),
            config,
            rotation_state: RotationState::new(),
            _client: client,
        }
    }

    fn should_use_proxy(&self, target_url: &str) -> bool {
        if !self.config.enabled {
            return false;
        }

        if !self.config.include_domains.is_empty() {
            return self.config.include_domains.iter().any(|domain| target_url.contains(domain));
        }

        if !self.config.exclude_domains.is_empty() {
            return !self.config.exclude_domains.iter().any(|domain| target_url.contains(domain));
        }

        true
    }

    fn get_active_proxies(&self) -> Vec<&Proxy> {
        self.proxies
            .values()
            .filter(|p| p.is_active)
            .collect()
    }

    fn select_proxy(&mut self, _target_url: &str) -> Option<Proxy> {
        let active_proxies: Vec<Proxy> = self.proxies
            .values()
            .filter(|p| p.is_active)
            .cloned()
            .collect();
        
        if active_proxies.is_empty() {
            return None;
        }

        // Get proxy references for selection algorithms
        let proxy_refs: Vec<&Proxy> = active_proxies.iter().collect();

        let selected = match self.config.rotation_strategy {
            RotationStrategy::RoundRobin => {
                let index = self.rotation_state.round_robin_index % proxy_refs.len();
                self.rotation_state.round_robin_index += 1;
                proxy_refs.get(index).map(|p| (*p).clone())
            }
            RotationStrategy::Random => {
                if proxy_refs.is_empty() {
                    None
                } else {
                    let index = rand::random::<usize>() % proxy_refs.len();
                    proxy_refs.get(index).map(|p| (*p).clone())
                }
            }
            RotationStrategy::LeastUsed => {
                proxy_refs
                    .iter()
                    .min_by_key(|p| self.rotation_state.usage_count.get(&p.url).unwrap_or(&0))
                    .map(|p| (*p).clone())
            }
            RotationStrategy::Fastest => {
                proxy_refs
                    .iter()
                    .filter(|p| p.response_time.is_some())
                    .min_by_key(|p| p.response_time.unwrap())
                    .or_else(|| proxy_refs.first())
                    .map(|p| (*p).clone())
            }
            RotationStrategy::Weighted => {
                let total_weight: f64 = proxy_refs
                    .iter()
                    .map(|p| self.calculate_weight(p))
                    .sum();

                if total_weight == 0.0 {
                    // Fallback to random
                    if proxy_refs.is_empty() {
                        None
                    } else {
                        let index = rand::random::<usize>() % proxy_refs.len();
                        proxy_refs.get(index).map(|p| (*p).clone())
                    }
                } else {
                    let mut random = rand::random::<f64>() * total_weight;
                    
                    for proxy in &proxy_refs {
                        random -= self.calculate_weight(proxy);
                        if random <= 0.0 {
                            return Some((*proxy).clone());
                        }
                    }
                    
                    proxy_refs.last().map(|p| (*p).clone())
                }
            }
        };

        if let Some(proxy) = selected {
            self.rotation_state.mark_used(&proxy.url);
            Some(proxy.clone())
        } else {
            None
        }
    }

    fn calculate_weight(&self, proxy: &Proxy) -> f64 {
        let mut weight = 1.0;
        
        // Weight by success rate
        weight *= proxy.success_rate;
        
        // Weight by response time (faster is better)
        if let Some(response_time) = proxy.response_time {
            weight *= (1000.0 / response_time as f64).min(10.0);
        }
        
        // Weight by recent usage (less recently used is better)
        if let Some(last_used) = proxy.last_used {
            let hours_since_last_use = (chrono::Utc::now() - last_used).num_hours() as f64;
            weight *= (hours_since_last_use + 1.0).min(10.0);
        }
        
        weight
    }
}

#[async_trait::async_trait]
impl ProxyManager for ProxyPool {
    async fn add_proxy(&mut self, proxy: Proxy) -> Result<()> {
        self.proxies.insert(proxy.url.clone(), proxy);
        Ok(())
    }

    async fn remove_proxy(&mut self, url: &str) -> Result<bool> {
        Ok(self.proxies.remove(url).is_some())
    }

    async fn get_proxy(&mut self, target_url: &str) -> Result<Option<Proxy>> {
        if !self.should_use_proxy(target_url) {
            return Ok(None);
        }

        Ok(self.select_proxy(target_url))
    }

    async fn mark_proxy_success(&mut self, proxy_url: &str, response_time: u64) -> Result<()> {
        if let Some(proxy) = self.proxies.get_mut(proxy_url) {
            proxy.response_time = Some(response_time);
            proxy.success_rate = (proxy.success_rate * 0.9) + (1.0 * 0.1); // Moving average
            proxy.last_used = Some(chrono::Utc::now());
            
            // Reset failure count on success
            self.rotation_state.failure_counts.remove(proxy_url);
        }
        Ok(())
    }

    async fn mark_proxy_failure(&mut self, proxy_url: &str) -> Result<()> {
        let failure_count = self.rotation_state.failure_counts.entry(proxy_url.to_string()).or_insert(0);
        *failure_count += 1;
        
        // Deactivate proxy if too many failures
        if *failure_count >= self.config.max_failures {
            if let Some(proxy) = self.proxies.get_mut(proxy_url) {
                proxy.is_active = false;
            }
        }
        
        // Update success rate
        if let Some(proxy) = self.proxies.get_mut(proxy_url) {
            proxy.success_rate = (proxy.success_rate * 0.9) + (0.0 * 0.1);
        }
        
        Ok(())
    }

    async fn health_check_all(&mut self) -> Result<Vec<ProxyHealth>> {
        let mut health_results = Vec::new();
        
        // Collect proxy URLs to avoid borrowing issues
        let proxy_urls: Vec<String> = self.proxies.keys().cloned().collect();
        
        for proxy_url in proxy_urls {
            let health = self.check_proxy_health(&proxy_url).await;
            
            // Update proxy status based on health check
            if let Some(proxy) = self.proxies.get_mut(&proxy_url) {
                match &health {
                    ProxyHealth { is_healthy: true, response_time, .. } => {
                        proxy.is_active = true;
                        proxy.response_time = Some(*response_time);
                    }
                    ProxyHealth { is_healthy: false, .. } => {
                        proxy.is_active = false;
                    }
                }
            }
            
            health_results.push(health);
        }
        
        Ok(health_results)
    }

    async fn get_stats(&self) -> ProxyStats {
        let active_proxies = self.get_active_proxies().len();
        let total_requests = self.rotation_state.total_requests;
        let successful_requests = self.rotation_state.successful_requests;
        let failed_requests = self.rotation_state.failed_requests;
        
        let success_rate = if total_requests > 0 {
            successful_requests as f64 / total_requests as f64 * 100.0
        } else {
            0.0
        };

        let average_response_time = if successful_requests > 0 {
            self.rotation_state.total_response_time as f64 / successful_requests as f64
        } else {
            0.0
        };

        ProxyStats {
            total_proxies: self.proxies.len(),
            active_proxies,
            total_requests,
            successful_requests,
            failed_requests,
            average_response_time,
            success_rate,
            last_rotation: self.rotation_state.last_rotation,
        }
    }

    async fn reset_stats(&mut self) -> Result<()> {
        self.rotation_state = RotationState::new();
        
        // Reset proxy stats
        for proxy in self.proxies.values_mut() {
            proxy.success_rate = 1.0;
            proxy.response_time = None;
            proxy.last_used = None;
            proxy.is_active = true;
        }
        
        Ok(())
    }
}

impl ProxyPool {
    async fn check_proxy_health(&self, proxy_url: &str) -> ProxyHealth {
        let start = std::time::Instant::now();
        
        let proxy = reqwest::Proxy::all(proxy_url).unwrap();
        
        let client = match Client::builder()
            .proxy(proxy)
            .timeout(Duration::from_secs(10))
            .build() {
                Ok(c) => c,
                Err(e) => {
                    return ProxyHealth {
                        proxy_url: proxy_url.to_string(),
                        is_healthy: false,
                        response_time: 0,
                        error: Some(e.to_string()),
                        checked_at: chrono::Utc::now(),
                    };
                }
            };

        match client.get("http://httpbin.org/ip").send().await {
                Ok(response) => {
                    let response_time = start.elapsed().as_millis() as u64;
                    ProxyHealth {
                        proxy_url: proxy_url.to_string(),
                        is_healthy: response.status().is_success(),
                        response_time,
                        error: None,
                        checked_at: chrono::Utc::now(),
                    }
                }
                Err(e) => ProxyHealth {
                    proxy_url: proxy_url.to_string(),
                    is_healthy: false,
                    response_time: 0,
                    error: Some(e.to_string()),
                    checked_at: chrono::Utc::now(),
                },
            }
    }
}

#[derive(Debug, Clone)]
struct RotationState {
    round_robin_index: usize,
    usage_count: HashMap<String, u32>,
    failure_counts: HashMap<String, u32>,
    total_requests: u64,
    successful_requests: u64,
    failed_requests: u64,
    total_response_time: u64,
    last_rotation: Option<chrono::DateTime<chrono::Utc>>,
}

impl RotationState {
    fn new() -> Self {
        Self {
            round_robin_index: 0,
            usage_count: HashMap::new(),
            failure_counts: HashMap::new(),
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            total_response_time: 0,
            last_rotation: None,
        }
    }

    fn mark_used(&mut self, proxy_url: &str) {
        *self.usage_count.entry(proxy_url.to_string()).or_insert(0) += 1;
        self.total_requests += 1;
        self.last_rotation = Some(chrono::Utc::now());
    }
}
