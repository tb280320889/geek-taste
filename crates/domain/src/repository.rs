//! Repository / RepoSnapshot 纯领域对象

/// 仓库领域对象 — 对应 repositories 表
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Repository {
    /// GitHub repo id
    pub repo_id: i64,
    /// "owner/name" (UNIQUE)
    pub full_name: String,
    /// owner login
    pub owner: String,
    /// repo short name
    pub name: String,
    /// source url
    pub html_url: String,
    /// repo description
    pub description: Option<String>,
    /// default branch
    pub default_branch: String,
    /// primary language
    pub primary_language: Option<String>,
    /// topic tags
    pub topics: Vec<String>,
    /// is archived
    pub archived: bool,
    /// is disabled
    pub disabled: bool,
    /// latest star count
    pub stargazers_count: i64,
    /// latest fork count
    pub forks_count: i64,
    /// last GitHub update time (ISO8601)
    pub updated_at: String,
    /// last push time (ISO8601)
    pub pushed_at: Option<String>,
    /// last time we synced this repo (ISO8601)
    pub last_synced_at: String,
}

/// 仓库快照 — 对应 repo_snapshots 表
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RepoSnapshot {
    /// snapshot id (ULID)
    pub snapshot_id: String,
    /// GitHub repo id
    pub repo_id: i64,
    /// snapshot time (ISO8601)
    pub snapshot_at: String,
    /// star count at snapshot time
    pub stargazers_count: i64,
    /// fork count at snapshot time
    pub forks_count: i64,
    /// updated_at at snapshot time (ISO8601)
    pub updated_at: String,
    /// pushed_at at snapshot time (ISO8601)
    pub pushed_at: Option<String>,
    /// release count at snapshot time (if available)
    pub release_count: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repository_debug_clone() {
        let repo = Repository {
            repo_id: 123456,
            full_name: "owner/name".into(),
            owner: "owner".into(),
            name: "name".into(),
            html_url: "https://github.com/owner/name".into(),
            description: Some("A cool repo".into()),
            default_branch: "main".into(),
            primary_language: Some("Rust".into()),
            topics: vec!["rust".into(), "cli".into()],
            archived: false,
            disabled: false,
            stargazers_count: 100,
            forks_count: 20,
            updated_at: "2026-03-23T00:00:00Z".into(),
            pushed_at: Some("2026-03-22T12:00:00Z".into()),
            last_synced_at: "2026-03-23T06:00:00Z".into(),
        };
        let cloned = repo.clone();
        assert_eq!(cloned.repo_id, 123456);
        assert_eq!(cloned.full_name, "owner/name");
    }

    #[test]
    fn repository_serializes_to_json() {
        let repo = Repository {
            repo_id: 1,
            full_name: "o/n".into(),
            owner: "o".into(),
            name: "n".into(),
            html_url: "https://github.com/o/n".into(),
            description: None,
            default_branch: "main".into(),
            primary_language: None,
            topics: vec![],
            archived: false,
            disabled: false,
            stargazers_count: 0,
            forks_count: 0,
            updated_at: "2026-03-23T00:00:00Z".into(),
            pushed_at: None,
            last_synced_at: "2026-03-23T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&repo).unwrap();
        let back: Repository = serde_json::from_str(&json).unwrap();
        assert_eq!(back.repo_id, 1);
        assert!(back.description.is_none());
        assert!(back.topics.is_empty());
    }

    #[test]
    fn repo_snapshot_round_trip() {
        let snap = RepoSnapshot {
            snapshot_id: "01HZ".into(),
            repo_id: 42,
            snapshot_at: "2026-03-23T06:00:00Z".into(),
            stargazers_count: 500,
            forks_count: 50,
            updated_at: "2026-03-22T18:00:00Z".into(),
            pushed_at: Some("2026-03-22T18:00:00Z".into()),
            release_count: Some(10),
        };
        let json = serde_json::to_string(&snap).unwrap();
        let back: RepoSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(back.snapshot_id, "01HZ");
        assert_eq!(back.release_count, Some(10));
    }
}
