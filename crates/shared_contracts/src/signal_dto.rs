//! Signal DTO — 前端展示用契约对象 (SignalCard)

use serde::{Deserialize, Serialize};

use domain::signal::{Signal, SignalState};

/// 信号卡片 DTO（前端展示用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalDto {
    pub signal_id: String,
    pub signal_type: String,
    pub priority: String,
    pub state: String,
    pub source_kind: String,
    pub repo_id: Option<i64>,
    pub full_name: Option<String>,
    pub title: String,
    pub summary: Option<String>,
    pub evidence: serde_json::Value,
    pub occurred_at: String,
    pub created_at: String,
}

impl From<Signal> for SignalDto {
    fn from(signal: Signal) -> Self {
        Self {
            signal_id: signal.signal_id,
            signal_type: serde_json::to_string(&signal.signal_type)
                .unwrap_or_default()
                .trim_matches('"')
                .to_string(),
            priority: serde_json::to_string(&signal.priority)
                .unwrap_or_default()
                .trim_matches('"')
                .to_string(),
            state: match signal.state {
                SignalState::New => "NEW".into(),
                SignalState::Seen => "SEEN".into(),
                SignalState::Acked => "ACKED".into(),
                SignalState::Archived => "ARCHIVED".into(),
            },
            source_kind: serde_json::to_string(&signal.source_kind)
                .unwrap_or_default()
                .trim_matches('"')
                .to_string(),
            repo_id: signal.repo_id,
            full_name: None, // JOIN 后填充
            title: signal.title,
            summary: signal.summary,
            evidence: signal.evidence,
            occurred_at: signal.occurred_at,
            created_at: signal.created_at,
        }
    }
}

/// 未读计数 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnreadCountsDto {
    pub total: i64,
    pub high: i64,
    pub medium: i64,
    pub low: i64,
}
