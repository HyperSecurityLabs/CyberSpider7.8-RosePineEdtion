use anyhow::Result;
use regex::Regex;
use crate::security::{SecurityDetector, SecurityFinding, Severity};

pub struct ApiDetector {
    api_regex: Regex,
    endpoint_regex: Regex,
}

impl ApiDetector {
    pub fn new() -> Self {
        let api_regex = Regex::new(r"/api/v?[0-9]?/|/rest/|/graphql|/ws/|/webhook|/oauth|/auth|/token").unwrap();
        let endpoint_regex = Regex::new(r"/(users|posts|comments|data|admin|config|files|upload|download|search|login|register|logout|profile|settings)/?[a-zA-Z0-9/_-]*").unwrap();

        Self {
            api_regex,
            endpoint_regex,
        }
    }

    fn extract_api_info(&self, content: &str, base_url: &str) -> Vec<SecurityFinding> {
        let mut findings = Vec::new();
        
        for api_match in self.api_regex.find_iter(content) {
            let api_path = api_match.as_str();
            findings.push(SecurityFinding {
                finding_type: "API Endpoint".to_string(),
                severity: Severity::Medium,
                description: format!("Discovered API endpoint: {}", api_path),
                url: base_url.to_string(),
                evidence: api_path.to_string(),
                recommendation: Some("Test for authentication bypass and data exposure".to_string()),
            });
        }

        for endpoint_match in self.endpoint_regex.find_iter(content) {
            let endpoint = endpoint_match.as_str();
            let severity = if endpoint.contains("/admin") || endpoint.contains("/config") {
                Severity::High
            } else if endpoint.contains("/users") || endpoint.contains("/profile") {
                Severity::Medium
            } else {
                Severity::Low
            };

            findings.push(SecurityFinding {
                finding_type: "API Endpoint".to_string(),
                severity,
                description: format!("Discovered sensitive API endpoint: {}", endpoint),
                url: base_url.to_string(),
                evidence: endpoint.to_string(),
                recommendation: Some("Test for proper access controls and data validation".to_string()),
            });
        }

        findings
    }
}

#[async_trait::async_trait]
impl SecurityDetector for ApiDetector {
    async fn detect(&self, content: &str, base_url: &str) -> Result<Vec<SecurityFinding>> {
        Ok(self.extract_api_info(content, base_url))
    }

    fn detector_name(&self) -> &'static str {
        "api_detector"
    }
}
