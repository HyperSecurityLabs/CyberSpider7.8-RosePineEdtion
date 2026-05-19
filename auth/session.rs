use anyhow::Result;
use crate::auth::{AuthSession, AuthManager, AuthConfig};
use serde::{Serialize, Deserialize};

pub struct SessionStore {
    sessions: Vec<StoredSession>,
    storage_backend: Box<dyn SessionStorage>,
}

impl SessionStore {
    pub fn new(storage_backend: Box<dyn SessionStorage>) -> Self {
        Self {
            sessions: Vec::new(),
            storage_backend,
        }
    }

    pub async fn save_session(&mut self, session: &AuthSession) -> Result<()> {
        let stored_session = StoredSession::from_auth_session(session)?;
        self.storage_backend.save_session(&stored_session).await?;
        self.sessions.push(stored_session);
        Ok(())
    }

    pub async fn load_session(&mut self, session_id: &str) -> Result<Option<AuthSession>> {
        if let Some(stored) = self.sessions.iter().find(|s| s.session_id == session_id) {
            return Ok(Some(stored.to_auth_session()?));
        }

        if let Some(stored) = self.storage_backend.load_session(session_id).await? {
            self.sessions.push(stored.clone());
            return Ok(Some(stored.to_auth_session()?));
        }

        Ok(None)
    }

    pub async fn delete_session(&mut self, session_id: &str) -> Result<bool> {
        self.sessions.retain(|s| s.session_id != session_id);
        self.storage_backend.delete_session(session_id).await
    }

    pub async fn list_sessions(&mut self) -> Result<Vec<AuthSession>> {
        let stored_sessions = self.storage_backend.list_sessions().await?;
        let mut auth_sessions = Vec::new();

        for stored in stored_sessions {
            auth_sessions.push(stored.to_auth_session()?);
            if !self.sessions.iter().any(|s| s.session_id == stored.session_id) {
                self.sessions.push(stored);
            }
        }

        Ok(auth_sessions)
    }

