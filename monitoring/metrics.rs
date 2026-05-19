use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use anyhow::Result;
use chrono::Utc;
use sysinfo::System;
use crate::monitoring::{Metric, MetricType, MetricsCollector, PerformanceMetrics};

/// Helper function for safe mutex locking in monitoring
fn safe_mutex_lock<T>(mutex: &Mutex<T>) -> Result<std::sync::MutexGuard<'_, T>>
where
    T: std::fmt::Debug,
{
    mutex.lock().map_err(|e| anyhow::anyhow!("Monitoring mutex poisoned: {:?}", e))
}

#[derive(Debug)]
pub struct InMemoryMetricsCollector {
    metrics: Arc<Mutex<HashMap<String, Vec<Metric>>>>,
    counters: Arc<Mutex<HashMap<String, f64>>>,
    gauges: Arc<Mutex<HashMap<String, f64>>>,
    histograms: Arc<Mutex<HashMap<String, Vec<f64>>>>,
    timers: Arc<Mutex<HashMap<String, Vec<Duration>>>>,
}

impl InMemoryMetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(HashMap::new())),
            counters: Arc::new(Mutex::new(HashMap::new())),
            gauges: Arc::new(Mutex::new(HashMap::new())),
            histograms: Arc::new(Mutex::new(HashMap::new())),
            timers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn get_performance_metrics_internal(&self) -> Result<PerformanceMetrics> {
        let counters = safe_mutex_lock(&self.counters)?;
        let gauges = safe_mutex_lock(&self.gauges)?;
        let histograms = safe_mutex_lock(&self.histograms)?;
        let timers = safe_mutex_lock(&self.timers)?;

        let total_requests = counters.get("total_requests").unwrap_or(&0.0);
        let successful_requests = counters.get("successful_requests").unwrap_or(&0.0);
        let failed_requests = counters.get("failed_requests").unwrap_or(&0.0);

        let success_rate = if *total_requests > 0.0 {
            successful_requests / total_requests * 100.0
        } else {
            0.0
        };

        let error_rate = if *total_requests > 0.0 {
            failed_requests / total_requests * 100.0
        } else {
            0.0
        };

        let average_response_time = if let Some(response_times) = histograms.get("response_times") {
            if !response_times.is_empty() {
                // Use percentile functions for advanced metrics analysis
                let _p95_response_time = Self::calculate_percentile(response_times, 95.0);
                let _p99_response_time = Self::calculate_percentile(response_times, 99.0);
                let average_response_time = response_times.iter().sum::<f64>() / response_times.len() as f64;
                average_response_time
            } else {
                0.0
            }
        } else {
            0.0
        };

        let requests_per_second = if let Some(timer_data) = timers.get("request_durations") {
            if !timer_data.is_empty() {
                let total_time: Duration = timer_data.iter().sum();
                let avg_time = total_time / timer_data.len() as u32;
                if avg_time.as_secs_f64() > 0.0 {
                    1.0 / avg_time.as_secs_f64()
                } else {
                    0.0
                }
            } else {
                0.0
            }
        } else {
            0.0
        };

        let memory_usage_mb = *gauges.get("memory_usage_mb").unwrap_or(&0.0);
        let cpu_usage_percent = *gauges.get("cpu_usage_percent").unwrap_or(&0.0);
        let active_connections = *gauges.get("active_connections").unwrap_or(&0.0) as usize;
        let queue_size = *gauges.get("queue_size").unwrap_or(&0.0) as usize;

        Ok(PerformanceMetrics {
            requests_per_second,
            average_response_time,
            success_rate,
            error_rate,
            memory_usage_mb,
            cpu_usage_percent,
            active_connections,
            queue_size,
            timestamp: Utc::now(),
        })
    }

    fn reset_internal(&mut self) -> Result<()> {
        let mut metrics = safe_mutex_lock(&self.metrics)?;
        metrics.clear();

        let mut counters = safe_mutex_lock(&self.counters)?;
        counters.clear();

        let mut gauges = safe_mutex_lock(&self.gauges)?;
        gauges.clear();

        let mut histograms = safe_mutex_lock(&self.histograms)?;
        histograms.clear();

        let mut timers = safe_mutex_lock(&self.timers)?;
        timers.clear();
        Ok(())
    }

    pub fn increment_counter(&self, name: &str, value: f64) -> Result<()> {
        let mut counters = safe_mutex_lock(&self.counters)?;
        let counter = counters.entry(name.to_string()).or_insert(0.0);
        *counter += value;
        Ok(())
    }

    pub fn set_gauge(&self, name: &str, value: f64) -> Result<()> {
        let mut gauges = safe_mutex_lock(&self.gauges)?;
        gauges.insert(name.to_string(), value);
        Ok(())
    }

    pub fn record_histogram(&self, name: &str, value: f64) -> Result<()> {
        let mut histograms = safe_mutex_lock(&self.histograms)?;
        histograms.entry(name.to_string()).or_insert_with(Vec::new).push(value);
        Ok(())
    }

    pub fn record_timer(&self, name: &str, duration: Duration) -> Result<()> {
        let mut timers = safe_mutex_lock(&self.timers)?;
        timers.entry(name.to_string()).or_insert_with(Vec::new).push(duration);
        Ok(())
    }

    fn calculate_percentile(values: &[f64], percentile: f64) -> f64 {
        if values.is_empty() {
            return 0.0;
        }

        let mut sorted_values = values.to_vec();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let index = ((values.len() as f64 - 1.0) * percentile / 100.0) as usize;
        sorted_values[index]
    }

}

