use clap::{Arg, 
               Command};
use colored::*;
use std::error::Error;
use std::process;
use cyberspider::{Spider, SpiderConfig};
use cyberspider::config::{
                      CyberSpiderConfig,
                       ConfigLoader};
use cyberspider::config::validator::ConfigValidator;
use cyberspider::cyberwave_progress::
                {CyberWaveProgress, 
                ProgressTheme, CrawlStats, BrailleSpinner};
use cyberspider::database::{Database, sqlite::SQLiteDatabase};
use cyberspider::plugins::PluginManager;
use cyberspider::monitoring::{MonitoringManager};
use cyberspider::monitoring::logger::StructuredLogger;
use cyberspider::monitoring::metrics::InMemoryMetricsCollector;
use cyberspider::webhooks::WebhookManager;
use cyberspider::visualization::GraphExport;
use cyberspider::proxy::{ProxyManager, ProxyPool, ProxyConfig};
use cyberspider::distributed::{DistributedSpider, DistributedConfig, NodeType};
use cyberspider::media_corruption::MediaCorruptionAttacker;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Check for --no-banner before clap parsing so banner shows even on errors
    if !std::env::args().any(|a| a == "--no-banner") {
        display_banner();
    }

    let matches = Command::new("CyberSpider")
        .version("7.8.0pro")
        .author("Khaninkali")
        .about("Advanced offensive security web spider for deep reconnaissance - RoséPine Evergreen Edition")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Configuration file (YAML/TOML/JSON)")
        )
        .arg(
            Arg::new("site")
                .short('s')
                .long("site")
                .value_name("URL")
                .help("Site to crawl")
                .required_unless_present_any(&["config", "sites", "distributed"])
        )
        .arg(
            Arg::new("sites")
                .short('S')
                .long("sites")
                .value_name("FILE")
                .help("Site list to crawl")
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("DIR")
                .help("Output folder")
        )
        .arg(
            Arg::new("threads")
                .short('j')
                .long("threads")
                .value_name("NUM")
                .help("Number of threads (default: 2)")
                .default_value("2")
                .value_parser(clap::value_parser!(usize))
        )
        .arg(
            Arg::new("concurrent")
                .short('C')
                .long("concurrent")
                .value_name("NUM")
                .help("Maximum concurrent requests per domain")
                .default_value("5")
                .value_parser(clap::value_parser!(usize))
        )
        .arg(
            Arg::new("depth")
                .short('d')
                .long("depth")
                .value_name("NUM")
                .help("Max recursion depth (1-10)")
                .default_value("1")
                .value_parser(clap::value_parser!(usize))
        )
        .arg(
            Arg::new("delay")
                .short('k')
                .long("delay")
                .value_name("SEC")
                .help("Delay between requests (seconds)")
                .default_value("0")
                .value_parser(clap::value_parser!(u64))
        )
        .arg(
            Arg::new("timeout")
                .short('m')
                .long("timeout")
                .value_name("SEC")
                .help("Request timeout (seconds)")
                .default_value("10")
                .value_parser(clap::value_parser!(u64))
        )
        .arg(
            Arg::new("json")
                .long("json")
                .help("Enable JSON output")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Verbose output")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("js")
                .long("js")
                .help("Enable link finder in JavaScript files")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("sitemap")
                .long("sitemap")
                .help("Try to crawl sitemap.xml")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("robots")
                .long("robots")
                .help("Try to crawl robots.txt")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("other-sources")
                .short('a')
                .long("other-sources")
                .help("Find URLs from 3rd party sources")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("deep-scan")
                .long("deep-scan")
                .help("Enable deep scanning with extended analysis")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("media-check")
                .long("media-check")
                .help("Check media files for corruption and anomalies")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("show-modules")
                .long("show-modules")
                .help("Show active scanning modules during crawl")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("no-banner")
                .long("no-banner")
                .help("Suppress banner display on startup")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("tag")
                .long("tag")
                .value_name("MODE")
                .help("Run specialized tag mode: media-corruption (aggressive media attack)")
                .value_parser(["media-corruption", "deep-scan"])
        )
        .arg(
            Arg::new("browser")
                .long("browser")
                .help("Enable headless browser for JavaScript-heavy sites")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("security")
                .long("security")
                .help("Enable security scanning")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("database")
                .long("database")
                .help("Enable database storage")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("plugins")
                .long("plugins")
                .value_name("DIR")
                .help("Load plugins from directory")
        )
        .arg(
            Arg::new("webhooks")
                .long("webhooks")
                .help("Enable webhook notifications")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("webhook-url")
                .long("webhook-url")
                .value_name("URL")
                .help("Webhook URL for notifications (required when --webhooks is enabled)")
        )
        .arg(
            Arg::new("visualize")
                .long("visualize")
                .help("Generate visualization graphs")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("proxy")
                .long("proxy")
                .value_name("URL")
                .help("Use proxy for all requests")
        )
        .arg(
            Arg::new("distributed")
                .long("distributed")
                .help("Run in distributed mode")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("node-type")
                .long("node-type")
                .value_name("TYPE")
                .help("Node type for distributed mode (coordinator, worker)")
                .value_parser(["coordinator", "worker"])
        )
        .get_matches();

    // Handle distributed mode
    if matches.get_flag("distributed") {
        return run_distributed_mode(&matches).await;
    }

    let progress = CyberWaveProgress::new(ProgressTheme::RosePine);
    let spinner = progress.create_spinner("CyberSpider v7.8.0pro Evergreen - Initializing...");

    // Load configuration
    let mut cyber_config = if let Some(config_file) = matches.get_one::<String>("config") {
        match CyberSpiderConfig::load_from_file(config_file) {
            Ok(config) => {
                spinner.set_message("Configuration loaded");
                config
            },
            Err(e) => {
                eprintln!("Error loading config file: {}", e);
                process::exit(1);
            }
        }
    } else {
        spinner.set_message("Using default configuration");
        CyberSpiderConfig::default()
    };

    // Override config with command line arguments
    if let Some(site) = matches.get_one::<String>("site") {
        cyber_config.spider.site = Some(site.clone());
    }
    if let Some(sites_file) = matches.get_one::<String>("sites") {
        cyber_config.spider.sites_file = Some(sites_file.clone());
    }
    if let Some(output_dir) = matches.get_one::<String>("output") {
        cyber_config.spider.output_dir = Some(output_dir.clone());
    }

    cyber_config.spider.threads = *matches.get_one::<usize>("threads").unwrap_or(&2);
    cyber_config.spider.concurrent = *matches.get_one::<usize>("concurrent").unwrap_or(&5);
    cyber_config.spider.depth = *matches.get_one::<usize>("depth").unwrap_or(&1);
    cyber_config.spider.delay = *matches.get_one::<u64>("delay").unwrap_or(&0);
    cyber_config.spider.timeout = *matches.get_one::<u64>("timeout").unwrap_or(&10);
    cyber_config.spider.json_output = matches.get_flag("json");
    cyber_config.spider.verbose = matches.get_flag("verbose");

    cyber_config.spider.js_enabled = matches.get_flag("js");
    cyber_config.spider.sitemap_enabled = matches.get_flag("sitemap");
    cyber_config.spider.robots_enabled = matches.get_flag("robots");
    cyber_config.spider.other_sources_enabled = matches.get_flag("other-sources");

    cyber_config.spider.progress_theme = "rosepine".to_string();

    cyber_config.spider.deep_scan = matches.get_flag("deep-scan");
    cyber_config.spider.media_check = matches.get_flag("media-check");
    cyber_config.spider.show_modules = matches.get_flag("show-modules");

    cyber_config.browser.enabled = matches.get_flag("browser");
    cyber_config.security.enabled = matches.get_flag("security");
    cyber_config.database.enabled = matches.get_flag("database");
    cyber_config.webhooks.enabled = matches.get_flag("webhooks");

    if let Some(webhook_url) = matches.get_one::<String>("webhook-url") {
        cyber_config.webhooks.url = Some(webhook_url.clone());
    }

    cyber_config.monitoring.enabled = true;

    spinner.set_message("Validating configuration...");

    let validation_report = ConfigValidator::validate(&cyber_config);
    let report = validation_report?;
    if !report.is_valid {
        eprintln!("Configuration validation failed:");
        report.print_report();
        process::exit(1);
    }

    spinner.set_message("CyberSpider engine ready - Starting reconnaissance...");
    match run_cyberspider_v7(cyber_config, &matches, &spinner).await {
        Ok(_) => {
            spinner.set_message("CyberSpider v7.8.0pro completed successfully");
            println!("CyberSpider v7.8.0pro completed successfully");
            Ok(())
        },
        Err(e) => {
            spinner.set_message("CyberSpider encountered an error");
            eprintln!("Error: {}", e);
            Err(e)
        }
    }
}

