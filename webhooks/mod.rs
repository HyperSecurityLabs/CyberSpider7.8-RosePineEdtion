use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEvent {
    pub event_type: String,
    pub timestamp: DateTime<Utc>,
    pub session_id: String,
    pub data: serde_json::Value,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub url: String,
    pub timeout_seconds: u64,
    pub retry_attempts: u32,
    pub retry_delay_seconds: u64,
    pub secret: Option<String>,
    pub headers: HashMap<String, String>,
    pub enabled_events: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookResult {
    pub success: bool,
    pub status_code: Option<u16>,
    pub response_body: Option<String>,
    pub error: Option<String>,
    pub attempt: u32,
    pub timestamp: DateTime<Utc>,
}

#[async_trait]
pub trait WebhookSender {
    async fn send_event(&mut self, event: WebhookEvent) -> Result<WebhookResult>;
    async fn send_batch(&mut self, events: Vec<WebhookEvent>) -> Result<Vec<WebhookResult>>;
    async fn test_connection(&self) -> Result<bool>;
    fn get_config(&self) -> &WebhookConfig;
}

pub struct HttpWebhookSender {
    client: Client,
    config: WebhookConfig,
}

impl HttpWebhookSender {
    pub fn new(config: WebhookConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()?;

        Ok(Self { client, config })
    }

    fn create_signature(&self, payload: &str) -> String {
        if let Some(secret) = &self.config.secret {
            use hmac::{Hmac, Mac};
            use sha2::Sha256;
            
            type HmacSha256 = Hmac<Sha256>;
            
            let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
            mac.update(payload.as_bytes());
            let result = mac.finalize();
            let code_bytes = result.into_bytes();
            
            format!("sha256={}", hex::encode(code_bytes))
        } else {
            String::new()
        }
    }

    async fn send_with_retry(&mut self, event: &WebhookEvent) -> WebhookResult {
        let mut last_result: Option<WebhookResult> = None;
        
        for attempt in 1..=self.config.retry_attempts {
            let result = self.send_single(event, attempt).await;
            
            match &result {
                WebhookResult { success: true, .. } => return result,
                WebhookResult { success: false, .. } => {
                    last_result = Some(result);
                    if attempt < self.config.retry_attempts {
                        tokio::time::sleep(std::time::Duration::from_secs(self.config.retry_delay_seconds)).await;
                    }
                }
            }
        }
        
        last_result.unwrap_or_else(|| WebhookResult {
            success: false,
            status_code: None,
            response_body: None,
            error: Some("All retry attempts failed".to_string()),
            attempt: self.config.retry_attempts,
            timestamp: Utc::now(),
        })
    }

    async fn send_single(&self, event: &WebhookEvent, attempt: u32) -> WebhookResult {
        let payload = serde_json::to_string(event).unwrap();
        
        let mut request = self.client
            .post(&self.config.url)
            .header("Content-Type", "application/json")
            .header("User-Agent", "CyberSpider/7.8.0pro")
            .header("X-CyberSpider-Event", &event.event_type)
            .header("X-CyberSpider-Attempt", attempt.to_string());

        for (key, value) in &self.config.headers {
            request = request.header(key, value);
        }

        let signature = self.create_signature(&payload);
        if !signature.is_empty() {
            request = request.header("X-CyberSpider-Signature", signature);
        }

        match request.body(payload).send().await {
            Ok(response) => {
                let status_code = response.status().as_u16();
                let response_body = response.text().await.ok();
                
                let success = status_code >= 200 && status_code < 300;
                
                WebhookResult {
                    success,
                    status_code: Some(status_code),
                    response_body,
                    error: if success { None } else { Some(format!("HTTP {}", status_code)) },
                    attempt,
                    timestamp: Utc::now(),
                }
            }
            Err(e) => WebhookResult {
                success: false,
                status_code: None,
                response_body: None,
                error: Some(e.to_string()),
                attempt,
                timestamp: Utc::now(),
            },
        }
    }
}

#[async_trait]
impl WebhookSender for HttpWebhookSender {
    async fn send_event(&mut self, event: WebhookEvent) -> Result<WebhookResult> {
        if !self.config.enabled_events.contains(&event.event_type) {
            return Ok(WebhookResult {
                success: true,
                status_code: None,
                response_body: None,
                error: Some("Event type not enabled".to_string()),
                attempt: 0,
                timestamp: Utc::now(),
            });
        }

        Ok(self.send_with_retry(&event).await)
    }

    async fn send_batch(&mut self, events: Vec<WebhookEvent>) -> Result<Vec<WebhookResult>> {
        let mut results = Vec::new();
        
        for event in events {
            let result = self.send_event(event).await?;
            results.push(result);
        }
        
        Ok(results)
    }

    async fn test_connection(&self) -> Result<bool> {
        let test_event = WebhookEvent {
            event_type: "test".to_string(),
            timestamp: Utc::now(),
            session_id: "test_session".to_string(),
            data: serde_json::json!({"message": "Test connection"}),
            metadata: HashMap::new(),
        };

        let result = self.send_single(&test_event, 1).await;
        Ok(result.success)
    }

    fn get_config(&self) -> &WebhookConfig {
        &self.config
    }
}

pub struct WebhookManager {
    senders: Vec<Box<dyn WebhookSender>>,
    event_queue: Vec<WebhookEvent>,
    batch_size: usize,
    batch_timeout_seconds: u64,
}

impl WebhookManager {
    pub fn new() -> Self {
        Self {
            senders: Vec::new(),
            event_queue: Vec::new(),
            batch_size: 10,
            batch_timeout_seconds: 30,
        }
    }

    pub fn add_sender(&mut self, sender: Box<dyn WebhookSender>) {
        self.senders.push(sender);
    }

    pub async fn queue_event(&mut self, event: WebhookEvent) {
        self.event_queue.push(event);
        
        if self.event_queue.len() >= self.batch_size {
            self.flush_events().await;
        }
    }

    pub async fn flush_events(&mut self) {
        if self.event_queue.is_empty() {
            return;
        }

        let events = std::mem::take(&mut self.event_queue);
        
        for sender in &mut self.senders {
            if let Err(e) = sender.send_batch(events.clone()).await {
                eprintln!("Failed to send webhook batch: {}", e);
            }
        }
    }

    pub async fn send_immediate(&mut self, event: WebhookEvent) -> Result<Vec<WebhookResult>> {
        let mut results = Vec::new();
        
        for sender in &mut self.senders {
            let result = sender.send_event(event.clone()).await?;
            results.push(result);
        }
        
        Ok(results)
    }

    pub async fn test_all_connections(&self) -> Vec<(String, bool)> {
        let mut results = Vec::new();
        
        for (i, sender) in self.senders.iter().enumerate() {
            let test_result = sender.test_connection().await.unwrap_or(false);
            results.push((format!("webhook_{}", i), test_result));
        }
        
        results
    }

    pub fn get_queue_size(&self) -> usize {
        self.event_queue.len()
    }

    pub async fn start_background_processor(&mut self) -> tokio::task::JoinHandle<()> {
        let batch_timeout = self.batch_timeout_seconds;
        
        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(batch_timeout));
            
            loop {
                interval.tick().await;
                // In a real implementation, you'd have a shared reference to the manager
                // to flush events periodically
            }
        });
        
        handle
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookTemplate {
    pub name: String,
    pub description: String,
    pub event_types: Vec<String>,
    pub payload_template: String,
    pub required_headers: HashMap<String, String>,
}

