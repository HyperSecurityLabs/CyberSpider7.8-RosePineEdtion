pub mod parser;
pub mod validator;

use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CyberSpiderConfig {
    pub spider: SpiderConfig,
    pub browser: BrowserConfig,
    pub database: DatabaseConfig,
    pub security: SecurityConfig,
    pub monitoring: MonitoringConfig,
    pub webhooks: WebhookConfig,
    pub plugins: PluginConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpiderConfig {
    pub site: Option<String>,
    pub sites_file: Option<String>,
    pub output_dir: Option<String>,
    pub threads: usize,
    pub concurrent: usize,
    pub depth: usize,
    pub delay: u64,
    pub timeout: u64,
    pub json_output: bool,
    pub verbose: bool,
    pub js_enabled: bool,
    pub sitemap_enabled: bool,
    pub robots_enabled: bool,
    pub other_sources_enabled: bool,
    pub progress_theme: String,
    pub deep_scan: bool,
    pub media_check: bool,
    pub show_modules: bool,
    pub user_agent: String,
    pub max_file_size: usize,
    pub max_redirects: usize,
    pub respect_robots: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserConfig {
    pub enabled: bool,
    pub headless: bool,
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub timeout_seconds: u64,
    pub enable_javascript: bool,
    pub enable_images: bool,
    pub wait_for_load: u64,
    pub screenshot_on_error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub enabled: bool,
    pub sqlite_path: Option<String>,
    pub redis_url: Option<String>,
    pub pool_size: u32,
    pub connection_timeout: u64,
    pub cleanup_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub enabled: bool,
    pub api_detection: bool,
    pub form_detection: bool,
    pub tech_detection: bool,
    pub vuln_scanning: bool,
    pub severity_filter: Vec<String>,
    pub export_findings: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub enabled: bool,
    pub log_level: String,
    pub metrics_enabled: bool,
    pub real_time_stats: bool,
    pub performance_tracking: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub enabled: bool,
    pub url: Option<String>,
    pub events: Vec<String>,
    pub timeout: u64,
    pub retry_attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub enabled: bool,
    pub plugin_dir: Option<String>,
    pub auto_load: Vec<String>,
    pub disabled_plugins: Vec<String>,
}

impl Default for CyberSpiderConfig {
    fn default() -> Self {
        Self {
            spider: SpiderConfig::default(),
            browser: BrowserConfig::default(),
            database: DatabaseConfig::default(),
            security: SecurityConfig::default(),
            monitoring: MonitoringConfig::default(),
            webhooks: WebhookConfig::default(),
            plugins: PluginConfig::default(),
        }
    }
}

impl Default for SpiderConfig {
    fn default() -> Self {
        Self {
            site: None,
            sites_file: None,
            output_dir: None,
            threads: 2,
            concurrent: 5,
            depth: 1,
            delay: 0,
            timeout: 10,
            json_output: false,
            verbose: false,
            js_enabled: false,
            sitemap_enabled: false,
            robots_enabled: false,
            other_sources_enabled: false,
            progress_theme: "rosepine".to_string(),
            deep_scan: false,
            media_check: false,
            show_modules: false,
            user_agent: "CyberSpider/7.8.0pro".to_string(),
            max_file_size: 10485760, // 10MB
            max_redirects: 5,
            respect_robots: true,
        }
    }
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            headless: true,
            viewport_width: 1920,
            viewport_height: 1080,
            timeout_seconds: 30,
            enable_javascript: true,
            enable_images: false,
            wait_for_load: 3000,
            screenshot_on_error: false,
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            sqlite_path: Some("cyberspider.db".to_string()),
            redis_url: None,
            pool_size: 10,
            connection_timeout: 30,
            cleanup_days: 30,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            api_detection: true,
            form_detection: true,
            tech_detection: true,
            vuln_scanning: true,
            severity_filter: vec!["medium".to_string(), "high".to_string(), "critical".to_string()],
            export_findings: false,
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            log_level: "info".to_string(),
            metrics_enabled: false,
            real_time_stats: true,
            performance_tracking: false,
        }
    }
}

impl Default for WebhookConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            url: None,
            events: vec![],
            timeout: 10,
            retry_attempts: 3,
        }
    }
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            plugin_dir: Some("plugins".to_string()),
            auto_load: vec![],
            disabled_plugins: vec![],
        }
    }
}

pub trait ConfigLoader {
    fn load_from_file<P: AsRef<Path>>(path: P) -> Result<CyberSpiderConfig>;
    fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()>;
    fn load_from_env() -> Result<CyberSpiderConfig>;
    fn merge(&mut self, other: CyberSpiderConfig);
}
