use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use crate::plugins::{Plugin, PluginInfo, PluginType, PluginConfig, PluginContext, PluginResult, PluginInfoFFI};

#[derive(Debug, Clone)]
pub struct ExampleDetectorPlugin {
    info: PluginInfo,
    initialized: bool,
}

unsafe impl Send for ExampleDetectorPlugin {}
unsafe impl Sync for ExampleDetectorPlugin {}

impl ExampleDetectorPlugin {
    pub fn new() -> Self {
        Self {
            info: PluginInfo {
                name: "example_detector".to_string(),
                version: "1.0.0".to_string(),
                description: "Example detector plugin that finds email addresses".to_string(),
                author: "CyberSpider Team".to_string(),
                plugin_type: PluginType::Detector,
                dependencies: vec![],
                permissions: vec!["read_content".to_string()],
            },
            initialized: false,
        }
    }

    fn extract_emails(&self, content: &str) -> Vec<String> {
        let email_regex = regex::Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap();
        
        email_regex
            .find_iter(content)
            .map(|m| m.as_str().to_string())
            .collect()
    }
}

#[async_trait]
impl Plugin for ExampleDetectorPlugin {
    fn plugin_info(&self) -> PluginInfo {
        self.info.clone()
    }

    async fn initialize(&mut self, _config: &PluginConfig) -> Result<()> {
        self.initialized = true;
        Ok(())
    }

    async fn execute(&mut self, context: &PluginContext) -> Result<PluginResult> {
        if !self.initialized {
            return Ok(PluginResult {
                success: false,
                data: None,
                error: Some("Plugin not initialized".to_string()),
                metadata: std::collections::HashMap::new(),
            });
        }

        let content = context.content.as_deref().unwrap_or("");
        let emails = self.extract_emails(content);

        let mut result_data = serde_json::Map::new();
        result_data.insert("emails".to_string(), Value::Array(
            emails.into_iter().map(Value::String).collect()
        ));
        result_data.insert("count".to_string(), Value::Number(
            serde_json::Number::from(result_data["emails"].as_array().unwrap().len())
        ));

        Ok(PluginResult {
            success: true,
            data: Some(Value::Object(result_data)),
            error: None,
            metadata: {
                let mut meta = std::collections::HashMap::new();
                meta.insert("detector_type".to_string(), "email".to_string());
                meta
            },
        })
    }

