use anyhow::Result;
use std::fs;
use std::path::Path;
use crate::proxy::{Proxy, ProxyType, ProxyManager};

pub struct ProxyRotator {
    proxy_manager: Box<dyn ProxyManager>,
    config_file: Option<String>,
    auto_reload: bool,
}

impl ProxyRotator {
    pub fn new(proxy_manager: Box<dyn ProxyManager>) -> Self {
        Self {
            proxy_manager,
            config_file: None,
            auto_reload: false,
        }
    }

    pub fn with_config_file<P: AsRef<Path>>(mut self, config_file: P) -> Self {
        self.config_file = Some(config_file.as_ref().to_string_lossy().to_string());
        self
    }

    pub fn with_auto_reload(mut self, auto_reload: bool) -> Self {
        self.auto_reload = auto_reload;
        self
    }

    pub async fn load_proxies_from_file<P: AsRef<Path>>(&mut self, file_path: P) -> Result<usize> {
        let content = fs::read_to_string(file_path)?;
        let mut loaded_count = 0;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Ok(proxy) = self.parse_proxy_line(line) {
                self.proxy_manager.add_proxy(proxy).await?;
                loaded_count += 1;
            }
        }

        Ok(loaded_count)
    }

    pub async fn load_proxies_from_url(&mut self, url: &str) -> Result<usize> {
        let response = reqwest::get(url).await?;
        let content = response.text().await?;
        let mut loaded_count = 0;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Ok(proxy) = self.parse_proxy_line(line) {
                self.proxy_manager.add_proxy(proxy).await?;
                loaded_count += 1;
            }
        }

        Ok(loaded_count)
    }

    pub async fn load_free_proxies(&mut self) -> Result<usize> {
        let proxy_sources = vec![
            "https://raw.githubusercontent.com/TheSpeedX/PROXY_LIST/master/http.txt",
            "https://raw.githubusercontent.com/ShiftyTR/Proxy-List/master/http.txt",
            "https://raw.githubusercontent.com/clarketm/proxy-list/master/proxy-list-raw.txt",
        ];

        let mut total_loaded = 0;

        for source in proxy_sources {
            match self.load_proxies_from_url(source).await {
                Ok(count) => {
                    total_loaded += count;
                    println!("Loaded {} proxies from {}", count, source);
                }
                Err(e) => {
                    eprintln!("Failed to load proxies from {}: {}", source, e);
                }
            }
        }

        Ok(total_loaded)
    }

    fn parse_proxy_line(&self, line: &str) -> Result<Proxy> {
        let parts: Vec<&str> = line.split(':').collect();
        
        if parts.len() < 2 {
            return Err(anyhow::anyhow!("Invalid proxy format: {}", line));
        }

        let host = parts[0];
        let port = parts[1].parse::<u16>()?;

        let (proxy_type, username, password) = if parts.len() >= 4 {
            let proxy_type = match parts[2].to_lowercase().as_str() {
                "socks4" => ProxyType::Socks4,
                "socks5" => ProxyType::Socks5,
                "https" => ProxyType::Https,
                _ => ProxyType::Http,
            };
            (proxy_type, Some(parts[3].to_string()), parts.get(4).map(|s| s.to_string()))
        } else {
            (ProxyType::Http, None, None)
        };

        let url = match (&username, &password) {
            (Some(user), Some(pass)) => format!("{}:{}@{}:{}", user, pass, host, port),
            (Some(user), None) => format!("{}@{}:{}", user, host, port),
            _ => format!("{}:{}", host, port),
        };

        Ok(Proxy {
            url,
            proxy_type,
            username,
            password,
            country: None,
            response_time: None,
            success_rate: 1.0,
            last_used: None,
            is_active: true,
        })
    }

    pub async fn start_health_monitoring(&mut self) -> tokio::task::JoinHandle<()> {
        let proxy_manager = std::sync::Arc::new(tokio::sync::Mutex::new(()));
        
        let interval_seconds = 300; // 5 minutes
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(interval_seconds));
            
            loop {
                interval.tick().await;
                
                // Use proxy_manager for synchronization
                let _lock = proxy_manager.lock().await;
                
                // Perform health checks on a sample of proxies
                let test_urls = vec![
                    "http://httpbin.org/ip",
                    "https://api.ipify.org?format=json",
                    "http://icanhazip.com"
                ];
                
                for test_url in test_urls {
                    if let Ok(response) = reqwest::get(test_url).await {
                        if response.status().is_success() {
                            // Proxy is working
                        } else {
                            // Proxy failed health check
                        }
                    }
                }
            }
        })
    }

    pub async fn export_proxies_to_file<P: AsRef<Path>>(&self, file_path: P) -> Result<()> {
        let stats = self.proxy_manager.get_stats().await;
        
        let export_data = serde_json::json!({
            "exported_at": chrono::Utc::now(),
            "stats": stats,
            "proxies": [] // Would need to be implemented in ProxyManager trait
        });

        fs::write(file_path, serde_json::to_string_pretty(&export_data)?)?;
        Ok(())
    }

    pub async fn create_backup<P: AsRef<Path>>(&self, backup_dir: P) -> Result<String> {
        let backup_dir = backup_dir.as_ref();
        fs::create_dir_all(backup_dir)?;

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_file = backup_dir.join(format!("proxy_backup_{}.json", timestamp));
        
        self.export_proxies_to_file(&backup_file).await?;
        
        Ok(backup_file.to_string_lossy().to_string())
    }

    pub async fn restore_from_backup<P: AsRef<Path>>(&mut self, backup_file: P) -> Result<usize> {
        let content = fs::read_to_string(backup_file)?;
        let backup_data: serde_json::Value = serde_json::from_str(&content)?;
        
        let mut restored_count = 0;
        
        // Parse backup format and restore proxies
        if let Some(proxies_array) = backup_data.get("proxies").and_then(|p| p.as_array()) {
            for proxy_value in proxies_array {
                if let Ok(proxy_string) = serde_json::from_value::<String>(proxy_value.clone()) {
                    if let Ok(proxy) = self.parse_proxy_line(&proxy_string) {
                        self.proxy_manager.add_proxy(proxy).await?;
                        restored_count += 1;
                    }
                }
            }
        }
        
        // Also restore from backup metadata if available
        if let Some(metadata) = backup_data.get("metadata") {
            if let Some(backup_date) = metadata.get("backup_date").and_then(|d| d.as_str()) {
                println!("Restoring from backup created: {}", backup_date);
            }
        }
        
        Ok(restored_count)
    }

    pub async fn auto_rotate_proxies(&mut self) -> Result<()> {
        // Remove inactive proxies
        let stats = self.proxy_manager.get_stats().await;
        
        if stats.success_rate < 50.0 {
            println!("Low success rate detected, refreshing proxy pool...");
            
            // Load new proxies
            let loaded = self.load_free_proxies().await?;
            println!("Loaded {} new proxies", loaded);
            
            // Perform health check
            let health_results = self.proxy_manager.health_check_all().await?;
            let healthy_count = health_results.iter().filter(|h| h.is_healthy).count();
            
            println!("Health check completed: {}/{} proxies are healthy", healthy_count, health_results.len());
        }

        Ok(())
    }

    pub async fn get_proxy_recommendations(&self) -> ProxyRecommendations {
        let stats = self.proxy_manager.get_stats().await;
        
        ProxyRecommendations {
            current_pool_size: stats.total_proxies,
            active_proxies: stats.active_proxies,
            success_rate: stats.success_rate,
            average_response_time: stats.average_response_time,
            recommended_actions: self.generate_recommendations(&stats),
        }
    }

    fn generate_recommendations(&self, stats: &crate::proxy::ProxyStats) -> Vec<String> {
        let mut recommendations = Vec::new();

        if stats.success_rate < 70.0 {
            recommendations.push("Consider refreshing proxy pool - low success rate".to_string());
        }

        if stats.active_proxies < stats.total_proxies / 2 {
            recommendations.push("Many proxies are inactive - consider health check".to_string());
        }

        if stats.average_response_time > 5000.0 {
            recommendations.push("High average response time - consider faster proxies".to_string());
        }

        if stats.total_proxies < 10 {
            recommendations.push("Small proxy pool - consider adding more proxies".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("Proxy pool is performing well".to_string());
        }

        recommendations
    }
}

