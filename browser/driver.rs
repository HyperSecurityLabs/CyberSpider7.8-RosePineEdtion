use anyhow::Result;
use headless_chrome::{Browser, Tab, LaunchOptions};
use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption;
use std::time::Duration;
use std::ffi::OsStr;
use std::sync::Arc;
use crate::browser::{BrowserEngine, BrowserConfig, Cookie};

pub struct ChromeDriver {
    browser: Option<Browser>,
    tab: Option<Arc<Tab>>,
    config: BrowserConfig,
}

impl ChromeDriver {
    pub async fn new(config: BrowserConfig) -> Result<Self> {
        let _launch_options = LaunchOptions {
            headless: config.headless,
            window_size: Some((config.viewport_width, config.viewport_height)),
            args: vec![
                OsStr::new("--no-sandbox"),
                OsStr::new("--disable-dev-shm-usage"),
                OsStr::new("--disable-gpu"),
                OsStr::new("--disable-web-security"),
                OsStr::new(&format!("--user-agent={}", config.user_agent)),
            ],
            ..Default::default()
        };

        Ok(Self {
            browser: None,
            tab: None,
            config,
        })
    }

    async fn initialize(&mut self) -> Result<()> {
        let user_agent_arg = format!("--user-agent={}", self.config.user_agent);
        let launch_options = LaunchOptions {
            headless: self.config.headless,
            window_size: Some((self.config.viewport_width, self.config.viewport_height)),
            args: vec![
                OsStr::new("--no-sandbox"),
                OsStr::new("--disable-dev-shm-usage"),
                OsStr::new("--disable-gpu"),
                OsStr::new("--disable-web-security"),
                OsStr::new(&user_agent_arg),
            ],
            ..Default::default()
        };

        self.browser = Some(Browser::new(launch_options)?);
        
        if let Some(browser) = &self.browser {
            self.tab = Some(browser.new_tab()?);
            
            if let Some(tab) = &self.tab {
                tab.set_default_timeout(Duration::from_secs(self.config.timeout_seconds));
                
                // Request interception disabled due to API compatibility issues
                // TODO: Fix request interception with correct API
                /*
                if !self.config.enable_images {
                    let interceptor = Arc::new(move |request: &RequestInterceptedEventParams| {
                        // Block images and other unnecessary resources
                        let resource_type = &request.resource_Type;
                        if matches!(resource_type, ResourceType::Image | ResourceType::Stylesheet | ResourceType::Font) {
                            headless_chrome::protocol::cdp::Network::ContinueInterceptedRequest {
                                interception_id: request.interception_id.clone(),
                                error_reason: None,
                                raw_response: None,
                                url: None,
                                method: None,
                                post_data: None,
                                headers: None,
                                auth_challenge_response: None,
                            }
                        } else {
                            headless_chrome::protocol::cdp::Network::ContinueInterceptedRequest {
                                interception_id: request.interception_id.clone(),
                                error_reason: None,
                                raw_response: None,
                                url: None,
                                method: None,
                                post_data: None,
                                headers: None,
                                auth_challenge_response: None,
                            }
                        }
                    });
                    
                    tab.enable_request_interception(interceptor)?;
                }
                */
            }
        }

        Ok(())
    }

    fn ensure_initialized(&mut self) -> Result<()> {
        if self.browser.is_none() || self.tab.is_none() {
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(self.initialize())
            })?;
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl BrowserEngine for ChromeDriver {
    async fn navigate_to(&mut self, url: &str) -> Result<()> {
        self.ensure_initialized()?;
        
        if let Some(tab) = &self.tab {
            tab.navigate_to(url)?;
            tab.wait_until_navigated()?;
        }
        
        Ok(())
    }

    async fn get_page_source(&self) -> Result<String> {
        if let Some(tab) = &self.tab {
            let html = tab.get_content()?;
            Ok(html)
        } else {
            Err(anyhow::anyhow!("Browser not initialized"))
        }
    }

    async fn get_title(&self) -> Result<String> {
        if let Some(tab) = &self.tab {
            let title = tab.get_title()?;
            Ok(title)
        } else {
            Err(anyhow::anyhow!("Browser not initialized"))
        }
    }

    async fn execute_script(&self, script: &str) -> Result<String> {
        if let Some(tab) = &self.tab {
            let result = tab.evaluate(script, true)?.value;
            Ok(format!("{:?}", result))
        } else {
            Err(anyhow::anyhow!("Browser not initialized"))
        }
    }

    async fn wait_for_load(&mut self) -> Result<()> {
        if let Some(tab) = &self.tab {
            tab.wait_until_navigated()?;
        }
        Ok(())
    }

    async fn screenshot(&self) -> Result<Vec<u8>> {
        if let Some(tab) = &self.tab {
            let png_data = tab.capture_screenshot(
                CaptureScreenshotFormatOption::Png,
                Some(90),
                None,
                true,
            )?;
            Ok(png_data)
        } else {
            Err(anyhow::anyhow!("Browser not initialized"))
        }
    }

    async fn get_cookies(&self) -> Result<Vec<Cookie>> {
        if let Some(tab) = &self.tab {
            let cookies = tab.get_cookies()?;
            let mut result = Vec::new();
            
            for cookie in cookies {
                result.push(Cookie {
                    name: cookie.name,
                    value: cookie.value,
                    domain: cookie.domain,
                    path: cookie.path,
                    secure: cookie.secure,
                    http_only: cookie.http_only,
                });
            }
            
            Ok(result)
        } else {
            Err(anyhow::anyhow!("Browser not initialized"))
        }
    }

    async fn set_cookies(&mut self, cookies: &[Cookie]) -> Result<()> {
        if let Some(tab) = &self.tab {
            for cookie in cookies {
                let cookie_builder = headless_chrome::protocol::cdp::Network::CookieParam {
                    name: cookie.name.clone(),
                    value: cookie.value.clone(),
                    url: None,
                    domain: Some(cookie.domain.clone()),
                    path: Some(cookie.path.clone()),
                    secure: Some(cookie.secure),
                    http_only: Some(cookie.http_only),
                    expires: None,
                    priority: None,
                    same_party: None,
                    source_scheme: None,
                    partition_key: None,
                    same_site: None,
                    source_port: None,
                };
                
                tab.set_cookies(vec![cookie_builder])?;
            }
        }
        Ok(())
    }

    async fn close(&mut self) -> Result<()> {
        self.tab = None;
        self.browser = None;
        Ok(())
    }
}