pub struct WebhookTemplateEngine {
    templates: HashMap<String, WebhookTemplate>,
}

impl WebhookTemplateEngine {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    pub fn add_template(&mut self, template: WebhookTemplate) {
        self.templates.insert(template.name.clone(), template);
    }

    pub fn render_event(&self, template_name: &str, event: &WebhookEvent) -> Result<serde_json::Value> {
        let template = self.templates.get(template_name)
            .ok_or_else(|| anyhow::anyhow!("Template '{}' not found", template_name))?;

        let mut context = serde_json::json!({
            "event": event,
            "timestamp": event.timestamp.to_rfc3339(),
            "session_id": event.session_id,
            "event_type": event.event_type,
        });

        // Merge event data into context
        if let serde_json::Value::Object(mut map) = context {
            if let serde_json::Value::Object(event_data) = &event.data {
                for (key, value) in event_data {
                    map.insert(key.clone(), value.clone());
                }
            }
            context = serde_json::Value::Object(map);
        }

        // Simple template rendering (in a real implementation, use a proper templating engine)
        let rendered = template.payload_template
            .replace("{{timestamp}}", &event.timestamp.to_rfc3339())
            .replace("{{session_id}}", &event.session_id)
            .replace("{{event_type}}", &event.event_type);

        // Log the context for debugging purposes
        eprintln!("Webhook context: {}", context);

        serde_json::from_str(&rendered).map_err(|e| anyhow::anyhow!("Failed to parse rendered template: {}", e))
    }
}
