use anyhow::Result;
use regex::Regex;
use crate::security::{SecurityDetector, SecurityFinding, Severity};

pub struct FormDetector {
    form_regex: Regex,
    input_regex: Regex,
    action_regex: Regex,
}

impl FormDetector {
    pub fn new() -> Self {
        let form_regex = Regex::new(r"(?i)<form[^>]*>(.*?)</form>").unwrap();
        let input_regex = Regex::new(r"(?i)<input[^>]*>").unwrap();
        let action_regex = Regex::new(r#"(?i)action\s*=\s*["']([^"']+)["']"#).unwrap();

        Self {
            form_regex,
            input_regex,
            action_regex,
        }
    }

    fn extract_forms(&self, content: &str, base_url: &str) -> Vec<SecurityFinding> {
        let mut findings = Vec::new();
        
        for form_match in self.form_regex.find_iter(content) {
            let form_content = form_match.as_str();
            
            let action = self.action_regex.captures(form_content)
                .and_then(|caps| caps.get(1))
                .map(|m| m.as_str())
                .unwrap_or("unknown");

            let input_count = self.input_regex.find_iter(form_content).count();
            
            let severity = if action.contains("login") || action.contains("auth") {
                Severity::High
            } else if action.contains("search") || action.contains("contact") {
                Severity::Medium
            } else {
                Severity::Low
            };

            findings.push(SecurityFinding {
                finding_type: "Form".to_string(),
                severity,
                description: format!("Form with {} inputs targeting: {}", input_count, action),
                url: base_url.to_string(),
                evidence: format!("Form action: {}", action),
                recommendation: Some("Test for XSS, SQL injection, and CSRF vulnerabilities".to_string()),
            });

            if input_count > 10 {
                findings.push(SecurityFinding {
                    finding_type: "Form".to_string(),
                    severity: Severity::Medium,
                    description: "Form with excessive number of inputs detected".to_string(),
                    url: base_url.to_string(),
                    evidence: format!("Input count: {}", input_count),
                    recommendation: Some("Review form complexity and potential for data harvesting".to_string()),
                });
            }
        }

        findings
    }
}

#[async_trait::async_trait]
impl SecurityDetector for FormDetector {
    async fn detect(&self, content: &str, base_url: &str) -> Result<Vec<SecurityFinding>> {
        Ok(self.extract_forms(content, base_url))
    }

    fn detector_name(&self) -> &'static str {
        "form_detector"
    }
}