async fn run_distributed_mode(args: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let node_type = args.get_one::<String>("node-type")
        .map(|s| s.as_str())
        .unwrap_or("coordinator");

    let node_type_enum = match node_type {
        "coordinator" => NodeType::Coordinator,
        "worker" => NodeType::Worker,
        _ => NodeType::Hybrid,
    };

    let distributed_config = DistributedConfig {
        node_id: format!("node_{}", uuid::Uuid::new_v4()),
        node_type: node_type_enum,
        coordinator_address: None,
        worker_address: "127.0.0.1:8080".to_string(),
        heartbeat_interval_seconds: 30,
        task_timeout_seconds: 300,
        max_concurrent_tasks: 10,
        enable_auto_scaling: false,
    };

    let mut distributed_spider = DistributedSpider::new(distributed_config);
    distributed_spider.start().await?;

    Ok(())
}

fn generate_yaml_config(config: &CyberSpiderConfig, _matches: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    use serde_yaml;
    
    let config_content = serde_yaml::to_string(config)?;
    let config_filename = "cyberspider_auto_config.yaml";
    
    std::fs::write(config_filename, config_content)?;
    
    println!("[+] Auto-generated configuration file: {}", config_filename);
    println!("[+] You can reuse this configuration with: --config {}", config_filename);
    
    Ok(())
}

