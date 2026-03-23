use std::fmt;

/// GitHub 用户
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct User {
    pub login: String,
    pub name: Option<String>,
    pub avatar_url: String,
    pub html_url: String,
}

impl User {
    /// 从 GitHub API 响应字段构建 User
    pub fn from_github_response(
        login: impl Into<String>,
        name: Option<String>,
        avatar_url: impl Into<String>,
        html_url: impl Into<String>,
    ) -> Self {
        Self {
            login: login.into(),
            name,
            avatar_url: avatar_url.into(),
            html_url: html_url.into(),
        }
    }
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

    /// 检查 token 是否需要重新验证
    /// v1 简化: 启动时验证一次，24h 内视为有效
    pub fn is_expired(&self) -> bool {
        match self.last_validated {
            Some(last) => {
                let elapsed = chrono::Utc::now() - last;
                elapsed.num_hours() >= 24
            }
            None => true, // 从未验证，视为过期
        }
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

    #[test]
    fn auth_token_new_is_expired_never_validated() {
        let token = AuthToken::new("ghp_abc123".to_string());
        assert!(token.is_expired(), "从未验证的 token 应视为过期");
    }

    #[test]
    fn auth_token_recently_validated_not_expired() {
        let mut token = AuthToken::new("ghp_abc123".to_string());
        token.mark_validated();
        assert!(!token.is_expired(), "刚验证的 token 不应过期");
    }

    #[test]
    fn user_from_github_response() {
        let user = User::from_github_response(
            "octocat",
            Some("The Octocat".into()),
            "https://avatars.githubusercontent.com/u/1",
            "https://github.com/octocat",
        );
        assert_eq!(user.login, "octocat");
        assert_eq!(user.name, Some("The Octocat".into()));
    }
}