impl Default for InMemoryMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsCollector for InMemoryMetricsCollector {
    fn record(&mut self, metric: Metric) {
        if let Err(e) = safe_mutex_lock(&self.metrics).and_then(|mut metrics| {
            metrics.entry(metric.name.clone()).or_insert_with(Vec::new).push(metric);
            Ok(())
        }) {
            eprintln!("Failed to record metric: {}", e);
        }
    }

    fn get_metrics(&self, name: &str) -> Vec<Metric> {
        safe_mutex_lock(&self.metrics)
            .map(|m| m.get(name).cloned().unwrap_or_default())
            .unwrap_or_default()
    }

    fn get_performance_metrics(&self) -> PerformanceMetrics {
        match self.get_performance_metrics_internal() {
            Ok(metrics) => metrics,
            Err(e) => {
                eprintln!("Failed to get performance metrics: {}", e);
                PerformanceMetrics {
                    requests_per_second: 0.0,
                    average_response_time: 0.0,
                    success_rate: 0.0,
                    error_rate: 0.0,
                    memory_usage_mb: 0.0,
                    cpu_usage_percent: 0.0,
                    active_connections: 0,
                    queue_size: 0,
                    timestamp: chrono::Utc::now(),
                }
            }
        }
    }

    fn reset(&mut self) {
        if let Err(e) = self.reset_internal() {
            eprintln!("Failed to reset metrics: {}", e);
        }
    }
}

pub struct PrometheusMetricsCollector {
    registry: HashMap<String, MetricFamily>,
    start_time: Instant,
    system: Arc<Mutex<System>>,
}

impl PrometheusMetricsCollector {
    pub fn new() -> Self {
        Self {
            registry: HashMap::new(),
            start_time: Instant::now(),
            system: Arc::new(Mutex::new(System::new_all())),
        }
    }

    pub fn export_prometheus_format(&self) -> String {
        let mut output = String::new();
        
        for metric_family in self.registry.values() {
            output.push_str(&metric_family.to_prometheus());
            output.push('\n');
        }
        
        output
    }

    /// Get current memory usage in MB from real system monitoring
    pub fn get_memory_usage(&self) -> f64 {
        match safe_mutex_lock(&self.system) {
            Ok(mut system) => {
                system.refresh_all();
                
                let total_memory = system.total_memory();
                let used_memory = system.used_memory();
                
                if total_memory > 0 {
                    (used_memory as f64) / (1024.0 * 1024.0) // Convert to MB
                } else {
                    0.0
                }
            }
            Err(e) => {
                eprintln!("Failed to get memory usage: {}", e);
                0.0
            }
        }
    }