async fn run_cyberspider_v7(config: CyberSpiderConfig, matches: &clap::ArgMatches, spinner: &BrailleSpinner) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize CyberWave progress
    let theme = match config.spider.progress_theme.as_str() {
        "cyberwave" => ProgressTheme::CyberWave,
        "matrix" => ProgressTheme::Matrix,
        "neon" => ProgressTheme::Neon,
        "terminal" => ProgressTheme::Terminal,
        _ => ProgressTheme::RosePine,
    };
    
    let progress = CyberWaveProgress::new(theme);

    // Initialize monitoring
    let logger_config = cyberspider::monitoring::logger::LoggerConfig {
        file_path: Some("cyberspider.log".to_string()),
        console_output: config.spider.verbose,
        log_level: match config.monitoring.log_level.as_str() {
            "trace" => cyberspider::monitoring::LogLevel::Trace,
            "debug" => cyberspider::monitoring::LogLevel::Debug,
            "warn" => cyberspider::monitoring::LogLevel::Warn,
            "error" => cyberspider::monitoring::LogLevel::Error,
            _ => cyberspider::monitoring::LogLevel::Info,
        },
        max_buffer_size: 10000,
        rotation_size_mb: Some(100),
        compression: false,
    };

    let logger = Box::new(StructuredLogger::new(logger_config)?);
    let metrics_collector = Box::new(InMemoryMetricsCollector::new());
    let mut monitoring_manager = MonitoringManager::new(logger, metrics_collector);

    monitoring_manager.log_info("Starting CyberSpider v7.8.0pro", "main");

    // Initialize database if enabled
    let mut _database: Option<Box<dyn Database>> = None;
    if config.database.enabled {
        // Auto-generate database path if not set
        let db_path = if let Some(path) = &config.database.sqlite_path {
            path.clone()
        } else {
            // Create database in output directory or current directory
            let default_path = if let Some(output_dir) = &config.spider.output_dir {
                format!("{}/cyberspider.db", output_dir)
            } else {
                "cyberspider.db".to_string()
            };
            default_path
        };
        
        // Ensure directory exists
        if let Some(parent) = std::path::Path::new(&db_path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        // Create empty database file if it doesn't exist
        if !std::path::Path::new(&db_path).exists() {
            std::fs::File::create(&db_path)?;
        }
        
        // Create database connection
        let connection_string = format!("sqlite:{}", db_path);
        monitoring_manager.log_info(&format!("Connecting to database: {}", connection_string), "database");
        
        let db = SQLiteDatabase::new(&connection_string).await?;
        _database = Some(Box::new(db));
        monitoring_manager.log_info(&format!("Database created at: {}", db_path), "database");
        
        // Update config with actual database path for YAML generation
        let mut config_for_yaml = config.clone();
        config_for_yaml.database.sqlite_path = Some(db_path.clone());
        
        // Auto-generate YAML config file
        generate_yaml_config(&config_for_yaml, &matches)?;
    }

    // Initialize proxy if configured
    let mut _proxy_manager: Option<Box<dyn ProxyManager>> = None;
    if let Some(proxy_url) = matches.get_one::<String>("proxy") {
        let proxy_url_str: String = proxy_url.clone();
        let proxy_config = ProxyConfig {
            enabled: true,
            rotation_strategy: cyberspider::proxy::RotationStrategy::RoundRobin,
            health_check_interval: 300,
            max_failures: 3,
            timeout_seconds: config.spider.timeout,
            exclude_domains: vec![],
            include_domains: vec![],
        };
        
        let mut proxy_pool = ProxyPool::new(proxy_config);
        let proxy = cyberspider::proxy::Proxy {
            url: proxy_url_str.clone(),
            proxy_type: cyberspider::proxy::ProxyType::Http,
            username: None,
            password: None,
            country: None,
            response_time: None,
            success_rate: 1.0,
            last_used: None,
            is_active: true,
        };
        proxy_pool.add_proxy(proxy).await?;
        _proxy_manager = Some(Box::new(proxy_pool));
        
        monitoring_manager.log_info(&format!("Proxy configured: {}", proxy_url_str), "proxy");
    }

    // Initialize webhooks if enabled
    let mut _webhook_manager: Option<WebhookManager> = None;
    if config.webhooks.enabled {
        _webhook_manager = Some(WebhookManager::new());
        monitoring_manager.log_info("Webhook notifications enabled", "webhooks");
    }

    // Initialize plugins if configured
    let mut _plugin_manager: Option<PluginManager> = None;
    if let Some(plugin_dir) = &config.plugins.plugin_dir {
        _plugin_manager = Some(PluginManager::new());
        if let Some(pm) = _plugin_manager.as_mut() {
            let loaded_count = pm.load_plugins_from_dir(plugin_dir).await?;
            monitoring_manager.log_info(&format!("Loaded {} plugins", loaded_count), "plugins");
        }
    }

    // Create spider config
    let spider_config = SpiderConfig {
        site: config.spider.site.clone(),
        sites_file: config.spider.sites_file.clone(),
        output_dir: config.spider.output_dir.clone(),
        threads: config.spider.threads,
        concurrent: config.spider.concurrent,
        depth: config.spider.depth,
        delay: config.spider.delay,
        timeout: config.spider.timeout,
        json_output: config.spider.json_output,
        verbose: config.spider.verbose,
        js_enabled: config.spider.js_enabled,
        sitemap_enabled: config.spider.sitemap_enabled,
        robots_enabled: config.spider.robots_enabled,
        other_sources_enabled: config.spider.other_sources_enabled,
        progress_theme: config.spider.progress_theme.clone(),
        deep_scan: config.spider.deep_scan,
        media_check: config.spider.media_check,
        show_modules: config.spider.show_modules,
        tag: matches.get_one::<String>("tag").cloned(),
    };

    // Run spider
    let attack_tag = spider_config.tag.clone();
    let mut spider = Spider::new(spider_config);
    let result = spider.run(spinner).await?;

    // ── TAG MODE: Media Corruption Campaign ──────────────────────
    let tag = &attack_tag;
    if let Some(tag_mode) = tag {
        if tag_mode == "media-corruption" {
            let attacker = MediaCorruptionAttacker::new();
            let domain = result.base_domain.clone();
            let media_urls: Vec<String> = result.discovered_urls.iter()
                .map(|u| u.url.clone())
                .collect();

            spinner.set_message(&format!("[TAG: media-corruption] Attacking {} media endpoints...", media_urls.len()));

            // Phase 1: Attack every discovered media URL
            for url in &media_urls {
                let corr_result = attacker.corrupt_url(url).await;
                if corr_result.success {
                    spinner.log(&format!("{} {} via {} — {}",
                        "[CORRUPTED]".truecolor(80, 250, 123).bold(),
                        corr_result.url.truecolor(80, 250, 123),
                        corr_result.method.truecolor(246, 193, 119),
                        corr_result.detail.truecolor(156, 207, 216)));
                } else if corr_result.method != "none" {
                    spinner.log(&format!("{} {} — {}",
                        "[FAILED]".truecolor(235, 111, 146).bold(),
                        corr_result.url.truecolor(80, 250, 123),
                        corr_result.detail.truecolor(156, 207, 216)));
                }
            }

            // Phase 2: Discover and attack upload endpoints
            spinner.set_message("[TAG: media-corruption] Probing upload endpoints...");
            let endpoints = attacker.discover_all_endpoints(&domain).await;
            if !endpoints.is_empty() {
                spinner.log(&format!("{} Found {} live endpoints — attacking...",
                    "[+]".truecolor(80, 250, 123).bold(),
                    endpoints.len().to_string().truecolor(246, 193, 119)));
                for ep in &endpoints {
                    spinner.log(&format!("  [+] {}", ep.truecolor(80, 250, 123)));
                }
            }

            // Phase 3: Scan admin/media paths for accessible pages
            spinner.set_message("[TAG: media-corruption] Scanning admin paths...");
            let admin_urls = attacker.scan_admin_paths(&domain).await;
            for url in &admin_urls {
                spinner.log(&format!("{} Page accessible: {}",
                    "[DISCOVERED]".truecolor(246, 193, 119).bold(),
                    url.truecolor(80, 250, 123)));
            }

            spinner.set_message("[TAG: media-corruption] Campaign complete");
        }
    }

    // Display results with CyberWave styling
    let mut crawl_stats = CrawlStats {
        total_requests: result.total_requests,
        successful_requests: result.successful_requests,
        failed_requests: result.failed_requests,
        urls_discovered: result.discovered_urls.len(),
        subdomains_found: result.subdomains.len(),
        s3_buckets_found: result.s3_buckets.len(),
        duration_ms: result.duration_ms,
        requests_per_second: 0.0,
    };
    crawl_stats.calculate_rps();
    
    progress.display_stats(&crawl_stats);

    // Display discovered URLs
    if !result.discovered_urls.is_empty() {
        println!("\n[+] Discovered URLs:");
        for (i, url) in result.discovered_urls.iter().enumerate() {
            println!("  {}. {} [{}] {}", 
                i + 1, 
                url.url, 
                url.status_code.unwrap_or(0),
                url.content_type.as_deref().unwrap_or("unknown")
            );
        }
        println!("Total URLs discovered: {}\n", result.discovered_urls.len());
    } else {
        println!("\n[*]  No URLs discovered. Check if the target site is accessible.");
    }

    // Display discovered subdomains
    if !result.subdomains.is_empty() {
        println!("🌐 Discovered Subdomains:");
        for (i, subdomain) in result.subdomains.iter().enumerate() {
            println!("  {}. {}", i + 1, subdomain);
        }
        println!("Total subdomains found: {}\n", result.subdomains.len());
    }

    // Display S3 buckets
    if !result.s3_buckets.is_empty() {
        println!("📦 Discovered S3 Buckets:");
        for (i, bucket) in result.s3_buckets.iter().enumerate() {
            println!("  {}. {}", i + 1, bucket);
        }
        println!("Total S3 buckets found: {}\n", result.s3_buckets.len());
    }

    // Generate visualizations if enabled
    if matches.get_flag("visualize") {
        generate_visualizations(&result, &config).await?;
    }

    monitoring_manager.log_info("CyberSpider v7.8.0pro completed", "main");

    Ok(())
}

