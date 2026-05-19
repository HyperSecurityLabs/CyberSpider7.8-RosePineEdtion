use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlSession {
    pub id: String,
    pub base_domain: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub status: SessionStatus,
    pub total_requests: usize,
    pub successful_requests: usize,
    pub failed_requests: usize,
    pub urls_discovered: usize,
    pub subdomains_found: usize,
    pub s3_buckets_found: usize,
    pub security_findings: usize,
    pub config: CrawlConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlConfig {
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlNode {
    pub url: String,
    pub parent_url: Option<String>,
    pub depth: usize,
    pub status: UrlStatus,
    pub discovered_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UrlStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityFinding {
    pub id: String,
    pub session_id: String,
    pub finding_type: String,
    pub severity: String,
    pub description: String,
    pub url: String,
    pub evidence: String,
    pub recommendation: Option<String>,
    pub discovered_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subdomain {
    pub id: String,
    pub session_id: String,
    pub subdomain: String,
    pub base_domain: String,
    pub source: String,
    pub discovered_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Bucket {
    pub id: String,
    pub session_id: String,
    pub bucket_url: String,
    pub base_domain: String,
    pub source: String,
    pub verified: bool,
    pub discovered_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlMetrics {
    pub session_id: String,
    pub timestamp: DateTime<Utc>,
    pub requests_per_second: f64,
    pub average_response_time: f64,
    pub success_rate: f64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
}

impl CrawlSession {
    pub fn new(id: String, base_domain: String, config: CrawlConfig) -> Self {
        Self {
            id,
            base_domain,
            started_at: Utc::now(),
            ended_at: None,
            status: SessionStatus::Running,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            urls_discovered: 0,
            subdomains_found: 0,
            s3_buckets_found: 0,
            security_findings: 0,
            config,
        }
    }

    pub fn complete(&mut self) {
        self.status = SessionStatus::Completed;
        self.ended_at = Some(Utc::now());
    }

    pub fn fail(&mut self) {
        self.status = SessionStatus::Failed;
        self.ended_at = Some(Utc::now());
    }

    pub fn cancel(&mut self) {
        self.status = SessionStatus::Cancelled;
        self.ended_at = Some(Utc::now());
    }

    pub fn duration(&self) -> Option<chrono::Duration> {
        match self.ended_at {
            Some(end) => Some(end - self.started_at),
            None => Some(Utc::now() - self.started_at),
        }
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.successful_requests as f64 / self.total_requests as f64 * 100.0
        }
    }
}

impl UrlNode {
    pub fn new(url: String, parent_url: Option<String>, depth: usize) -> Self {
        Self {
            url,
            parent_url,
            depth,
            status: UrlStatus::Pending,
            discovered_at: Utc::now(),
            processed_at: None,
            error_message: None,
        }
    }

    pub fn mark_processing(&mut self) {
        self.status = UrlStatus::Processing;
    }

    pub fn mark_completed(&mut self) {
        self.status = UrlStatus::Completed;
        self.processed_at = Some(Utc::now());
    }

    pub fn mark_failed(&mut self, error: String) {
        self.status = UrlStatus::Failed;
        self.processed_at = Some(Utc::now());
        self.error_message = Some(error);
    }

    pub fn mark_skipped(&mut self) {
        self.status = UrlStatus::Skipped;
        self.processed_at = Some(Utc::now());
    }
}
