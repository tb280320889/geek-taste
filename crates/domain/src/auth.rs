use std::fmt;

/// GitHub 用户
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct User {
    pub login: String,
    pub name: Option<String>,
    pub avatar_url: String,
    pub html_url: String,
}

/// 认证令牌（Debug 时 mask）
#[derive(Clone)]
pub struct AuthToken {
    token: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_validated: Option<chrono::DateTime<chrono::Utc>>,
}

impl AuthToken {
    pub fn new(token: String) -> Self {
        Self {
            token,
            created_at: chrono::Utc::now(),
            last_validated: None,
        }
    }

    pub fn as_str(&self) -> &str {
        &self.token
    }

    pub fn mark_validated(&mut self) {
        self.last_validated = Some(chrono::Utc::now());
    }
}

impl fmt::Debug for AuthToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AuthToken")
            .field("token", &"***MASKED***")
            .field("created_at", &self.created_at)
            .field("last_validated", &self.last_validated)
            .finish()
    }
}

/// 认证错误
#[derive(Debug, Clone, thiserror::Error)]
pub enum AuthError {
    #[error("Token 无效 (401)")]
    InvalidToken,
    #[error("API 速率限制 (403)")]
    RateLimited,
    #[error("网络连接失败: {0}")]
    NetworkError(String),
    #[error("未知错误: {0}")]
    Unknown(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_token_debug_masks_token() {
        let token = AuthToken::new("ghp_abc123".to_string());
        let debug = format!("{:?}", token);
        assert!(debug.contains("MASKED"));
        assert!(!debug.contains("ghp_abc123"));
    }

    #[test]
    fn auth_token_as_str() {
        let token = AuthToken::new("ghp_abc123".to_string());
        assert_eq!(token.as_str(), "ghp_abc123");
    }
}