    async fn cleanup(&mut self) -> Result<()> {
        self.initialized = false;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ExampleProcessorPlugin {
    info: PluginInfo,
    initialized: bool,
}

unsafe impl Send for ExampleProcessorPlugin {}
unsafe impl Sync for ExampleProcessorPlugin {}

impl ExampleProcessorPlugin {
    pub fn new() -> Self {
        Self {
            info: PluginInfo {
                name: "example_processor".to_string(),
                version: "1.0.0".to_string(),
                description: "Example processor plugin that normalizes URLs".to_string(),
                author: "CyberSpider Team".to_string(),
                plugin_type: PluginType::Processor,
                dependencies: vec![],
                permissions: vec!["modify_content".to_string()],
            },
            initialized: false,
        }
    }

    fn normalize_url(&self, url: &str) -> String {
        if let Ok(mut parsed) = url::Url::parse(url) {
            // Remove fragment for cleaner URL
            parsed.set_fragment(None);
            
            // Filter out tracking parameters and common noise parameters
            let mut filtered_params = Vec::new();
            for (key, value) in parsed.query_pairs() {
                let key_str = key.to_lowercase();
                // Keep parameters that might be useful for security analysis
                let should_keep = !matches!(
                    key_str.as_str(),
                    "utm_source" | "utm_medium" | "utm_campaign" | "utm_term" | "utm_content" |
                    "fbclid" | "gclid" | "msclkid" | "_ga" | "_gid" | "mcid" |
                    "ref" | "source" | "campaign" | "click_id"
                );
                
                if should_keep {
                    filtered_params.push((key.to_string(), value.to_string()));
                }
            }
            
            // Rebuild query string with filtered parameters
            if !filtered_params.is_empty() {
                let query_string = filtered_params
                    .iter()
                    .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
                    .collect::<Vec<_>>()
                    .join("&");
                parsed.set_query(Some(&query_string));
            } else {
                parsed.set_query(None);
            }
            
            parsed.to_string()
        } else {
            url.to_string()
        }
    }
}

#[async_trait]
impl Plugin for ExampleProcessorPlugin {
    fn plugin_info(&self) -> PluginInfo {
        self.info.clone()
    }

    async fn initialize(&mut self, _config: &PluginConfig) -> Result<()> {
        self.initialized = true;
        Ok(())
    }

    async fn execute(&mut self, context: &PluginContext) -> Result<PluginResult> {
        if !self.initialized {
            return Ok(PluginResult {
                success: false,
                data: None,
                error: Some("Plugin not initialized".to_string()),
                metadata: std::collections::HashMap::new(),
            });
        }

        let url = context.url.as_deref().unwrap_or("");
        let normalized_url = self.normalize_url(url);

        // Red team: Perform security analysis on the URL
        let mut security_findings = Vec::new();
        let mut risk_score = 0.0;

        // Check for potential security indicators
        if let Ok(parsed) = url::Url::parse(url) {
            // Analyze domain for suspicious patterns
            let domain = parsed.host_str().unwrap_or("");
            
            // Check for suspicious TLDs
            let suspicious_tlds = [".tk", ".ml", ".ga", ".cf", ".pw", ".top", ".click"];
            if suspicious_tlds.iter().any(|tld| domain.ends_with(tld)) {
                security_findings.push("Suspicious TLD detected".to_string());
                risk_score += 20.0;
            }

            // Check for URL shorteners
            let shorteners = ["bit.ly", "tinyurl.com", "t.co", "goo.gl", "ow.ly"];
            if shorteners.iter().any(|short| domain.contains(short)) {
                security_findings.push("URL shortener detected - may hide real destination".to_string());
                risk_score += 15.0;
            }

            // Check for suspicious parameters
            for (key, value) in parsed.query_pairs() {
                let key_lower = key.to_lowercase();
                match key_lower.as_str() {
                    "redirect" | "url" | "return" | "callback" | "next" => {
                        security_findings.push(format!("Potential redirect parameter: {}={}", key, value));
                        risk_score += 10.0;
                    }
                    "token" | "key" | "secret" | "password" | "pass" => {
                        security_findings.push(format!("Sensitive parameter detected: {}", key));
                        risk_score += 25.0;
                    }
                    "id" | "user" | "email" | "username" => {
                        security_findings.push(format!("User-related parameter: {}", key));
                        risk_score += 5.0;
                    }
                    _ => {}
                }
            }

            // Check path for interesting patterns
            let path = parsed.path();
            if path.contains("/admin") || path.contains("/wp-admin") || path.contains("/phpmyadmin") {
                security_findings.push("Admin interface detected".to_string());
                risk_score += 30.0;
            }
            
            if path.contains("/api/") || path.contains("/v1/") || path.contains("/rest/") {
                security_findings.push("API endpoint detected".to_string());
                risk_score += 10.0;
            }
        }

        let mut result_data = serde_json::Map::new();
        result_data.insert("original_url".to_string(), Value::String(url.to_string()));
        result_data.insert("normalized_url".to_string(), Value::String(normalized_url));
        result_data.insert("security_findings".to_string(), Value::Array(
            security_findings.into_iter().map(Value::String).collect()
        ));
        result_data.insert("risk_score".to_string(), Value::Number(serde_json::Number::from_f64(risk_score).unwrap()));
        result_data.insert("risk_level".to_string(), Value::String(
            if risk_score >= 50.0 { "HIGH".to_string() }
            else if risk_score >= 25.0 { "MEDIUM".to_string() }
            else if risk_score > 0.0 { "LOW".to_string() }
            else { "NONE".to_string() }
        ));

        let mut metadata = std::collections::HashMap::new();
        metadata.insert("processed_at".to_string(), chrono::Utc::now().to_rfc3339());
        metadata.insert("analysis_type".to_string(), "security_recon".to_string());

        Ok(PluginResult {
            success: true,
            data: Some(Value::Object(result_data)),
            error: None,
            metadata,
        })
    }

    async fn cleanup(&mut self) -> Result<()> {
        self.initialized = false;
        Ok(())
    }
}

// Opaque pointer type for FFI safety
#[repr(C)]
pub struct OpaquePlugin {
    _private: [u8; 0],
}

#[no_mangle]
pub extern "C" fn create_detector_plugin() -> *mut OpaquePlugin {
    let plugin = ExampleDetectorPlugin::new();
    let boxed: Box<dyn Plugin> = Box::new(plugin);
    Box::into_raw(boxed) as *mut OpaquePlugin
}

#[no_mangle]
pub extern "C" fn create_processor_plugin() -> *mut OpaquePlugin {
    let plugin = ExampleProcessorPlugin::new();
    let boxed: Box<dyn Plugin> = Box::new(plugin);
    Box::into_raw(boxed) as *mut OpaquePlugin
}

#[no_mangle]
pub extern "C" fn get_detector_plugin_info() -> PluginInfoFFI {
    let info = ExampleDetectorPlugin::new().plugin_info();
    PluginInfoFFI::from_info(&info)
}

#[no_mangle]
pub extern "C" fn get_processor_plugin_info() -> PluginInfoFFI {
    let info = ExampleProcessorPlugin::new().plugin_info();
    PluginInfoFFI::from_info(&info)
}
