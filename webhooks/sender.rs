use anyhow::Result;
use crate::webhooks::{WebhookSender, WebhookEvent, WebhookResult, WebhookConfig};
use std::collections::HashMap;

pub struct WebhookSenderFactory;

impl WebhookSenderFactory {
    pub fn create_http_sender(config: WebhookConfig) -> Result<Box<dyn WebhookSender>> {
        Ok(Box::new(crate::webhooks::HttpWebhookSender::new(config)?))
    }

    pub fn create_slack_sender(webhook_url: String) -> Result<Box<dyn WebhookSender>> {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let config = WebhookConfig {
            url: webhook_url,
            timeout_seconds: 30,
            retry_attempts: 3,
            retry_delay_seconds: 5,
            secret: None,
            headers,
            enabled_events: vec![
                "crawl_started".to_string(),
                "crawl_completed".to_string(),
                "security_finding".to_string(),
                "error".to_string(),
            ],
        };

        Self::create_http_sender(config)
    }

    pub fn create_discord_sender(webhook_url: String) -> Result<Box<dyn WebhookSender>> {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let config = WebhookConfig {
            url: webhook_url,
            timeout_seconds: 30,
            retry_attempts: 3,
            retry_delay_seconds: 5,
            secret: None,
            headers,
            enabled_events: vec![
                "crawl_started".to_string(),
                "crawl_completed".to_string(),
                "security_finding".to_string(),
                "error".to_string(),
            ],
        };

        Self::create_http_sender(config)
    }

    pub fn create_teams_sender(webhook_url: String) -> Result<Box<dyn WebhookSender>> {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let config = WebhookConfig {
            url: webhook_url,
            timeout_seconds: 30,
            retry_attempts: 3,
            retry_delay_seconds: 5,
            secret: None,
            headers,
            enabled_events: vec![
                "crawl_started".to_string(),
                "crawl_completed".to_string(),
                "security_finding".to_string(),
                "error".to_string(),
            ],
        };

        Self::create_http_sender(config)
    }
}

pub struct SlackWebhookSender {
    http_sender: Box<dyn WebhookSender>,
}

impl SlackWebhookSender {
    pub fn new(webhook_url: String) -> Result<Self> {
        let http_sender = WebhookSenderFactory::create_slack_sender(webhook_url)?;
        Ok(Self { http_sender })
    }

    fn format_slack_message(&self, event: &WebhookEvent) -> WebhookEvent {
        let slack_payload = match event.event_type.as_str() {
            "crawl_started" => serde_json::json!({
                "text": format!("🕷️ CyberSpider crawl started for session {}", event.session_id),
                "attachments": [{
                    "color": "good",
                    "fields": [{
                        "title": "Session ID",
                        "value": event.session_id,
                        "short": true
                    }, {
                        "title": "Started At",
                        "value": event.timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                        "short": true
                    }]
                }]
            }),
            "crawl_completed" => serde_json::json!({
                "text": format!("✅ CyberSpider crawl completed for session {}", event.session_id),
                "attachments": [{
                    "color": "good",
                    "fields": [
                        {"title": "Session ID", "value": event.session_id, "short": true},
                        {"title": "Completed At", "value": event.timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string(), "short": true}
                    ]
                }]
            }),
            "security_finding" => {
                let severity = event.data.get("severity").and_then(|v| v.as_str()).unwrap_or("unknown");
                let color = match severity {
                    "critical" => "danger",
                    "high" => "warning",
                    "medium" => "warning",
                    "low" => "good",
                    _ => "#cccccc",
                };

                serde_json::json!({
                    "text": format!("🔍 Security finding detected: {}", event.data.get("finding_type").and_then(|v| v.as_str()).unwrap_or("Unknown")),
                    "attachments": [{
                        "color": color,
                        "fields": [
                            {"title": "Type", "value": event.data.get("finding_type").and_then(|v| v.as_str()).unwrap_or("Unknown"), "short": true},
                            {"title": "Severity", "value": severity, "short": true},
                            {"title": "URL", "value": event.data.get("url").and_then(|v| v.as_str()).unwrap_or("N/A"), "short": false},
                            {"title": "Description", "value": event.data.get("description").and_then(|v| v.as_str()).unwrap_or("N/A"), "short": false}
                        ]
                    }]
                })
            },
            "error" => serde_json::json!({
                "text": format!("❌ CyberSpider error in session {}", event.session_id),
                "attachments": [{
                    "color": "danger",
                    "fields": [
                        {"title": "Error", "value": event.data.get("error").and_then(|v| v.as_str()).unwrap_or("Unknown error"), "short": false},
                        {"title": "Session ID", "value": event.session_id, "short": true},
                        {"title": "Time", "value": event.timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string(), "short": true}
                    ]
                }]
            }),
            _ => serde_json::json!({
                "text": format!("CyberSpider event: {}", event.event_type)
            }),
        };

        let mut slack_event = event.clone();
        slack_event.data = slack_payload;
        slack_event
    }
}