    pub async fn cleanup_expired(&mut self) -> Result<usize> {
        let mut removed_count = 0;
        let max_age = chrono::Duration::hours(24);
        let now = chrono::Utc::now();

        // Remove from memory
        self.sessions.retain(|s| now - s.created_at <= max_age);

        // Remove from storage
        removed_count = self.storage_backend.cleanup_expired(max_age).await?;

        Ok(removed_count)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredSession {
    pub session_id: String,
    pub base_url: String,
    pub cookies: Vec<StoredCookie>,
    pub headers: std::collections::HashMap<String, String>,
    pub auth_type: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredCookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub secure: bool,
    pub http_only: bool,
    pub expires: Option<chrono::DateTime<chrono::Utc>>,
}

impl StoredSession {
    fn from_auth_session(session: &AuthSession) -> Result<Self> {
        let cookies = session.cookies.iter().map(|c| StoredCookie {
            name: c.name.clone(),
            value: c.value.clone(),
            domain: c.domain.clone(),
            path: c.path.clone(),
            secure: c.secure,
            http_only: c.http_only,
            expires: c.expires,
        }).collect();

        Ok(Self {
            session_id: session.session_id.clone(),
            base_url: session.base_url.clone(),
            cookies,
            headers: session.headers.clone(),
            auth_type: format!("{:?}", session.auth_type),
            created_at: session.created_at,
            last_used: session.last_used,
            is_active: session.is_active,
        })
    }

    fn to_auth_session(&self) -> Result<AuthSession> {
        use crate::auth::AuthType;

        let cookies = self.cookies.iter().map(|c| crate::auth::AuthCookie {
            name: c.name.clone(),
            value: c.value.clone(),
            domain: c.domain.clone(),
            path: c.path.clone(),
            secure: c.secure,
            http_only: c.http_only,
            expires: c.expires,
        }).collect();

        let auth_type = match self.auth_type.as_str() {
            "Basic" => AuthType::Basic,
            "Bearer" => AuthType::Bearer,
            "Cookie" => AuthType::Cookie,
            "Form" => AuthType::Form,
            "OAuth2" => AuthType::OAuth2,
            "Custom" => AuthType::Custom,
            _ => AuthType::None,
        };

        Ok(AuthSession {
            session_id: self.session_id.clone(),
            base_url: self.base_url.clone(),
            cookies,
            headers: self.headers.clone(),
            auth_type,
            created_at: self.created_at,
            last_used: self.last_used,
            is_active: self.is_active,
        })
    }
}

#[async_trait::async_trait]
pub trait SessionStorage {
    async fn save_session(&mut self, session: &StoredSession) -> Result<()>;
    async fn load_session(&self, session_id: &str) -> Result<Option<StoredSession>>;
    async fn delete_session(&mut self, session_id: &str) -> Result<bool>;
    async fn list_sessions(&self) -> Result<Vec<StoredSession>>;
    async fn cleanup_expired(&mut self, max_age: chrono::Duration) -> Result<usize>;
}

pub struct FileSessionStorage {
    file_path: String,
}

impl FileSessionStorage {
    pub fn new<P: AsRef<std::path::Path>>(file_path: P) -> Self {
        Self {
            file_path: file_path.as_ref().to_string_lossy().to_string(),
        }
    }
}

#[async_trait::async_trait]
impl SessionStorage for FileSessionStorage {
    async fn save_session(&mut self, session: &StoredSession) -> Result<()> {
        let mut sessions = self.load_all_sessions().await?;
        
        // Update or add session
        if let Some(pos) = sessions.iter().position(|s| s.session_id == session.session_id) {
            sessions[pos] = session.clone();
        } else {
            sessions.push(session.clone());
        }

        self.save_all_sessions(&sessions).await
    }

    async fn load_session(&self, session_id: &str) -> Result<Option<StoredSession>> {
        let sessions = self.load_all_sessions().await?;
        Ok(sessions.into_iter().find(|s| s.session_id == session_id))
    }

    async fn delete_session(&mut self, session_id: &str) -> Result<bool> {
        let mut sessions = self.load_all_sessions().await?;
        let original_len = sessions.len();
        sessions.retain(|s| s.session_id != session_id);
        
        if sessions.len() < original_len {
            self.save_all_sessions(&sessions).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn list_sessions(&self) -> Result<Vec<StoredSession>> {
        self.load_all_sessions().await
    }

    async fn cleanup_expired(&mut self, max_age: chrono::Duration) -> Result<usize> {
        let mut sessions = self.load_all_sessions().await?;
        let now = chrono::Utc::now();
        let original_len = sessions.len();
        
        sessions.retain(|s| now - s.created_at <= max_age);
        
        let removed_count = original_len - sessions.len();
        if removed_count > 0 {
            self.save_all_sessions(&sessions).await?;
        }
        
        Ok(removed_count)
    }
}

impl FileSessionStorage {
    async fn load_all_sessions(&self) -> Result<Vec<StoredSession>> {
        if std::path::Path::new(&self.file_path).exists() {
            let content = std::fs::read_to_string(&self.file_path)?;
            let sessions: Vec<StoredSession> = serde_json::from_str(&content)?;
            Ok(sessions)
        } else {
            Ok(Vec::new())
        }
    }

    async fn save_all_sessions(&self, sessions: &[StoredSession]) -> Result<()> {
        let content = serde_json::to_string_pretty(sessions)?;
        std::fs::write(&self.file_path, content)?;
        Ok(())
    }
}

pub struct MemorySessionStorage {
    sessions: std::collections::HashMap<String, StoredSession>,
}

impl MemorySessionStorage {
    pub fn new() -> Self {
        Self {
            sessions: std::collections::HashMap::new(),
        }
    }
}

#[async_trait::async_trait]
impl SessionStorage for MemorySessionStorage {
    async fn save_session(&mut self, session: &StoredSession) -> Result<()> {
        self.sessions.insert(session.session_id.clone(), session.clone());
        Ok(())
    }

    async fn load_session(&self, session_id: &str) -> Result<Option<StoredSession>> {
        Ok(self.sessions.get(session_id).cloned())
    }

    async fn delete_session(&mut self, session_id: &str) -> Result<bool> {
        Ok(self.sessions.remove(session_id).is_some())
    }

    async fn list_sessions(&self) -> Result<Vec<StoredSession>> {
        Ok(self.sessions.values().cloned().collect())
    }

    async fn cleanup_expired(&mut self, max_age: chrono::Duration) -> Result<usize> {
        let now = chrono::Utc::now();
        let original_len = self.sessions.len();
        
        self.sessions.retain(|_, s| now - s.created_at <= max_age);
        
        Ok(original_len - self.sessions.len())
    }
}

pub struct SessionValidator;

impl SessionValidator {
    pub fn validate_session_config(config: &AuthConfig) -> Result<()> {
        if let (Some(username), Some(password)) = (&config.username, &config.password) {
            if username.is_empty() || password.is_empty() {
                return Err(anyhow::anyhow!("Username and password cannot be empty"));
            }
        }

        if let Some(api_key) = &config.api_key {
            if api_key.is_empty() {
                return Err(anyhow::anyhow!("API key cannot be empty"));
            }
        }

        if let Some(bearer_token) = &config.bearer_token {
            if bearer_token.is_empty() {
                return Err(anyhow::anyhow!("Bearer token cannot be empty"));
            }
        }

        if let Some(login_url) = &config.login_url {
            if !login_url.starts_with("http://") && !login_url.starts_with("https://") {
                return Err(anyhow::anyhow!("Login URL must start with http:// or https://"));
            }
        }

        Ok(())
    }

    pub fn sanitize_session_data(session: &mut AuthSession) {
        // Remove sensitive data from headers
        session.headers.retain(|key, _| {
            !key.to_lowercase().contains("password") 
            && !key.to_lowercase().contains("secret")
            && !key.to_lowercase().contains("token")
        });

        // Mark old sessions as inactive
        let max_age = chrono::Duration::hours(24);
        if chrono::Utc::now() - session.created_at > max_age {
            session.is_active = false;
        }
    }

    pub fn generate_session_summary(&self, sessions: &[AuthSession]) -> SessionSummary {
        let total_sessions = sessions.len();
        let active_sessions = sessions.iter().filter(|s| s.is_active).count();
        let auth_types = sessions.iter()
            .map(|s| format!("{:?}", s.auth_type))
            .collect::<std::collections::Counter<_>>();

        SessionSummary {
            total_sessions,
            active_sessions,
            auth_types,
            oldest_session: sessions.iter().min_by_key(|s| s.created_at).map(|s| s.created_at),
            newest_session: sessions.iter().max_by_key(|s| s.created_at).map(|s| s.created_at),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SessionSummary {
    pub total_sessions: usize,
    pub active_sessions: usize,
    pub auth_types: std::collections::Counter<String>,
    pub oldest_session: Option<chrono::DateTime<chrono::Utc>>,
    pub newest_session: Option<chrono::DateTime<chrono::Utc>>,
}
