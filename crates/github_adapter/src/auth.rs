use domain::auth::{AuthError, User};
use shared_contracts::repo_dto::RepoBasicInfo;

/// 使用 token 验证 GitHub 认证
pub async fn validate_token(token: &str) -> Result<User, AuthError> {
    let octocrab = octocrab::Octocrab::builder()
        .personal_token(token.to_string())
        .build()
        .map_err(|e| AuthError::Unknown(e.to_string()))?;

    match octocrab.current().user().await {
        Ok(user) => Ok(User {
            login: user.login.to_string(),
            name: user.name,
            avatar_url: user.avatar_url.to_string(),
            html_url: user.html_url.to_string(),
        }),
        Err(octocrab::Error::GitHub { source, .. }) => {
            match source.status_code.as_u16() {
                401 => Err(AuthError::InvalidToken),
                403 => Err(AuthError::RateLimited),
                _ => Err(AuthError::Unknown(source.message)),
            }
        }
        Err(e) => Err(AuthError::NetworkError(e.to_string())),
    }
}

/// 获取仓库基本信息
pub async fn fetch_repo_info(
    token: &str,
    owner: &str,
    repo: &str,
) -> Result<RepoBasicInfo, String> {
    let octocrab = octocrab::Octocrab::builder()
        .personal_token(token.to_string())
        .build()
        .map_err(|e| e.to_string())?;

    let repo_data = octocrab
        .repos(owner, repo)
        .get()
        .await
        .map_err(|e| e.to_string())?;

    Ok(RepoBasicInfo {
        full_name: repo_data.full_name.unwrap_or_else(|| format!("{}/{}", owner, repo)),
        description: repo_data.description,
        stargazers_count: repo_data.stargazers_count.unwrap_or(0) as u64,
        forks_count: repo_data.forks_count.unwrap_or(0) as u64,
        language: repo_data.language.map(|l| l.to_string()),
        topics: repo_data.topics.unwrap_or_default(),
        html_url: repo_data.html_url.map(|u| u.to_string()).unwrap_or_default(),
    })
}
