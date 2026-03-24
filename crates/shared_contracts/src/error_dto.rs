use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorKind {
    AuthExpired,
    NetworkError,
    RateLimited,
    NotFound,
    Internal,
}

#[derive(Debug, Clone, Serialize)]
pub struct AppErrorDto {
    pub code: ErrorKind,
    pub message: String,
}

impl std::fmt::Display for AppErrorDto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.code, self.message)
    }
}
