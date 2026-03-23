//! Signal 纯领域对象

use serde::{Deserialize, Serialize};
use thiserror::Error;

const CROCKFORD_BASE32: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

fn encode_crockford_26(mut value: u128) -> String {
    let mut out = [b'0'; 26];
    for i in (0..26).rev() {
        let idx = (value & 0x1f) as usize;
        out[i] = CROCKFORD_BASE32[idx];
        value >>= 5;
    }
    String::from_utf8(out.to_vec()).expect("crockford encoding should be valid utf8")
}

fn generate_ulid_like() -> String {
    let timestamp_ms = chrono::Utc::now().timestamp_millis().max(0) as u128;
    let nanos = chrono::Utc::now()
        .timestamp_nanos_opt()
        .unwrap_or_default()
        .unsigned_abs() as u128;
    let pid = std::process::id() as u128;
    let entropy = (nanos ^ (pid << 32)) & ((1u128 << 80) - 1);
    let ulid_bits = ((timestamp_ms & ((1u128 << 48) - 1)) << 80) | entropy;
    encode_crockford_26(ulid_bits)
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SignalError {
    #[error("invalid signal state transition: {from:?} -> {to:?}")]
    InvalidStateTransition { from: SignalState, to: SignalState },
}

/// 信号类型 — 对应 spec 04 中的 U1-U4 规则
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SignalType {
    ReleasePublished,
    ReleasePrereleased,
    TagPublished,
    DefaultBranchActivityDigest,
    PrMergedDigest,
    TopkViewChanged,
    ResourceEmerged,
    ResourceReranked,
}

/// 信号优先级
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum SignalPriority {
    High,
    Medium,
    Low,
}

impl PartialOrd for SignalPriority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SignalPriority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_weight().cmp(&other.as_weight())
    }
}

impl SignalPriority {
    fn as_weight(&self) -> u8 {
        match self {
            Self::High => 3,
            Self::Medium => 2,
            Self::Low => 1,
        }
    }
}

/// 信号状态 — 状态机: New -> Seen -> Acked, any -> Archived
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum SignalState {
    New,
    Seen,
    Acked,
    Archived,
}

impl Default for SignalState {
    fn default() -> Self {
        Self::New
    }
}

impl SignalState {
    pub fn can_transition_to(&self, next: &Self) -> bool {
        matches!(
            (self, next),
            (Self::New, Self::Seen)
                | (Self::Seen, Self::Acked)
                | (Self::New, Self::Archived)
                | (Self::Seen, Self::Archived)
                | (Self::Acked, Self::Archived)
        )
    }
}

/// 信号来源类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum SourceKind {
    Repository,
    RankingView,
    Resource,
}

/// 信号领域对象 — 对应 signals 表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    /// signal id (ULID 风格)
    pub signal_id: String,
    /// 幂等去重键 (UNIQUE)
    pub signal_key: String,
    /// 信号类型
    pub signal_type: SignalType,
    /// 来源类型
    pub source_kind: SourceKind,
    /// 关联 repo id
    pub repo_id: Option<i64>,
    /// 关联 ranking view id
    pub ranking_view_id: Option<String>,
    /// 关联 resource id
    pub resource_id: Option<String>,
    /// 优先级
    pub priority: SignalPriority,
    /// 当前状态
    pub state: SignalState,
    /// 标题
    pub title: String,
    /// 摘要
    pub summary: Option<String>,
    /// 外部事实引用 (JSON)
    pub evidence: serde_json::Value,
    /// 事件发生时间 (ISO8601)
    pub occurred_at: String,
    /// digest 桶开始时间
    pub bucket_start_at: Option<String>,
    /// digest 桶结束时间
    pub bucket_end_at: Option<String>,
    /// 创建时间 (ISO8601)
    pub created_at: String,
}

