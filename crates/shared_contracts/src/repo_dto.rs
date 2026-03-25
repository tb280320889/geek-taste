use serde::{Deserialize, Serialize};

/// 仓库基本信息 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoBasicInfo {
    pub full_name: String,
    pub description: Option<String>,
    pub stargazers_count: u64,
    pub forks_count: u64,
    pub language: Option<String>,
    pub topics: Vec<String>,
    pub html_url: String,
}