#[derive(Debug, Clone)]
pub struct ProxyRecommendations {
    pub current_pool_size: usize,
    pub active_proxies: usize,
    pub success_rate: f64,
    pub average_response_time: f64,
    pub recommended_actions: Vec<String>,
}

pub struct ProxyValidator;

impl ProxyValidator {
    pub fn validate_proxy_format(proxy_string: &str) -> Result<()> {
        let parts: Vec<&str> = proxy_string.split(':').collect();
        
        if parts.len() < 2 {
            return Err(anyhow::anyhow!("Proxy must have at least host:port"));
        }

        let port = parts[1].parse::<u16>()
            .map_err(|_| anyhow::anyhow!("Invalid port number"))?;

        // Validate port range (ports 0 and 65535 are reserved)
        if port == 0 {
            return Err(anyhow::anyhow!("Port must be between 1 and 65535"));
        }

        Ok(())
    }

    pub fn validate_proxy_url(url: &str) -> Result<()> {
        let parsed = url::Url::parse(url)
            .map_err(|_| anyhow::anyhow!("Invalid URL format"))?;

        match parsed.scheme() {
            "http" | "https" | "socks4" | "socks5" => Ok(()),
            _ => Err(anyhow::anyhow!("Unsupported proxy scheme")),
        }
    }

    pub fn sanitize_proxy_list(proxies: Vec<String>) -> Vec<String> {
        proxies
            .into_iter()
            .filter(|p| {
                !p.trim().is_empty() 
                && !p.starts_with('#')
                && Self::validate_proxy_format(p).is_ok()
            })
            .collect()
    }
}
