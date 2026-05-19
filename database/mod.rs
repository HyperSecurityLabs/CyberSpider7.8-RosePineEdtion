pub mod sqlite;
pub mod redis;
pub mod models;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

#[async_trait]
pub trait Database {
    async fn save_url(&mut self, url: &UrlRecord) -> Result<()>;
    async fn get_urls(&self, limit: Option<usize>) -> Result<Vec<UrlRecord>>;
    async fn save_subdomain(&mut self, subdomain: &SubdomainRecord) -> Result<()>;
    async fn get_subdomains(&self, domain: &str) -> Result<Vec<SubdomainRecord>>;
    async fn save_s3_bucket(&mut self, bucket: &S3BucketRecord) -> Result<()>;
    async fn get_s3_buckets(&self) -> Result<Vec<S3BucketRecord>>;
    async fn save_security_finding(&mut self, finding: &SecurityFindingRecord) -> Result<()>;
    async fn get_security_findings(&self, severity: Option<&str>) -> Result<Vec<SecurityFindingRecord>>;
    async fn get_stats(&self) -> Result<DatabaseStats>;
    async fn cleanup_old_records(&mut self, days: u32) -> Result<usize>;
}

#[async_trait]
pub trait Queue {
    async fn push_url(&mut self, url: &str) -> Result<()>;
    async fn pop_url(&mut self) -> Result<Option<String>>;
    async fn get_queue_size(&self) -> Result<usize>;
    async fn clear_queue(&mut self) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlRecord {
    pub id: Option<i64>,
    pub url: String,
    pub base_domain: String,
    pub source: String,
    pub status_code: Option<u16>,
    pub content_type: Option<String>,
    pub title: Option<String>,
    pub method: String,
    pub discovered_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubdomainRecord {
    pub id: Option<i64>,
    pub subdomain: String,
    pub base_domain: String,
    pub source: String,
    pub discovered_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3BucketRecord {
    pub id: Option<i64>,
    pub bucket_url: String,
    pub base_domain: String,
    pub source: String,
    pub verified: bool,
    pub discovered_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityFindingRecord {
    pub id: Option<i64>,
    pub finding_type: String,
    pub severity: String,
    pub description: String,
    pub url: String,
    pub evidence: String,
    pub recommendation: Option<String>,
    pub discovered_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub total_urls: i64,
    pub total_subdomains: i64,
    pub total_s3_buckets: i64,
    pub total_security_findings: i64,
    pub unique_domains: i64,
    pub oldest_record: Option<chrono::DateTime<chrono::Utc>>,
    pub newest_record: Option<chrono::DateTime<chrono::Utc>>,
}

pub struct DatabaseConfig {
    pub sqlite_path: Option<String>,
    pub redis_url: Option<String>,
    pub pool_size: u32,
    pub connection_timeout: u64,
}
