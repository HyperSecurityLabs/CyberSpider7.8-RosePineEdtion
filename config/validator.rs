use anyhow::Result;
use crate::config::CyberSpiderConfig;

pub struct ConfigValidator;

impl ConfigValidator {
    pub fn validate(config: &CyberSpiderConfig) -> Result<ValidationReport> {
        let mut report = ValidationReport::new();
        
        // Validate spider config
        Self::validate_spider_config(&config.spider, &mut report);
        
        // Validate browser config
        Self::validate_browser_config(&config.browser, &mut report);
        
        // Validate database config
        Self::validate_database_config(&config.database, &mut report);
        
        // Validate security config
        Self::validate_security_config(&config.security, &mut report);
        
        // Validate monitoring config
        Self::validate_monitoring_config(&config.monitoring, &mut report);
        
        // Validate webhook config
        Self::validate_webhook_config(&config.webhooks, &mut report);
        
        // Validate plugin config
        Self::validate_plugin_config(&config.plugins, &mut report);
        
        Ok(report)
    }
    
    fn validate_spider_config(config: &crate::config::SpiderConfig, report: &mut ValidationReport) {
        if config.threads == 0 {
            report.add_error("threads", "Threads must be greater than 0");
        }
        
        if config.concurrent == 0 {
            report.add_error("concurrent", "Concurrent requests must be greater than 0");
        }
        
        if config.concurrent > 100 {
            report.add_warning("concurrent", "High concurrent requests may cause rate limiting");
        }
        
        if config.timeout == 0 {
            report.add_error("timeout", "Timeout must be greater than 0");
        }
        
        if config.timeout > 300 {
            report.add_warning("timeout", "Very high timeout may slow down crawling");
        }
        
        if config.delay > 60 {
            report.add_warning("delay", "High delay may significantly slow down crawling");
        }
        
        if let Some(ref output_dir) = config.output_dir {
            if output_dir.is_empty() {
                report.add_error("output_dir", "Output directory cannot be empty");
            }
        }
        
        if !["cyberwave", "matrix", "neon", "terminal", "rosepine"].contains(&config.progress_theme.as_str()) {
            report.add_error("progress_theme", "Invalid progress theme. Must be one of: cyberwave, matrix, neon, terminal, rosepine");
        }
    }
    
    fn validate_browser_config(config: &crate::config::BrowserConfig, report: &mut ValidationReport) {
        if config.enabled {
            if config.viewport_width == 0 || config.viewport_height == 0 {
                report.add_error("viewport", "Viewport dimensions must be greater than 0");
            }
            
            if config.timeout_seconds == 0 {
                report.add_error("browser_timeout", "Browser timeout must be greater than 0");
            }
            
            if config.viewport_width > 4000 || config.viewport_height > 4000 {
                report.add_warning("viewport", "Very large viewport may cause performance issues");
            }
        }
    }
    
    fn validate_database_config(config: &crate::config::DatabaseConfig, report: &mut ValidationReport) {
        if config.enabled {
            if config.sqlite_path.is_none() && config.redis_url.is_none() {
                report.add_error("database", "At least one database (SQLite or Redis) must be configured");
            }
            
            if config.pool_size == 0 {
                report.add_error("pool_size", "Database pool size must be greater than 0");
            }
            
            if config.pool_size > 100 {
                report.add_warning("pool_size", "Large pool size may consume excessive memory");
            }
            
            if let Some(ref sqlite_path) = config.sqlite_path {
                if sqlite_path.is_empty() {
                    report.add_error("sqlite_path", "SQLite path cannot be empty");
                }
            }
            
            if let Some(ref redis_url) = config.redis_url {
                if !redis_url.starts_with("redis://") && !redis_url.starts_with("rediss://") {
                    report.add_error("redis_url", "Redis URL must start with redis:// or rediss://");
                }
            }
        }
    }
    
    fn validate_security_config(config: &crate::config::SecurityConfig, report: &mut ValidationReport) {
        if config.enabled {
            if config.severity_filter.is_empty() {
                report.add_warning("severity_filter", "No severity filter specified - all findings will be reported");
            }
            
            for severity in &config.severity_filter {
                if !["info", "low", "medium", "high", "critical"].contains(&severity.as_str()) {
                    report.add_error("severity_filter", &format!("Invalid severity level: {}", severity));
                }
            }
        }
    }
    
    fn validate_monitoring_config(config: &crate::config::MonitoringConfig, report: &mut ValidationReport) {
        if !["trace", "debug", "info", "warn", "error"].contains(&config.log_level.as_str()) {
            report.add_error("log_level", "Invalid log level. Must be one of: trace, debug, info, warn, error");
        }
    }
    
    fn validate_webhook_config(config: &crate::config::WebhookConfig, report: &mut ValidationReport) {
        if config.enabled {
            if config.url.is_none() {
                report.add_error("webhook_url", "Webhook URL is required when webhooks are enabled");
            } else if let Some(url) = &config.url {
                if !url.starts_with("http://") && !url.starts_with("https://") {
                    report.add_error("webhook_url", "Webhook URL must start with http:// or https://");
                }
            }
            
            if config.timeout == 0 {
                report.add_error("webhook_timeout", "Webhook timeout must be greater than 0");
            }
            
            if config.events.is_empty() {
                report.add_warning("webhook_events", "No webhook events specified");
            }
        }
    }
    
    fn validate_plugin_config(config: &crate::config::PluginConfig, report: &mut ValidationReport) {
        if config.enabled {
            if let Some(ref plugin_dir) = config.plugin_dir {
                if plugin_dir.is_empty() {
                    report.add_error("plugin_dir", "Plugin directory cannot be empty when plugins are enabled");
                } else if !std::path::Path::new(plugin_dir).exists() {
                    report.add_warning("plugin_dir", "Plugin directory does not exist");
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub is_valid: bool,
}

impl ValidationReport {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            is_valid: true,
        }
    }
    
    pub fn add_error(&mut self, field: &str, message: &str) {
        self.errors.push(ValidationError {
            field: field.to_string(),
            message: message.to_string(),
        });
        self.is_valid = false;
    }
    
    pub fn add_warning(&mut self, field: &str, message: &str) {
        self.warnings.push(ValidationWarning {
            field: field.to_string(),
            message: message.to_string(),
        });
    }
    
    pub fn print_report(&self) {
        if self.errors.is_empty() && self.warnings.is_empty() {
            println!("✓ Configuration is valid");
            return;
        }
        
        if !self.errors.is_empty() {
            println!("❌ Configuration errors:");
            for error in &self.errors {
                println!("  - {}: {}", error.field, error.message);
            }
        }
        
        if !self.warnings.is_empty() {
            println!("⚠  Configuration warnings:");
            for warning in &self.warnings {
                println!("  - {}: {}", warning.field, warning.message);
            }
        }
        
        if self.is_valid {
            println!("✓ Configuration is valid with warnings");
        } else {
            println!("❌ Configuration is invalid");
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct ValidationWarning {
    pub field: String,
    pub message: String,
}
