pub mod driver;
pub mod js_executor;

use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait BrowserEngine {
    async fn navigate_to(&mut self, url: &str) -> Result<()>;
    async fn get_page_source(&self) -> Result<String>;
    async fn get_title(&self) -> Result<String>;
    async fn execute_script(&self, script: &str) -> Result<String>;
    async fn wait_for_load(&mut self) -> Result<()>;
    async fn screenshot(&self) -> Result<Vec<u8>>;
    async fn get_cookies(&self) -> Result<Vec<Cookie>>;
    async fn set_cookies(&mut self, cookies: &[Cookie]) -> Result<()>;
    async fn close(&mut self) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub secure: bool,
    pub http_only: bool,
}

#[derive(Debug, Clone)]
pub struct BrowserConfig {
    pub headless: bool,
    pub user_agent: String,
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub timeout_seconds: u64,
    pub enable_javascript: bool,
    pub enable_images: bool,
}
