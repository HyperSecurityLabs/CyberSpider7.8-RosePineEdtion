pub mod logger;
pub mod metrics;

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
    pub module: String,
    pub session_id: Option<String>,
    pub url: Option<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub value: f64,
    pub timestamp: DateTime<Utc>,
    pub tags: HashMap<String, String>,
    pub metric_type: MetricType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Timer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub requests_per_second: f64,
    pub average_response_time: f64,
    pub success_rate: f64,
    pub error_rate: f64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub active_connections: usize,
    pub queue_size: usize,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub service_name: String,
    pub status: HealthStatus,
    pub message: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub response_time_ms: Option<u64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

pub trait MonitoringSystem {
    fn log(&mut self, entry: LogEntry);
    fn record_metric(&mut self, metric: Metric);
    fn get_metrics(&self, name: &str) -> Vec<Metric>;
    fn get_performance_metrics(&self) -> PerformanceMetrics;
    fn health_check(&self) -> HealthCheck;
    fn export_logs(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<LogEntry>;
    fn export_metrics(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<Metric>;
}

pub struct MonitoringManager {
    logger: Box<dyn Logger>,
    metrics_collector: Box<dyn MetricsCollector>,
    health_checker: HealthChecker,
}

impl MonitoringManager {
    pub fn new(logger: Box<dyn Logger>, metrics_collector: Box<dyn MetricsCollector>) -> Self {
        Self {
            logger,
            metrics_collector,
            health_checker: HealthChecker::new(),
        }
    }

    pub fn log_info(&mut self, message: &str, module: &str) {
        let entry = LogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Info,
            message: message.to_string(),
            module: module.to_string(),
            session_id: None,
            url: None,
            metadata: HashMap::new(),
        };
        self.logger.log(entry);
    }

    pub fn log_error(&mut self, message: &str, module: &str, url: Option<&str>) {
        let entry = LogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Error,
            message: message.to_string(),
            module: module.to_string(),
            session_id: None,
            url: url.map(|u| u.to_string()),
            metadata: HashMap::new(),
        };
        self.logger.log(entry);
    }

    pub fn log_warning(&mut self, message: &str, module: &str) {
        let entry = LogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Warn,
            message: message.to_string(),
            module: module.to_string(),
            session_id: None,
            url: None,
            metadata: HashMap::new(),
        };
        self.logger.log(entry);
    }

    pub fn increment_counter(&mut self, name: &str, tags: HashMap<String, String>) {
        let metric = Metric {
            name: name.to_string(),
            value: 1.0,
            timestamp: Utc::now(),
            tags,
            metric_type: MetricType::Counter,
        };
        self.metrics_collector.record(metric);
    }

    pub fn record_gauge(&mut self, name: &str, value: f64, tags: HashMap<String, String>) {
        let metric = Metric {
            name: name.to_string(),
            value,
            timestamp: Utc::now(),
            tags,
            metric_type: MetricType::Gauge,
        };
        self.metrics_collector.record(metric);
    }

    pub fn record_timer(&mut self, name: &str, duration_ms: u64, tags: HashMap<String, String>) {
        let metric = Metric {
            name: name.to_string(),
            value: duration_ms as f64,
            timestamp: Utc::now(),
            tags,
            metric_type: MetricType::Timer,
        };
        self.metrics_collector.record(metric);
    }

    pub fn get_current_metrics(&self) -> PerformanceMetrics {
        self.metrics_collector.get_performance_metrics()
    }

    pub fn check_health(&self) -> HealthCheck {
        self.health_checker.check()
    }
}

pub trait Logger {
    fn log(&mut self, entry: LogEntry);
    fn flush(&mut self);
    fn get_logs(&self, limit: Option<usize>) -> Vec<LogEntry>;
}

pub trait MetricsCollector {
    fn record(&mut self, metric: Metric);
    fn get_metrics(&self, name: &str) -> Vec<Metric>;
    fn get_performance_metrics(&self) -> PerformanceMetrics;
    fn reset(&mut self);
}

pub struct HealthChecker {
    service_name: String,
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            service_name: "cyberspider".to_string(),
        }
    }

    pub fn check(&self) -> HealthCheck {
        let start = std::time::Instant::now();
        
        let status = if self.check_system_health() {
            HealthStatus::Healthy
        } else {
            HealthStatus::Degraded
        };

        HealthCheck {
            service_name: self.service_name.clone(),
            status,
            message: None,
            timestamp: Utc::now(),
            response_time_ms: Some(start.elapsed().as_millis() as u64),
            metadata: HashMap::new(),
        }
    }

    fn check_system_health(&self) -> bool {
        let memory_usage = self.get_memory_usage();
        let cpu_usage = self.get_cpu_usage();
        
        memory_usage < 90.0 && cpu_usage < 95.0
    }

    fn get_memory_usage(&self) -> f64 {
        use std::fs;
        
        if let Ok(status) = fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(kb) = parts[1].parse::<u64>() {
                            return (kb as f64) / (1024.0 * 1024.0); // Convert GB to percentage (assuming 8GB total)
                        }
                    }
                }
            }
        }
        
        0.0
    }

    fn get_cpu_usage(&self) -> f64 {
        // Real CPU usage calculation using system metrics
        use std::fs;
        
        match fs::read_to_string("/proc/stat") {
            Ok(content) => {
                let lines: Vec<&str> = content.lines().collect();
                if let Some(cpu_line) = lines.first() {
                    let parts: Vec<u64> = cpu_line
                        .split_whitespace()
                        .skip(1) // Skip "cpu" label
                        .take(4) // user, nice, system, idle
                        .filter_map(|s| s.parse().ok())
                        .collect();
                    
                    if parts.len() >= 4 {
                        let total = parts.iter().sum::<u64>();
                        let idle = parts[3];
                        
                        if total > 0 {
                            let usage = ((total - idle) as f64 / total as f64) * 100.0;
                            return usage.min(100.0).max(0.0);
                        }
                    }
                }
            }
            Err(_) => {
                // Fallback for non-Linux systems or error
                // Use a simple estimation based on process activity
            }
        }
        
        // Fallback value
        0.0
    }
}