async fn generate_visualizations(
    result: &cyberspider::SpiderResult,
    config: &CyberSpiderConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating visualizations...");
    
    // Create URL graph with real data
    let url_nodes: Vec<cyberspider::visualization::UrlNode> = result.discovered_urls
        .iter()
        .enumerate()
        .map(|(i, url)| {
            cyberspider::visualization::UrlNode {
                url: url.url.clone(),
                title: url.title.clone(),
                status_code: url.status_code,
                content_type: url.content_type.clone(),
                depth: i, // Use enumeration as depth approximation
                parent_url: None,
                discovered_at: chrono::Utc::now(),
            }
        })
        .collect();
    
    // Use config for visualization settings
    println!("Using visualization config: threads={}, depth={}", config.spider.threads, config.spider.depth);

    let mut url_graph = cyberspider::visualization::UrlGraph::new();
    let mut _node_indices = Vec::new();
    
    // Add nodes
    for node in &url_nodes {
        _node_indices.push(url_graph.add_node(node.clone()));
    }
    
    // Create edges based on actual URL relationships
    // For now, create logical connections between URLs from the same domain
    let mut edges_created = 0;
    for (i, node_i) in url_nodes.iter().enumerate() {
        for (j, node_j) in url_nodes.iter().enumerate() {
            if i < j {
                // Create edges between URLs from the same domain or related patterns
                if should_create_edge(&node_i.url, &node_j.url) {
                    let edge = cyberspider::visualization::UrlEdge {
                        source: node_i.url.clone(),
                        target: node_j.url.clone(),
                        link_type: determine_link_type(&node_i.url, &node_j.url),
                        anchor_text: Some("discovered".to_string()),
                    };
                    url_graph.add_edge(edge);
                    edges_created += 1;
                }
            }
        }
    }
    
    println!("Created {} edges between URLs", edges_created);

    // Export graph
    let dot_content = url_graph.to_dot()?;
    std::fs::write("cyberspider_graph.dot", dot_content)?;

    println!("Visualization files generated: cyberspider_graph.dot");
    
    Ok(())
}

// Helper function to determine if we should create an edge between two URLs
fn should_create_edge(url1: &str, url2: &str) -> bool {
    use url::Url;
    
    if let (Ok(parsed1), Ok(parsed2)) = (Url::parse(url1), Url::parse(url2)) {
        // Connect URLs from the same domain
        if let (Some(domain1), Some(domain2)) = (parsed1.host_str(), parsed2.host_str()) {
            return domain1 == domain2;
        }
    }
    
    false
}

// Helper function to determine link type between URLs
fn determine_link_type(url1: &str, url2: &str) -> cyberspider::visualization::LinkType {
    use url::Url;
    
    if let (Ok(parsed1), Ok(parsed2)) = (Url::parse(url1), Url::parse(url2)) {
        // Different subdomains of same domain
        if let (Some(domain1), Some(domain2)) = (parsed1.host_str(), parsed2.host_str()) {
            if domain1 != domain2 {
                return cyberspider::visualization::LinkType::External;
            }
        }
    }
    
    cyberspider::visualization::LinkType::Direct
}

fn display_banner() {
    let progress = CyberWaveProgress::new(ProgressTheme::RosePine);
    progress.display_cyberwave_logo();
}
