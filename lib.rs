//! # CyberSpider v7.8.0pro - Offensive Security Edition
//! # Today i will Make it Advanced, Lets Start The dirty Things With Russians.
//! An advanced offensive security web spider for deep reconnaissance, completely rewritten and enhanced for version 7.8.0pro.
//! Inspired by GoSpider but built with Rust for maximum performance and safety.
//! 
//! ## 🚀 Features
//! 
//! ### Core Spider Engine
//! - **High-performance crawling** with concurrent requests and async I/O
//! - **JavaScript-aware crawling** with headless browser integration
//! - **Sitemap and robots.txt parsing** for respectful crawling
//! - **3rd party source integration** (Wayback Machine, Common Crawl, VirusTotal)
//! - **AWS S3 bucket detection** with verification
//! - **Subdomain discovery** with multiple techniques
//! - **Multiple output formats** (JSON, text, CSV)
//! 
//! ### Security & Analysis
//! - **API endpoint discovery** with automatic detection
//! - **Form and input field detection** for attack surface analysis
//! - **Technology stack identification** (frameworks, CMS, versions)
//! - **Vulnerability scanning** with pattern matching
//! - **Security findings reporting** with severity classification
//! 
//! ### Advanced Features
//! - **Headless browser integration** for JavaScript-heavy sites
//! - **Database storage** with SQLite and Redis support
//! - **Configuration management** with YAML/TOML/JSON support
//! - **Plugin system** with dynamic loading capabilities
//! - **Structured logging and metrics** for monitoring
//! - **Webhook notifications** for real-time alerts
//! - **Graph visualization** for URL relationships
//! - **Proxy pool management** with rotation strategies
//! - **Session management** for authenticated crawling
//! - **Distributed crawling** for multi-machine coordination
//! 
//! ### CyberWave Experience
//! - **Professional progress indicators** with multiple themes
//! - **Beautiful terminal output** with colored formatting
//! - **Real-time statistics** and performance metrics
//! - **Interactive visualizations** and graph exports
//! 
//! ## 🎯 Quick Start
//! 
//! ### Basic Usage
//! 
//! ```rust
//! use cyberspider::{Spider, SpiderConfig};
//! 
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = SpiderConfig {
//!         site: Some("https://example.com".to_string()),
//!         depth: 2,
//!         concurrent: 10,
//!         js_enabled: true,
//!         ..Default::default()
//!     };
//!     
//!     let mut spider = Spider::new(config);
//!     let result = spider.run().await?;
//!     
//!     println!("Found {} URLs", result.discovered_urls.len());
//!     Ok(())
//! }
//! ```
//! 
//! ### Advanced Configuration
//! 
//! ```rust
//! use cyberspider::{CyberSpiderConfig, ConfigLoader};
//! 
//! let config = CyberSpiderConfig::load_from_file("config.yaml")?;
//! 
//! // Enable all advanced features
//! let mut advanced_config = config;
//! advanced_config.security.enabled = true;
//! advanced_config.browser.enabled = true;
//! advanced_config.database.enabled = true;
//! advanced_config.webhooks.enabled = true;
//! ```
//! 
//! ## 📚 Modules
//! 
//! - [`spider`] - Core spider engine and crawling logic
//! - [`detectors`] - S3 bucket and subdomain detection
//! - [`sources`] - External data sources integration
//! - [`output`] - Result formatting and export
//! - [`progress`] - CyberWave progress system
//! - [`security`] - Security scanning and analysis
//! - [`browser`] - Headless browser integration
//! - [`database`] - Database storage and queuing
//! - [`config`] - Configuration management
//! - [`plugins`] - Plugin system and extensibility
//! - [`monitoring`] - Logging and metrics collection
//! - [`webhooks`] - Webhook notifications
//! - [`visualization`] - Graph generation and exports
//! - [`proxy`] - Proxy pool management
//! - [`auth`] - Session and authentication management
//! - [`distributed`] - Multi-machine coordination
//! 
//! ## 🔧 Configuration
//! 
//! CyberSpider v7.8.0pro supports multiple configuration formats:
//! 
//! ### YAML Configuration
//! ```yaml
//! spider:
//!   site: "https://example.com"
//!   depth: 3
//!   concurrent: 15
//!   js_enabled: true
//!   progress_theme: "cyberwave"
//! 
//! security:
//!   enabled: true
//!   api_detection: true
//!   vuln_scanning: true
//! 
//! browser:
//!   enabled: true
//!   headless: true
//!   timeout_seconds: 30
//! 
//! database:
//!   enabled: true
//!   sqlite_path: "cyberspider.db"
//! 
//! monitoring:
//!   enabled: true
//!   log_level: "info"
//!   metrics_enabled: true
//! ```
//! 
//! ## 🌐 Distributed Crawling
//! 
//! CyberSpider v7.8.0pro supports distributed crawling across multiple machines:
//! 
//! ```bash
//! # Start coordinator
//! cyberspider --distributed --node-type coordinator
//! 
//! # Start worker
//! cyberspider --distributed --node-type worker
//! ```
//! 
//! ## 🔌 Plugin Development
//! 
//! Create custom plugins to extend CyberSpider's functionality:
//! 
//! ```rust
//! use cyberspider::plugins::{Plugin, PluginInfo, PluginContext, PluginResult};
//! 
//! pub struct MyCustomDetector;
//! 
//! #[async_trait::async_trait]
//! impl Plugin for MyCustomDetector {
//!     fn plugin_info(&self) -> PluginInfo {
//!         PluginInfo {
//!             name: "custom_detector".to_string(),
//!             version: "1.0.0".to_string(),
//!             description: "Custom detection plugin".to_string(),
//!             // ... other fields
//!         }
//!     }
//! 
//!     async fn execute(&mut self, context: &PluginContext) -> Result<PluginResult> {
//!         // Custom logic here
//!         Ok(PluginResult {
//!             success: true,
//!             data: Some(serde_json::json!({"found": true})),
//!             error: None,
//!             metadata: std::collections::HashMap::new(),
//!         })
//!     }
//! }
//! ```
//! 
//! ## 📊 Monitoring & Metrics
//! 
//! Built-in monitoring with Prometheus-compatible metrics:
//! 
//! ```rust
//! use cyberspider::monitoring::{MonitoringManager, InMemoryMetricsCollector};
//! 
//! let metrics = InMemoryMetricsCollector::new();
//! let manager = MonitoringManager::new(logger, Box::new(metrics));
//! 
//! // Automatic metrics collection
//! manager.increment_counter("requests_total", tags);
//! manager.record_gauge("active_connections", 42.0, tags);
//! ```
//! 
//! ## 🎨 Visualization
//! 
//! Generate beautiful graphs and visualizations:
//! 
//! ```rust
//! use cyberspider::visualization::{UrlGraph, GraphExporter};
//! 
//! let mut graph = UrlGraph::new();
//! // Add nodes and edges...
//! 
//! // Export to multiple formats
//! graph.to_dot()?;  // Graphviz
//! graph.to_mermaid()?;  // Mermaid
//! graph.to_json()?;  // Interactive web graph
//! ```
//! 
//! ## 🔒 Security Features
//! 
//! Comprehensive security scanning capabilities:
//! 
//! - **API Discovery**: Automatic REST API endpoint detection
//! - **Form Analysis**: Input field identification and validation
//! - **Technology Detection**: Framework, CMS, and version identification
//! - **Vulnerability Scanning**: Pattern-based vulnerability detection
//! - **Severity Classification**: Critical, High, Medium, Low, Info levels
//! 
//! ## 🌍 Proxy Support
//! 
//! Advanced proxy management with rotation strategies:
//! 
//! ```rust
//! use cyberspider::proxy::{ProxyPool, ProxyConfig, RotationStrategy};
//! 
//! let config = ProxyConfig {
//!     enabled: true,
//!     rotation_strategy: RotationStrategy::LeastUsed,
//!     health_check_interval: 300,
//!     max_failures: 3,
//!     // ... other config
//! };
//! 
//! let mut proxy_pool = ProxyPool::new(config);
//! proxy_pool.load_proxies_from_file("proxies.txt").await?;
//! ```
//! 
//! ## 📱 Webhook Integration
//! 
//! Real-time notifications with multiple platforms:
//! 
//! - **Slack**: Rich formatting with attachments
//! - **Discord**: Embedded cards and styling
//! - **Microsoft Teams**: Adaptive cards support
//! - **Custom HTTP**: Flexible webhook system
//! 
//! ## 🏗️ Architecture
//! 
//! CyberSpider v7.8.0pro is built with a modular, extensible architecture:
//! 
//! ```
//! ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
//! │   CLI Interface  │    │  Web Interface  │    │  API Gateway    │
//! └─────────────────┘    └─────────────────┘    └─────────────────┘
//!          │                       │                       │
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    Configuration Layer                        │
//! └─────────────────────────────────────────────────────────────────┘
//!          │
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    CyberSpider Core                             │
//! │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ │
//! │  │   Spider    │ │   Security  │ │   Browser   │ │   Database  │ │
//! │  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘ │
//! └─────────────────────────────────────────────────────────────────┘
//!          │
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                 Extension & Integration Layer                    │
//! │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ │
//! │  │   Plugins   │ │  Monitoring  │ │  Webhooks   │ │  Proxy      │ │
//! │  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘ │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//! 
//! ## 🤝 Contributing
//! 
//! We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.
//! 
//! ## 📄 License
//! 
//! This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
//! 
//! ## 🙏 Acknowledgments
//! 
//! - Inspired by [GoSpider](https://github.com/jaeles-project/gospider)
//! - Built with [Rust](https://www.rust-lang.org/)
//! - CyberWave UI design by Khaninkali
//! 
//! ## 📞 Support
//! 
//! - 📧 Email: khaninkali@example.com
//! - 💬 Telegram: https://t.me/hypersecurity_offsec
//! - 🐛 Issues: [GitHub Issues](https://github.com/khaninkali/cyberspider/issues)

pub mod spider;
pub mod detectors;
pub mod sources;
pub mod output;
pub mod progress;
pub mod cyberwave_progress;
pub mod r#box;
pub mod media_corruption;

// v7.8.0pro modules
pub mod security;
pub mod browser;
pub mod database;
pub mod config;
pub mod plugins;
pub mod monitoring;
pub mod webhooks;
pub mod visualization;
pub mod proxy;
pub mod auth;
pub mod distributed;

use serde::{Deserialize, Serialize};

/// Configuration for the spider crawling behavior
#[derive(Debug, Clone)]
pub struct SpiderConfig {
    /// Single site to crawl (mutually exclusive with sites_file)
    pub site: Option<String>,
    /// File containing list of sites to crawl
    pub sites_file: Option<String>,
    /// Output directory for results
    pub output_dir: Option<String>,
    /// Number of threads for parallel processing
    pub threads: usize,
    /// Maximum concurrent requests per domain
    pub concurrent: usize,
    /// Maximum recursion depth (0 for infinite)
    pub depth: usize,
    /// Delay between requests in seconds
    pub delay: u64,
    /// Request timeout in seconds
    pub timeout: u64,
    /// Output results in JSON format
    pub json_output: bool,
    /// Enable verbose logging
    pub verbose: bool,
    /// Enable JavaScript link extraction
    pub js_enabled: bool,
    /// Enable sitemap.xml parsing
    pub sitemap_enabled: bool,
    /// Enable robots.txt parsing
    pub robots_enabled: bool,
    /// Enable 3rd party source integration
    pub other_sources_enabled: bool,
    /// Progress bar theme
    pub progress_theme: String,
    /// Enable deep scanning with extended analysis
    pub deep_scan: bool,
    /// Check media files for corruption
    pub media_check: bool,
    /// Show active scanning modules
    pub show_modules: bool,
    /// Specialized tag mode (media-corruption, deep-scan, etc.)
    pub tag: Option<String>,
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
            tag: None,
        }
    }
}

/// Represents a discovered URL with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredUrl {
    /// The URL that was discovered
    pub url: String,
    /// Source where the URL was found (crawl, js, sitemap, etc.)
    pub source: String,
    /// HTTP status code if available
    pub status_code: Option<u16>,
    /// Content type if available
    pub content_type: Option<String>,
    /// Page title if available
    pub title: Option<String>,
    /// HTTP method used
    pub method: String,
}

/// Complete spider crawl results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpiderResult {
    /// Base domain that was crawled
    pub base_domain: String,
    /// All discovered URLs
    pub discovered_urls: Vec<DiscoveredUrl>,
    /// Discovered subdomains
    pub subdomains: Vec<String>,
    /// Discovered S3 buckets
    pub s3_buckets: Vec<String>,
    /// Total number of requests made
    pub total_requests: usize,
    /// Number of successful requests
    pub successful_requests: usize,
    /// Number of failed requests
    pub failed_requests: usize,
    /// Total crawl duration in milliseconds
    pub duration_ms: u64,
}

pub use spider::Spider;
