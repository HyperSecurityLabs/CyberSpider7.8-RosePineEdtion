use anyhow::Result;
use regex::Regex;
use crate::security::{SecurityDetector, SecurityFinding, Severity};

pub struct VulnScanner {
    vuln_patterns: Vec<(Regex, String, Severity, String)>,
}

impl VulnScanner {
    pub fn new() -> Self {
        let vuln_patterns = vec![
            (Regex::new(r#"password\s*=\s*["'][^"']+["']"#).unwrap(), "Hardcoded Password".to_string(), Severity::Critical, "Remove hardcoded credentials".to_string()),
            (Regex::new(r#"api[_-]?key\s*=\s*["'][^"']+["']"#).unwrap(), "Hardcoded API Key".to_string(), Severity::Critical, "Use environment variables for APIkeys".to_string()),
            (Regex::new(r#"secret[_-]?key\s*=\s*["'][^"']+["']"#).unwrap(), "Hardcoded Secret".to_string(), Severity::Critical, "Store secrets securely".to_string()),
            (Regex::new(r"mysql_connect\(|mysqli_connect\(").unwrap(), "Database Connection".to_string(), Severity::Medium, "Ensure proper database security".to_string()),
            (Regex::new(r"eval\s*\(|exec\s*\(").unwrap(), "Code Execution".to_string(), Severity::High, "Avoid eval/exec functions".to_string()),
            (Regex::new(r"system\s*\(|shell_exec\s*\(").unwrap(), "System Command".to_string(), Severity::High, "Validate and sanitize user input".to_string()),
            (Regex::new(r"document\.write\s*\(|innerHTML\s*=").unwrap(), "XSS Risk".to_string(), Severity::Medium, "Use safe DOM manipulation methods".to_string()),
            (Regex::new(r"debug.*true|DEBUG.*true").unwrap(), "Debug Mode".to_string(), Severity::Medium, "Disable debug in production".to_string()),
            (Regex::new(r"allow_url_include.*On|allow_url_fopen.*On").unwrap(), "File Inclusion".to_string(), Severity::High, "Disable URL file inclusion".to_string()),
            (Regex::new(r"Error reporting.*E_ALL|display_errors.*On").unwrap(), "Error Disclosure".to_string(), Severity::Low, "Restrict error reporting inproduction".to_string()),
            (Regex::new(r"<script[^>]*>.*alert\s*\(").unwrap(), "JavaScript Alert".to_string(), Severity::Low, "Remove debug alerts".to_string()),
            (Regex::new(r"console\.log|console\.debug").unwrap(), "Console Logging".to_string(), Severity::Info, "Remove console logs in production".to_string()),
        ];

        Self { vuln_patterns }
    }

    fn scan_vulnerabilities(&self, content: &str, base_url: &str) -> Vec<SecurityFinding> {
        let mut findings = Vec::new();
        
        for (regex, vuln_type, severity, recommendation) in &self.vuln_patterns {
            for match_result in regex.find_iter(content) {
                let matched_text = match_result.as_str();
                
                findings.push(SecurityFinding {
                    finding_type: vuln_type.clone(),
                    severity: severity.clone(),
                         description: format!("Potential {} detected", vuln_type),
                    url: base_url.to_string(),
                     evidence: matched_text.to_string(),
                     recommendation: Some(recommendation.clone()),
                });
            }
        }

        findings
    }
}

#[async_trait::async_trait]
impl SecurityDetector for VulnScanner {
    async fn detect(&self, content: &str, base_url: &str) -> Result<Vec<SecurityFinding>> {
        Ok(self.scan_vulnerabilities(content, base_url))
    }

    fn detector_name(&self) -> &'static str {
        "vuln_scanner"
    }
}
