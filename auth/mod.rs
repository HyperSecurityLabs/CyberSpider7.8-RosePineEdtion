use anyhow::Result;
use async_trait::async_trait;
use reqwest::{Client, header};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::time::Duration;
use base64::{Engine as _, engine::general_purpose};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSession {
    pub session_id: String,
    pub base_url: String,
    pub cookies: Vec<AuthCookie>,
    pub headers: HashMap<String, String>,
    pub auth_type: AuthType,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthCookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub secure: bool,
    pub http_only: bool,
    pub expires: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum AuthType {
    None,
    Basic,
    Bearer,
    Cookie,
    Form,
    OAuth2,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub username: Option<String>,
    pub password: Option<String>,
    pub api_key: Option<String>,
    pub bearer_token: Option<String>,
    pub login_url: Option<String>,
    pub login_form_data: Option<HashMap<String, String>>,
    pub oauth2_config: Option<OAuth2Config>,
    pub custom_headers: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Config {
    pub client_id: String,
    pub client_secret: String,
    pub token_url: String,
    pub scope: Option<String>,
    pub grant_type: String,
}

#[async_trait]
pub trait AuthManager {
    async fn create_session(&mut self, base_url: String, config: AuthConfig) -> Result<AuthSession>;
    async fn authenticate(&mut self, session: &mut AuthSession) -> Result<()>;
    async fn refresh_session(&mut self, session: &mut AuthSession) -> Result<()>;
    async fn validate_session(&self, session: &AuthSession) -> Result<bool>;
    async fn destroy_session(&mut self, session_id: &str) -> Result<bool>;
    async fn get_active_sessions(&self) -> Vec<&AuthSession>;
    async fn update_session_headers(&mut self, session_id: &str, headers: HashMap<String, String>) -> Result<()>;
}

pub struct SessionManager {
    sessions: HashMap<String, AuthSession>,
    client: Client,
}

impl SessionManager {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .cookie_store(true)
            .build()
            .unwrap();

        Self {
            sessions: HashMap::new(),
            client,
        }
    }

    fn generate_session_id(&self) -> String {
        uuid::Uuid::new_v4().to_string()
    }

    async fn test_authentication(&self, session: &AuthSession) -> Result<bool> {
        let test_url = format!("{}/", session.base_url.trim_end_matches('/'));
        
        let mut request = self.client.get(&test_url);
        
        // Add headers
        for (key, value) in &session.headers {
            request = request.header(key, value);
        }
        
        // Add cookies
        for cookie in &session.cookies {
            let cookie_str = format!("{}={}", cookie.name, cookie.value);
            request = request.header(header::COOKIE, cookie_str);
        }
        
        match request.send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

#[async_trait::async_trait]
impl AuthManager for SessionManager {
    async fn create_session(&mut self, base_url: String, config: AuthConfig) -> Result<AuthSession> {
        let session_id = self.generate_session_id();
        let auth_type = self.determine_auth_type(&config);
        
        let mut session = AuthSession {
            session_id: session_id.clone(),
            base_url,
            cookies: Vec::new(),
            headers: config.custom_headers.clone(),
            auth_type,
            created_at: chrono::Utc::now(),
            last_used: None,
            is_active: false,
        };

        // Authenticate before inserting into sessions
        self.authenticate(&mut session).await?;
        
        // Insert authenticated session
        self.sessions.insert(session_id.clone(), session.clone());
        
        Ok(session)
    }

    async fn authenticate(&mut self, session: &mut AuthSession) -> Result<()> {
        match session.auth_type {
            AuthType::Basic => self.authenticate_basic(session).await?,
            AuthType::Bearer => self.authenticate_bearer(session).await?,
            AuthType::Cookie => self.authenticate_cookie(session).await?,
            AuthType::Form => self.authenticate_form(session).await?,
            AuthType::OAuth2 => self.authenticate_oauth2(session).await?,
            AuthType::None | AuthType::Custom => {}
        }

        session.is_active = self.test_authentication(session).await?;
        session.last_used = Some(chrono::Utc::now());
        
        Ok(())
    }

    async fn refresh_session(&mut self, session: &mut AuthSession) -> Result<()> {
        match session.auth_type {
            AuthType::OAuth2 => self.refresh_oauth2_token(session).await?,
            AuthType::Cookie => self.refresh_cookies(session).await?,
            _ => {}
        }

        session.is_active = self.test_authentication(session).await?;
        session.last_used = Some(chrono::Utc::now());
        
        Ok(())
    }

    async fn validate_session(&self, session: &AuthSession) -> Result<bool> {
        if !session.is_active {
            return Ok(false);
        }

        // Check if session is too old (24 hours)
        let max_age = chrono::Duration::hours(24);
        if chrono::Utc::now() - session.created_at > max_age {
            return Ok(false);
        }

        self.test_authentication(session).await
    }

    async fn destroy_session(&mut self, session_id: &str) -> Result<bool> {
        Ok(self.sessions.remove(session_id).is_some())
    }

    async fn get_active_sessions(&self) -> Vec<&AuthSession> {
        self.sessions
            .values()
            .filter(|s| s.is_active)
            .collect()
    }

    async fn update_session_headers(&mut self, session_id: &str, headers: HashMap<String, String>) -> Result<()> {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.headers.extend(headers);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Session not found"))
        }
    }
}

impl SessionManager {
    fn determine_auth_type(&self, config: &AuthConfig) -> AuthType {
        if config.oauth2_config.is_some() {
            AuthType::OAuth2
        } else if let Some(bearer_token) = &config.bearer_token {
            if !bearer_token.is_empty() {
                AuthType::Bearer
            } else {
                AuthType::None
            }
        } else if let (Some(username), Some(password)) = (&config.username, &config.password) {
            if !username.is_empty() && !password.is_empty() {
                AuthType::Basic
            } else {
                AuthType::None
            }
        } else if config.login_form_data.is_some() || config.login_url.is_some() {
            AuthType::Form
        } else {
            AuthType::None
        }
    }

    async fn authenticate_basic(&self, session: &mut AuthSession) -> Result<()> {
        if let (Some(username), Some(password)) = (&session.headers.get("username"), &session.headers.get("password")) {
            let credentials = format!("{}:{}", username, password);
            let encoded = general_purpose::STANDARD.encode(credentials);
            session.headers.insert("Authorization".to_string(), format!("Basic {}", encoded));
        }
        Ok(())
    }

    async fn authenticate_bearer(&self, session: &mut AuthSession) -> Result<()> {
        if let Some(bearer_token) = session.headers.get("bearer_token") {
            session.headers.insert("Authorization".to_string(), format!("Bearer {}", bearer_token));
        }
        Ok(())
    }

    async fn authenticate_cookie(&self, session: &mut AuthSession) -> Result<()> {
        // Real cookie-based authentication implementation
        // Validate existing cookies and refresh if needed
        
        let test_url = format!("{}/", session.base_url.trim_end_matches('/'));
        let mut request = self.client.get(&test_url);
        
        // Add existing cookies
        for cookie in &session.cookies {
            let cookie_str = format!("{}={}", cookie.name, cookie.value);
            request = request.header(header::COOKIE, cookie_str);
        }
        
        let response = request.send().await?;
        
        if response.status().is_success() {
            // Extract any new cookies from response
            let new_cookies: Vec<_> = response.cookies()
                .collect::<Vec<_>>()
                .into_iter()
                .map(|c| AuthCookie {
                    name: c.name().to_string(),
                    value: c.value().to_string(),
                    domain: c.domain().unwrap_or(&session.base_url).to_string(),
                    path: c.path().unwrap_or("/").to_string(),
                    secure: c.secure(),
                    http_only: c.http_only(),
                    expires: c.expires().map(|exp| exp.into()),
                })
                .collect();
            
            // Merge new cookies with existing ones
            for new_cookie in new_cookies {
                if let Some(pos) = session.cookies.iter().position(|c| c.name == new_cookie.name && c.domain == new_cookie.domain) {
                    session.cookies[pos] = new_cookie;
                } else {
                    session.cookies.push(new_cookie);
                }
            }
        } else {
            return Err(anyhow::anyhow!("Cookie authentication failed with status: {}", response.status()));
        }
        
        Ok(())
    }

    async fn authenticate_form(&self, session: &mut AuthSession) -> Result<()> {
        let login_url = session.headers.get("login_url")
            .or_else(|| session.headers.get("login_url"))
            .cloned()
            .unwrap_or_else(|| format!("{}/login", session.base_url));

        let mut form_data = HashMap::new();
        form_data.insert("username".to_string(), session.headers.get("username").unwrap_or(&String::new()).clone());
        form_data.insert("password".to_string(), session.headers.get("password").unwrap_or(&String::new()).clone());

        let response = self.client.post(login_url)
            .form(&form_data)
            .send()
            .await?;

        if response.status().is_success() {
            // Extract cookies from response
            let cookies = response.cookies()
                .map(|c| AuthCookie {
                    name: c.name().to_string(),
                    value: c.value().to_string(),
                    domain: c.domain().unwrap_or("").to_string(),
                    path: c.path().unwrap_or("/").to_string(),
                    secure: c.secure(),
                    http_only: c.http_only(),
                    expires: None,
                })
                .collect();

            session.cookies = cookies;
        }

        Ok(())
    }

    async fn authenticate_oauth2(&self, session: &mut AuthSession) -> Result<()> {
        // Real OAuth2 implementation
        let client_id = session.headers.get("client_id")
            .or_else(|| session.headers.get("oauth2_client_id"))
            .ok_or_else(|| anyhow::anyhow!("OAuth2 client_id not found"))?;
        
        let client_secret = session.headers.get("client_secret")
            .or_else(|| session.headers.get("oauth2_client_secret"))
            .ok_or_else(|| anyhow::anyhow!("OAuth2 client_secret not found"))?;
        
        let token_url = session.headers.get("token_url")
            .or_else(|| session.headers.get("oauth2_token_url"))
            .cloned()
            .unwrap_or_else(|| format!("{}/oauth/token", session.base_url));
        
        let mut form_data = HashMap::new();
        form_data.insert("grant_type".to_string(), "client_credentials".to_string());
        form_data.insert("client_id".to_string(), client_id.clone());
        form_data.insert("client_secret".to_string(), client_secret.clone());
        
        if let Some(scope) = session.headers.get("scope") {
            form_data.insert("scope".to_string(), scope.clone());
        }
        
        let response = self.client.post(token_url)
            .form(&form_data)
            .send()
            .await?;
        
        if response.status().is_success() {
            let token_response: serde_json::Value = response.json().await?;
            
            if let Some(access_token) = token_response.get("access_token").and_then(|t| t.as_str()) {
                session.headers.insert("Authorization".to_string(), format!("Bearer {}", access_token));
                
                // Store token info for refresh
                if let Some(expires_in) = token_response.get("expires_in").and_then(|e| e.as_u64()) {
                    let expires_at = chrono::Utc::now() + chrono::Duration::seconds(expires_in as i64);
                    session.headers.insert("token_expires_at".to_string(), expires_at.to_rfc3339());
                }
                
                if let Some(refresh_token) = token_response.get("refresh_token").and_then(|t| t.as_str()) {
                    session.headers.insert("refresh_token".to_string(), refresh_token.to_string());
                }
            } else {
                return Err(anyhow::anyhow!("No access token in OAuth2 response"));
            }
        } else {
            return Err(anyhow::anyhow!("OAuth2 token request failed: {}", response.status()));
        }
        
        Ok(())
    }

    async fn refresh_oauth2_token(&self, session: &mut AuthSession) -> Result<()> {
        // Real OAuth2 token refresh implementation
        let refresh_token = session.headers.get("refresh_token")
            .ok_or_else(|| anyhow::anyhow!("No refresh token available"))?;
        
        let client_id = session.headers.get("client_id")
            .or_else(|| session.headers.get("oauth2_client_id"))
            .ok_or_else(|| anyhow::anyhow!("OAuth2 client_id not found"))?;
        
        let client_secret = session.headers.get("client_secret")
            .or_else(|| session.headers.get("oauth2_client_secret"))
            .ok_or_else(|| anyhow::anyhow!("OAuth2 client_secret not found"))?;
        
        let default_token_url = format!("{}/oauth/token", session.base_url);
        let token_url = session.headers.get("token_url")
            .or_else(|| session.headers.get("oauth2_token_url"))
            .unwrap_or(&default_token_url);
        
        let mut form_data = HashMap::new();
        form_data.insert("grant_type".to_string(), "refresh_token".to_string());
        form_data.insert("refresh_token".to_string(), refresh_token.clone());
        form_data.insert("client_id".to_string(), client_id.clone());
        form_data.insert("client_secret".to_string(), client_secret.clone());
        
        let response = self.client.post(token_url)
            .form(&form_data)
            .send()
            .await?;
        
        if response.status().is_success() {
            let token_response: serde_json::Value = response.json().await?;
            
            if let Some(access_token) = token_response.get("access_token").and_then(|t| t.as_str()) {
                session.headers.insert("Authorization".to_string(), format!("Bearer {}", access_token));
                
                // Update token expiration
                if let Some(expires_in) = token_response.get("expires_in").and_then(|e| e.as_u64()) {
                    let expires_at = chrono::Utc::now() + chrono::Duration::seconds(expires_in as i64);
                    session.headers.insert("token_expires_at".to_string(), expires_at.to_rfc3339());
                }
                
                // Update refresh token if provided
                if let Some(new_refresh_token) = token_response.get("refresh_token").and_then(|t| t.as_str()) {
                    session.headers.insert("refresh_token".to_string(), new_refresh_token.to_string());
                }
            } else {
                return Err(anyhow::anyhow!("No access token in OAuth2 refresh response"));
            }
        } else {
            return Err(anyhow::anyhow!("OAuth2 token refresh failed: {}", response.status()));
        }
        
        Ok(())
    }

    async fn refresh_cookies(&self, session: &mut AuthSession) -> Result<()> {
        // Real cookie refresh implementation
        let test_url = format!("{}/", session.base_url.trim_end_matches('/'));
        
        // Make a request to refresh cookies
        let mut request = self.client.get(&test_url);
        
        // Add existing cookies
        for cookie in &session.cookies {
            let cookie_str = format!("{}={}", cookie.name, cookie.value);
            request = request.header(header::COOKIE, cookie_str);
        }
        
        let response = request.send().await?;
        
        if response.status().is_success() {
            // Extract updated cookies from response
            let updated_cookies: Vec<AuthCookie> = response.cookies()
                .map(|c| AuthCookie {
                    name: c.name().to_string(),
                    value: c.value().to_string(),
                    domain: c.domain().unwrap_or(&session.base_url).to_string(),
                    path: c.path().unwrap_or("/").to_string(),
                    secure: c.secure(),
                    http_only: c.http_only(),
                    expires: c.expires().map(|exp| exp.into()),
                })
                .collect();
            
            // Update session cookies with fresh ones
            for updated_cookie in updated_cookies {
                if let Some(pos) = session.cookies.iter().position(|c| c.name == updated_cookie.name && c.domain == updated_cookie.domain) {
                    // Update existing cookie
                    session.cookies[pos] = updated_cookie;
                } else {
                    // Add new cookie
                    session.cookies.push(updated_cookie);
                }
            }
            
            // Remove expired cookies
            let now = chrono::Utc::now();
            session.cookies.retain(|cookie| {
                if let Some(expires) = cookie.expires {
                    now < expires
                } else {
                    true // No expiration date, keep it
                }
            });
        } else {
            return Err(anyhow::anyhow!("Cookie refresh failed with status: {}", response.status()));
        }
        
        Ok(())
    }
}

pub struct AuthClient {
    session_manager: SessionManager,
}

impl AuthClient {
    pub fn new() -> Self {
        Self {
            session_manager: SessionManager::new(),
        }
    }

    pub async fn make_authenticated_request(
        &mut self,
        session_id: &str,
        url: &str,
        method: reqwest::Method,
    ) -> Result<reqwest::Response> {
        let session = self.session_manager.sessions.get(session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found"))?;

        let mut request = self.session_manager.client.request(method.clone(), url);

        // Add headers
        for (key, value) in &session.headers {
            request = request.header(key, value);
        }

        // Add cookies
        for cookie in &session.cookies {
            let cookie_str = format!("{}={}", cookie.name, cookie.value);
            request = request.header(header::COOKIE, cookie_str);
        }

        let response = request.send().await?;
        
        // Update session last used
        if let Some(session) = self.session_manager.sessions.get_mut(session_id) {
            session.last_used = Some(chrono::Utc::now());
        }

        Ok(response)
    }

    pub async fn get_authenticated(&mut self, session_id: &str, url: &str) -> Result<reqwest::Response> {
        self.make_authenticated_request(session_id, url, reqwest::Method::GET).await
    }

    pub async fn post_authenticated(
        &mut self,
        session_id: &str,
        url: &str,
        _body: String,
    ) -> Result<reqwest::Response> {
        let response = self.make_authenticated_request(session_id, url, reqwest::Method::POST).await?;
        Ok(response)
    }

    pub async fn cleanup_expired_sessions(&mut self) -> Result<usize> {
        let mut expired_sessions = Vec::new();
        let max_age = chrono::Duration::hours(24);

        for (session_id, session) in &self.session_manager.sessions {
            if chrono::Utc::now() - session.created_at > max_age {
                expired_sessions.push(session_id.clone());
            }
        }

        let count = expired_sessions.len();
        for session_id in expired_sessions {
            self.session_manager.destroy_session(&session_id).await?;
        }

        Ok(count)
    }

    pub async fn get_session_stats(&self) -> AuthStats {
        let total_sessions = self.session_manager.sessions.len();
        let active_sessions = self.session_manager.get_active_sessions().await.len();
        
        let auth_types: HashMap<AuthType, usize> = self.session_manager.sessions
            .values()
            .fold(HashMap::new(), |mut acc, session| {
                *acc.entry(session.auth_type.clone()).or_insert(0) += 1;
                acc
            });

        AuthStats {
            total_sessions,
            active_sessions,
            auth_types,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuthStats {
    pub total_sessions: usize,
    pub active_sessions: usize,
    pub auth_types: HashMap<AuthType, usize>,
}
