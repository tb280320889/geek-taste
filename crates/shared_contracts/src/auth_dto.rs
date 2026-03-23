use serde::{Deserialize, Serialize};

/// 用户 DTO（跨层传递）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDto {
    pub login: String,
    pub name: Option<String>,
    pub avatar_url: String,
    pub html_url: String,
}

impl From<domain::auth::User> for UserDto {
    fn from(u: domain::auth::User) -> Self {
        Self {
            login: u.login,
            name: u.name,
            avatar_url: u.avatar_url,
            html_url: u.html_url,
        }
    }
}

/// 认证状态
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum AuthStatus {
    Authenticated { user: UserDto },
    Unauthenticated,
    TokenExpired,
}

/// 验证 token 请求
#[derive(Debug, Deserialize)]
pub struct ValidateTokenRequest {
    pub token: String,
}

/// 验证 token 响应
#[derive(Debug, Serialize)]
pub struct ValidateTokenResponse {
    pub success: bool,
    pub user: Option<UserDto>,
    pub error: Option<String>,
}