    /// Get current CPU usage percentage from real system monitoring
    pub fn get_cpu_usage(&self) -> f64 {
        match safe_mutex_lock(&self.system) {
            Ok(mut system) => {
                system.refresh_cpu();
                
                let cpus = system.cpus();
                if cpus.is_empty() {
                    0.0
                } else {
                    let total_usage: f32 = cpus.iter().map(|cpu| cpu.cpu_usage()).sum();
                    (total_usage as f64) / cpus.len() as f64
                }
            }
            Err(e) => {
                eprintln!("Failed to get CPU usage: {}", e);
                0.0
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct MetricFamily {
    pub name: String,
    pub help: String,
    pub metric_type: MetricType,
    pub metrics: Vec<Metric>,
}

impl MetricFamily {
    pub fn to_prometheus(&self) -> String {
        let mut output = String::new();
        
        output.push_str("# HELP ");
        output.push_str(&self.name);
        output.push(' ');
        output.push_str(&self.help);
        output.push('\n');

        let type_str = match self.metric_type {
            MetricType::Counter => "counter",
            MetricType::Gauge => "gauge",
            MetricType::Histogram => "histogram",
            MetricType::Timer => "timer",
        };

        output.push_str("# TYPE ");
        output.push_str(&self.name);
        output.push(' ');
        output.push_str(type_str);
        output.push('\n');

        for metric in &self.metrics {
            let mut labels = String::new();
            if !metric.tags.is_empty() {
                let tag_strings: Vec<String> = metric.tags
                    .iter()
                    .map(|(k, v)| format!("{}=\"{}\"", k, v))
                    .collect();
                labels = format!("{{{}}}", tag_strings.join(","));
            }

            output.push_str(&self.name);
            output.push_str(&labels);
            output.push(' ');
            output.push_str(&metric.value.to_string());
            output.push(' ');
            output.push_str(&metric.timestamp.timestamp_millis().to_string());
            output.push('\n');
        }

        output
    }
}

impl MetricsCollector for PrometheusMetricsCollector {
    fn record(&mut self, metric: Metric) {
        let family = self.registry.entry(metric.name.clone()).or_insert_with(|| MetricFamily {
            name: metric.name.clone(),
            help: "CyberSpider metric".to_string(),
            metric_type: metric.metric_type.clone(),
            metrics: Vec::new(),
        });

        family.metrics.push(metric);
    }

    fn get_metrics(&self, name: &str) -> Vec<Metric> {
        self.registry
            .get(name)
            .map(|family| family.metrics.clone())
            .unwrap_or_default()
    }

    fn get_performance_metrics(&self) -> PerformanceMetrics {
        // Create default metric families to avoid temporary value issues
        let default_total_requests = MetricFamily {
            name: "total_requests".to_string(),
            help: "Total requests".to_string(),
            metric_type: MetricType::Counter,
            metrics: Vec::new(),
        };
        let default_total_response_time = MetricFamily {
            name: "total_response_time".to_string(),
            help: "Total response time".to_string(),
            metric_type: MetricType::Counter,
            metrics: Vec::new(),
        };
        let default_error_count = MetricFamily {
            name: "error_count".to_string(),
            help: "Error count".to_string(),
            metric_type: MetricType::Counter,
            metrics: Vec::new(),
        };
        
        let total_requests = self.registry.get("total_requests").unwrap_or(&default_total_requests);
        let total_response_time = self.registry.get("total_response_time").unwrap_or(&default_total_response_time);
        let error_count = self.registry.get("error_count").unwrap_or(&default_error_count);
        
        // Calculate requests per second based on recent activity
        let requests_per_second = {
            let elapsed = self.start_time.elapsed();
            if elapsed.as_secs() > 0 {
                total_requests.metrics.len() as f64 / elapsed.as_secs() as f64
            } else {
                0.0
            }
        };
        
        // Calculate average response time
        let average_response_time = if !total_requests.metrics.is_empty() {
            total_response_time.metrics.iter().map(|m| m.value).sum::<f64>() / total_requests.metrics.len() as f64
        } else {
            0.0
        };
        
        // Calculate error rate
        let error_rate = if !total_requests.metrics.is_empty() {
            (error_count.metrics.len() as f64 / total_requests.metrics.len() as f64) * 100.0
        } else {
            0.0
        };
        
        // Calculate success rate
        let success_rate = if !total_requests.metrics.is_empty() {
            ((total_requests.metrics.len() as f64 - error_count.metrics.len() as f64) / total_requests.metrics.len() as f64) * 100.0
        } else {
            100.0
        };
        
        // Get real system metrics
        let memory_usage = self.get_memory_usage();
        let cpu_usage = self.get_cpu_usage();
        
        PerformanceMetrics {
            requests_per_second,
            average_response_time,
            success_rate,
            error_rate,
            memory_usage_mb: memory_usage,
            cpu_usage_percent: cpu_usage,
            active_connections: self.registry.get("active_connections").unwrap_or(&MetricFamily {
                name: "active_connections".to_string(),
                help: "Active connections".to_string(),
                metric_type: MetricType::Gauge,
                metrics: Vec::new(),
            }).metrics.len(),
            queue_size: self.registry.get("queue_size").unwrap_or(&MetricFamily {
                name: "queue_size".to_string(),
                help: "Queue size".to_string(),
                metric_type: MetricType::Gauge,
                metrics: Vec::new(),
            }).metrics.len(),
            timestamp: Utc::now(),
        }
    }

    fn reset(&mut self) {
        self.registry.clear();
    }
}

pub struct MetricsTimer {
    start_time: Instant,
    collector: Arc<Mutex<InMemoryMetricsCollector>>,
    metric_name: String,
    tags: HashMap<String, String>,
}

impl MetricsTimer {
    pub fn new(
        collector: Arc<Mutex<InMemoryMetricsCollector>>,
        metric_name: String,
        tags: HashMap<String, String>,
    ) -> Self {
        Self {
            start_time: Instant::now(),
            collector,
            metric_name,
            tags,
        }
    }
}

impl Drop for MetricsTimer {
    fn drop(&mut self) {
        let duration = self.start_time.elapsed();
        
        if let Ok(collector) = safe_mutex_lock(&self.collector) {
            // Record timer with tags for advanced metrics tracking
            if let Err(e) = collector.record_timer(&self.metric_name, duration) {
                eprintln!("Failed to record timer metric: {}", e);
            }
            
            // Also record tags as gauge metrics for advanced analysis
            for (key, value) in &self.tags {
                let tagged_metric_name = format!("{}_tag_{}", self.metric_name, key);
                if let Ok(numeric_value) = value.parse::<f64>() {
                    if let Err(e) = collector.set_gauge(&tagged_metric_name, numeric_value) {
                        eprintln!("Failed to record gauge metric: {}", e);
                    }
                }
            }
        } else {
            eprintln!("Failed to lock collector in MetricsTimer::drop");
        }
    }
}
