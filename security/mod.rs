pub mod api_detector;
pub mod form_detector;
pub mod tech_detector;
pub mod vuln_scanner;

use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait SecurityDetector {
    async fn detect(&self, content: &str, base_url: &str) -> Result<Vec<SecurityFinding>>;
    fn detector_name(&self) -> &'static str;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SecurityFinding {
    pub finding_type: String,
    pub severity: Severity,
    pub description: String,
    pub url: String,
    pub evidence: String,
    pub recommendation: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}
