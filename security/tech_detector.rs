use anyhow::Result;
use regex::Regex;
use crate::security::{SecurityDetector, SecurityFinding, Severity};

pub struct TechDetector {
    tech_patterns: Vec<(Regex, String, Severity)>,
}

impl TechDetector {
    pub fn new() -> Self {
        let tech_patterns = vec![
            (Regex::new(r"WordPress\s+([\d.]+)").unwrap(), "WordPress".to_string(), Severity::Medium),
            (Regex::new(r"Joomla!").unwrap(), "Joomla".to_string(), Severity::Medium),
            (Regex::new(r"Drupal\s+([\d.]+)").unwrap(), "Drupal".to_string(), Severity::Medium),
            (Regex::new(r"Magento").unwrap(), "Magento".to_string(), Severity::Medium),
            (Regex::new(r"Shopify").unwrap(), "Shopify".to_string(), Severity::Low),
            (Regex::new(r#"React\.version\s*=\s*["']([^"']+)["']"#).unwrap(), "React".to_string(), Severity::Low),
            (Regex::new(r#"angular\.version\s*=\s*["']([^"']+)["']"#).unwrap(), "Angular".to_string(), Severity::Low),
            (Regex::new(r"jQuery\s*([^\s;]+)").unwrap(), "jQuery".to_string(), Severity::Low),
            (Regex::new(r"Bootstrap\s+([^\s;]+)").unwrap(), "Bootstrap".to_string(), Severity::Low),
            (Regex::new(r"Laravel").unwrap(), "Laravel".to_string(), Severity::Medium),
            (Regex::new(r"Django").unwrap(), "Django".to_string(), Severity::Medium),
            (Regex::new(r"Flask").unwrap(), "Flask".to_string(), Severity::Medium),
            (Regex::new(r"Express\.js").unwrap(), "Express.js".to_string(), Severity::Medium),
            (Regex::new(r"Spring\s+Framework").unwrap(), "Spring Framework".to_string(), Severity::Medium),
            (Regex::new(r"Apache\s+([^\s;]+)").unwrap(), "Apache".to_string(), Severity::Low),
            (Regex::new(r"nginx").unwrap(), "Nginx".to_string(), Severity::Low),
            (Regex::new(r"IIS").unwrap(), "IIS".to_string(), Severity::Low),
        ];

        Self { tech_patterns }
    }

    fn extract_technologies(&self, content: &str, base_url: &str) -> Vec<SecurityFinding> {
        let mut findings = Vec::new();
        
        for (regex, tech_name, severity) in &self.tech_patterns {
            if let Some(captures) = regex.captures(content) {
                let version = captures.get(1).map(|m| m.as_str()).unwrap_or("unknown");
                
                findings.push(SecurityFinding {
                    finding_type: "Technology".to_string(),
                    severity: severity.clone(),
                    description: format!("Detected technology: {} {}", tech_name, if version != "unknown" { format!("v{}", version) } else { String::new() }),
                    url: base_url.to_string(),
                    evidence: format!("{}: {}", tech_name, version),
                    recommendation: Some("Check for known vulnerabilities in this version".to_string()),
                });
            }
        }

        findings
    }
}

#[async_trait::async_trait]
impl SecurityDetector for TechDetector {
    async fn detect(&self, content: &str, base_url: &str) -> Result<Vec<SecurityFinding>> {
        Ok(self.extract_technologies(content, base_url))
    }

    fn detector_name(&self) -> &'static str {
        "tech_detector"
    }
}
