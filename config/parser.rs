use anyhow::Result;
use std::path::Path;
use serde_yaml;
use toml;
use crate::config::{CyberSpiderConfig, ConfigLoader};

pub struct ConfigParser;

impl ConfigParser {
    pub fn determine_format<P: AsRef<Path>>(path: P) -> Result<ConfigFormat> {
        let path_str = path.as_ref().to_string_lossy();
        
        if path_str.ends_with(".yaml") || path_str.ends_with(".yml") {
            Ok(ConfigFormat::Yaml)
        } else if path_str.ends_with(".toml") {
            Ok(ConfigFormat::Toml)
        } else if path_str.ends_with(".json") {
            Ok(ConfigFormat::Json)
        } else {
            Err(anyhow::anyhow!("Unsupported config format. Use .yaml, .yml, .toml, or .json"))
        }
    }
}

#[derive(Debug, Clone)]
pub enum ConfigFormat {
    Yaml,
    Toml,
    Json,
}

impl ConfigLoader for CyberSpiderConfig {
    fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let format = ConfigParser::determine_format(&path)?;
        let content = std::fs::read_to_string(path)?;
        
        match format {
            ConfigFormat::Yaml => {
                let config: CyberSpiderConfig = serde_yaml::from_str(&content)?;
                Ok(config)
            }
            ConfigFormat::Toml => {
                let config: CyberSpiderConfig = toml::from_str(&content)?;
                Ok(config)
            }
            ConfigFormat::Json => {
                let config: CyberSpiderConfig = serde_json::from_str(&content)?;
                Ok(config)
            }
        }
    }

    fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let format = ConfigParser::determine_format(&path)?;
        
        let content = match format {
            ConfigFormat::Yaml => serde_yaml::to_string(self)?,
            ConfigFormat::Toml => toml::to_string_pretty(self)?,
            ConfigFormat::Json => serde_json::to_string_pretty(self)?,
        };
        
        std::fs::write(path, content)?;
        Ok(())
    }

    fn load_from_env() -> Result<Self> {
        let mut config = CyberSpiderConfig::default();
        
        // Spider config from environment
        if let Ok(site) = std::env::var("CYBERSPIDER_SITE") {
            config.spider.site = Some(site);
        }
        
        if let Ok(threads) = std::env::var("CYBERSPIDER_THREADS") {
            config.spider.threads = threads.parse().unwrap_or(1);
        }
        
        if let Ok(concurrent) = std::env::var("CYBERSPIDER_CONCURRENT") {
            config.spider.concurrent = concurrent.parse().unwrap_or(5);
        }
        
        if let Ok(depth) = std::env::var("CYBERSPIDER_DEPTH") {
            config.spider.depth = depth.parse().unwrap_or(1);
        }
        
        if let Ok(delay) = std::env::var("CYBERSPIDER_DELAY") {
            config.spider.delay = delay.parse().unwrap_or(0);
        }
        
        if let Ok(timeout) = std::env::var("CYBERSPIDER_TIMEOUT") {
            config.spider.timeout = timeout.parse().unwrap_or(10);
        }
        
        if let Ok(theme) = std::env::var("CYBERSPIDER_THEME") {
            config.spider.progress_theme = theme;
        }
        
        // Database config from environment
        if let Ok(db_path) = std::env::var("CYBERSPIDER_DB_PATH") {
            config.database.sqlite_path = Some(db_path);
        }
        
        if let Ok(redis_url) = std::env::var("CYBERSPIDER_REDIS_URL") {
            config.database.redis_url = Some(redis_url);
        }
        
        // Browser config from environment
        if let Ok(headless) = std::env::var("CYBERSPIDER_HEADLESS") {
            config.browser.headless = headless.parse().unwrap_or(true);
        }
        
        if let Ok(js_enabled) = std::env::var("CYBERSPIDER_JS_ENABLED") {
            config.browser.enable_javascript = js_enabled.parse().unwrap_or(true);
        }
        
        // Security config from environment
        if let Ok(security_enabled) = std::env::var("CYBERSPIDER_SECURITY_ENABLED") {
            config.security.enabled = security_enabled.parse().unwrap_or(true);
        }
        
        // Monitoring config from environment
        if let Ok(log_level) = std::env::var("CYBERSPIDER_LOG_LEVEL") {
            config.monitoring.log_level = log_level;
        }
        
        Ok(config)
    }

    fn merge(&mut self, other: CyberSpiderConfig) {
        // Merge spider config
        if other.spider.site.is_some() {
            self.spider.site = other.spider.site;
        }
        if other.spider.sites_file.is_some() {
            self.spider.sites_file = other.spider.sites_file;
        }
        if other.spider.output_dir.is_some() {
            self.spider.output_dir = other.spider.output_dir;
        }
        if other.spider.threads != 1 {
            self.spider.threads = other.spider.threads;
        }
        if other.spider.concurrent != 5 {
            self.spider.concurrent = other.spider.concurrent;
        }
        if other.spider.depth != 1 {
            self.spider.depth = other.spider.depth;
        }
        if other.spider.delay != 0 {
            self.spider.delay = other.spider.delay;
        }
        if other.spider.timeout != 10 {
            self.spider.timeout = other.spider.timeout;
        }
        if other.spider.json_output {
            self.spider.json_output = other.spider.json_output;
        }
        if other.spider.verbose {
            self.spider.verbose = other.spider.verbose;
        }
        if other.spider.js_enabled {
            self.spider.js_enabled = other.spider.js_enabled;
        }
        if other.spider.sitemap_enabled {
            self.spider.sitemap_enabled = other.spider.sitemap_enabled;
        }
        if other.spider.robots_enabled {
            self.spider.robots_enabled = other.spider.robots_enabled;
        }
        if other.spider.other_sources_enabled {
            self.spider.other_sources_enabled = other.spider.other_sources_enabled;
        }
        if other.spider.progress_theme != "cyberwave" {
            self.spider.progress_theme = other.spider.progress_theme;
        }
        
        // Merge other configs (replace entire sections)
        if other.browser.enabled {
            self.browser = other.browser;
        }
        if other.database.enabled {
            self.database = other.database;
        }
        if other.security.enabled {
            self.security = other.security;
        }
        if other.monitoring.enabled {
            self.monitoring = other.monitoring;
        }
        if other.webhooks.enabled {
            self.webhooks = other.webhooks;
        }
        if other.plugins.enabled {
            self.plugins = other.plugins;
        }
    }
}