#[async_trait::async_trait]
impl WebhookSender for SlackWebhookSender {
    async fn send_event(&mut self, event: WebhookEvent) -> Result<WebhookResult> {
        let slack_event = self.format_slack_message(&event);
        self.http_sender.send_event(slack_event).await
    }

    async fn send_batch(&mut self, events: Vec<WebhookEvent>) -> Result<Vec<WebhookResult>> {
        let slack_events: Vec<WebhookEvent> = events.iter().map(|e| self.format_slack_message(e)).collect();
        self.http_sender.send_batch(slack_events).await
    }

    async fn test_connection(&self) -> Result<bool> {
        self.http_sender.test_connection().await
    }

    fn get_config(&self) -> &WebhookConfig {
        self.http_sender.get_config()
    }
}

pub struct DiscordWebhookSender {
    http_sender: Box<dyn WebhookSender>,
}

impl DiscordWebhookSender {
    pub fn new(webhook_url: String) -> Result<Self> {
        let http_sender = WebhookSenderFactory::create_discord_sender(webhook_url)?;
        Ok(Self { http_sender })
    }

    fn format_discord_message(&self, event: &WebhookEvent) -> WebhookEvent {
        let discord_payload = match event.event_type.as_str() {
            "crawl_started" => serde_json::json!({
                "content": format!(" CyberSpider crawl started for session {}", event.session_id),
                "embeds": [{
                    "title": "Crawl Started",
                    "color": 65280, // Green
                    "fields": [
                        {"name": "Session ID", "value": event.session_id, "inline": true},
                        {"name": "Started At", "value": event.timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string(), "inline": true}
                    ],
                    "timestamp": event.timestamp.to_rfc3339()
                }]
            }),
            "crawl_completed" => serde_json::json!({
                "content": format!(" CyberSpider crawl completed for session {}", event.session_id),
                "embeds": [{
                    "title": "Crawl Completed",
                    "color": 65280, // Green
                    "fields": [
                        {"name": "Session ID", "value": event.session_id, "inline": true},
                        {"name": "Completed At", "value": event.timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string(), "inline": true}
                    ],
                    "timestamp": event.timestamp.to_rfc3339()
                }]
            }),
            "security_finding" => {
                let severity = event.data.get("severity").and_then(|v| v.as_str()).unwrap_or("unknown");
                let color = match severity {
                    "critical" => 16711680, // Red
                    "high" => 16776960,     // Yellow
                    "medium" => 16776960,    // Yellow
                    "low" => 65280,          // Green
                    _ => 11750863,           // Grey
                };

                serde_json::json!({
                    "content": format!("🔍 Security finding detected: {}", event.data.get("finding_type").and_then(|v| v.as_str()).unwrap_or("Unknown")),
                    "embeds": [{
                        "title": "Security Finding",
                        "color": color,
                        "fields": [
                            {"name": "Type", "value": event.data.get("finding_type").and_then(|v| v.as_str()).unwrap_or("Unknown"), "inline": true},
                            {"name": "Severity", "value": severity, "inline": true},
                            {"name": "URL", "value": event.data.get("url").and_then(|v| v.as_str()).unwrap_or("N/A")},
                            {"name": "Description", "value": event.data.get("description").and_then(|v| v.as_str()).unwrap_or("N/A")}
                        ],
                        "timestamp": event.timestamp.to_rfc3339()
                    }]
                })
            },
            "error" => serde_json::json!({
                "content": format!("❌ CyberSpider error in session {}", event.session_id),
                "embeds": [{
                    "title": "Error Occurred",
                    "color": 16711680, // Red
                    "fields": [
                        {"name": "Error", "value": event.data.get("error").and_then(|v| v.as_str()).unwrap_or("Unknown error")},
                        {"name": "Session ID", "value": event.session_id, "inline": true},
                        {"name": "Time", "value": event.timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string(), "inline": true}
                    ],
                    "timestamp": event.timestamp.to_rfc3339()
                }]
            }),
            _ => serde_json::json!({
                "content": format!("CyberSpider event: {}", event.event_type)
            }),
        };

        let mut discord_event = event.clone();
        discord_event.data = discord_payload;
        discord_event
    }
}

#[async_trait::async_trait]
impl WebhookSender for DiscordWebhookSender {
    async fn send_event(&mut self, event: WebhookEvent) -> Result<WebhookResult> {
        let discord_event = self.format_discord_message(&event);
        self.http_sender.send_event(discord_event).await
    }

    async fn send_batch(&mut self, events: Vec<WebhookEvent>) -> Result<Vec<WebhookResult>> {
        let discord_events: Vec<WebhookEvent> = events.iter().map(|e| self.format_discord_message(e)).collect();
        self.http_sender.send_batch(discord_events).await
    }

    async fn test_connection(&self) -> Result<bool> {
        self.http_sender.test_connection().await
    }

    fn get_config(&self) -> &WebhookConfig {
        self.http_sender.get_config()
    }
}