impl Signal {
    /// 创建 Release 信号 — signal_key = "{repo_id}:RELEASE_PUBLISHED:{release_id}"
    pub fn new_release(repo_id: i64, release_id: &str, title: String) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            signal_id: generate_ulid_like(),
            signal_key: format!("{}:RELEASE_PUBLISHED:{}", repo_id, release_id),
            signal_type: SignalType::ReleasePublished,
            source_kind: SourceKind::Repository,
            repo_id: Some(repo_id),
            ranking_view_id: None,
            resource_id: None,
            priority: SignalPriority::High,
            state: SignalState::default(),
            title,
            summary: None,
            evidence: serde_json::json!({
                "releaseId": release_id,
            }),
            occurred_at: now.clone(),
            bucket_start_at: None,
            bucket_end_at: None,
            created_at: now,
        }
    }

    /// 创建 Tag 信号 — signal_key = "{repo_id}:TAG_PUBLISHED:{tag_name}"
    pub fn new_tag(repo_id: i64, tag_name: &str, title: String) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            signal_id: generate_ulid_like(),
            signal_key: format!("{}:TAG_PUBLISHED:{}", repo_id, tag_name),
            signal_type: SignalType::TagPublished,
            source_kind: SourceKind::Repository,
            repo_id: Some(repo_id),
            ranking_view_id: None,
            resource_id: None,
            priority: SignalPriority::Medium,
            state: SignalState::default(),
            title,
            summary: None,
            evidence: serde_json::json!({
                "tagName": tag_name,
            }),
            occurred_at: now.clone(),
            bucket_start_at: None,
            bucket_end_at: None,
            created_at: now,
        }
    }

    /// 创建 Digest 信号 — signal_key = "{repo_id}:DEFAULT_BRANCH_ACTIVITY_DIGEST:{start}:{end}"
    pub fn new_digest(repo_id: i64, bucket_start: &str, bucket_end: &str, title: String) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            signal_id: generate_ulid_like(),
            signal_key: format!(
                "{}:DEFAULT_BRANCH_ACTIVITY_DIGEST:{}:{}",
                repo_id, bucket_start, bucket_end
            ),
            signal_type: SignalType::DefaultBranchActivityDigest,
            source_kind: SourceKind::Repository,
            repo_id: Some(repo_id),
            ranking_view_id: None,
            resource_id: None,
            priority: SignalPriority::Medium,
            state: SignalState::default(),
            title,
            summary: None,
            evidence: serde_json::json!({
                "bucketStart": bucket_start,
                "bucketEnd": bucket_end,
            }),
            occurred_at: now.clone(),
            bucket_start_at: Some(bucket_start.into()),
            bucket_end_at: Some(bucket_end.into()),
            created_at: now,
        }
    }

    pub fn transition_state(&mut self, next: SignalState) -> Result<(), SignalError> {
        let from = self.state.clone();
        if self.state.can_transition_to(&next) {
            self.state = next;
            Ok(())
        } else {
            Err(SignalError::InvalidStateTransition { from, to: next })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signal_priority_ordering() {
        assert!(SignalPriority::High > SignalPriority::Medium);
        assert!(SignalPriority::Medium > SignalPriority::Low);
    }

    #[test]
    fn signal_state_defaults_to_new() {
        assert_eq!(SignalState::default(), SignalState::New);
    }

    #[test]
    fn signal_state_machine_paths() {
        assert!(SignalState::New.can_transition_to(&SignalState::Seen));
        assert!(SignalState::Seen.can_transition_to(&SignalState::Acked));
        assert!(SignalState::Acked.can_transition_to(&SignalState::Archived));
        assert!(!SignalState::Acked.can_transition_to(&SignalState::Seen));
    }

    #[test]
    fn new_release_signal_key_format() {
        let signal = Signal::new_release(12345, "998877", "v1.0.0 released".into());
        assert_eq!(signal.signal_key, "12345:RELEASE_PUBLISHED:998877");
        assert_eq!(signal.priority, SignalPriority::High);
        assert_eq!(signal.signal_type, SignalType::ReleasePublished);
        assert_eq!(signal.signal_id.len(), 26);
    }

    #[test]
    fn new_tag_signal_key_format() {
        let signal = Signal::new_tag(12345, "v0.9.0", "New tag v0.9.0".into());
        assert_eq!(signal.signal_key, "12345:TAG_PUBLISHED:v0.9.0");
        assert_eq!(signal.priority, SignalPriority::Medium);
    }

    #[test]
    fn new_digest_signal_key_format() {
        let signal = Signal::new_digest(
            12345,
            "2026-03-23T00:00:00Z",
            "2026-03-23T12:00:00Z",
            "Activity on default branch".into(),
        );
        assert_eq!(
            signal.signal_key,
            "12345:DEFAULT_BRANCH_ACTIVITY_DIGEST:2026-03-23T00:00:00Z:2026-03-23T12:00:00Z"
        );
    }

    #[test]
    fn signal_serializes_round_trip() {
        let mut signal = Signal::new_release(1, "100", "Release".into());
        signal.signal_id = "sig_04".into();
        let json = serde_json::to_string(&signal).unwrap();
        let back: Signal = serde_json::from_str(&json).unwrap();
        assert_eq!(back.signal_id, "sig_04");
        assert_eq!(back.signal_type, SignalType::ReleasePublished);
    }

    #[test]
    fn transition_state_rejects_invalid_transition() {
        let mut signal = Signal::new_release(1, "100", "Release".into());
        signal.transition_state(SignalState::Seen).unwrap();
        signal.transition_state(SignalState::Acked).unwrap();
        let err = signal.transition_state(SignalState::Seen).unwrap_err();
        match err {
            SignalError::InvalidStateTransition { from, to } => {
                assert_eq!(from, SignalState::Acked);
                assert_eq!(to, SignalState::Seen);
            }
        }
    }
}
