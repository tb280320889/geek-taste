//! Subscription 纯领域对象

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
    // 48-bit timestamp(ms) + 80-bit entropy-like payload
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
pub enum SubscriptionError {
    #[error("digest_window must be one of: 12h, 24h")]
    InvalidDigestWindow,
}

/// 订阅状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum SubscriptionState {
    Active,
    Paused,
    Archived,
}

impl Default for SubscriptionState {
    fn default() -> Self {
        Self::Active
    }
}

/// 跟踪模式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum TrackingMode {
    Standard,
    Advanced,
}

impl Default for TrackingMode {
    fn default() -> Self {
        Self::Standard
    }
}

/// 默认事件类型集合
pub fn default_event_types() -> Vec<String> {
    vec![
        "RELEASE_PUBLISHED".into(),
        "TAG_PUBLISHED".into(),
        "DEFAULT_BRANCH_ACTIVITY_DIGEST".into(),
    ]
}

/// 订阅领域对象 — 对应 subscriptions 表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    /// subscription id (ULID 风格)
    pub subscription_id: String,
    /// GitHub repo id (FK -> repositories)
    pub repo_id: i64,
    /// 当前状态
    pub state: SubscriptionState,
    /// 跟踪模式
    pub tracking_mode: TrackingMode,
    /// 启用的信号类型 (JSON 序列化存储)
    pub event_types: Vec<String>,
    /// digest 时间窗 ("12h" | "24h")
    pub digest_window: String,
    /// HIGH 优先级信号是否立即通知
    pub notify_high_immediately: bool,
    /// 创建时间 (ISO8601)
    pub created_at: String,
    /// 更新时间 (ISO8601)
    pub updated_at: String,
    /// 上次成功同步时间 (ISO8601)
    pub last_successful_sync_at: Option<String>,
    /// Release 游标 (上次同步到的最新 release id)
    pub cursor_release_id: Option<String>,
    /// Tag 游标 (上次同步到的最新 tag name)
    pub cursor_tag_name: Option<String>,
    /// 分支游标 (默认分支 baseline SHA)
    pub cursor_branch_sha: Option<String>,
}

impl Default for Subscription {
    fn default() -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            subscription_id: generate_ulid_like(),
            repo_id: 0,
            state: SubscriptionState::Active,
            tracking_mode: TrackingMode::Standard,
            event_types: default_event_types(),
            digest_window: "24h".into(),
            notify_high_immediately: true,
            created_at: now.clone(),
            updated_at: now,
            last_successful_sync_at: None,
            cursor_release_id: None,
            cursor_tag_name: None,
            cursor_branch_sha: None,
        }
    }
}

impl Subscription {
    /// 创建新订阅（带规范默认值）
    pub fn new(repo_id: i64) -> Self {
        let mut sub = Self::default();
        sub.repo_id = repo_id;
        sub
    }

    /// 更新 digest window，限制在规范值范围内
    pub fn set_digest_window(&mut self, digest_window: &str) -> Result<(), SubscriptionError> {
        match digest_window {
            "12h" | "24h" => {
                self.digest_window = digest_window.to_string();
                Ok(())
            }
            _ => Err(SubscriptionError::InvalidDigestWindow),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subscription_state_defaults_to_active() {
        assert_eq!(SubscriptionState::default(), SubscriptionState::Active);
    }

    #[test]
    fn tracking_mode_defaults_to_standard() {
        assert_eq!(TrackingMode::default(), TrackingMode::Standard);
    }

    #[test]
    fn default_event_types_contains_three() {
        let types = default_event_types();
        assert_eq!(types.len(), 3);
        assert!(types.contains(&"RELEASE_PUBLISHED".into()));
        assert!(types.contains(&"TAG_PUBLISHED".into()));
        assert!(types.contains(&"DEFAULT_BRANCH_ACTIVITY_DIGEST".into()));
    }

    #[test]
    fn subscription_serializes_round_trip() {
        let mut sub = Subscription::new(12345);
        sub.subscription_id = "sub_01".into();
        sub.created_at = "2026-03-23T00:00:00Z".into();
        sub.updated_at = "2026-03-23T00:00:00Z".into();
        let json = serde_json::to_string(&sub).unwrap();
        let back: Subscription = serde_json::from_str(&json).unwrap();
        assert_eq!(back.subscription_id, "sub_01");
        assert_eq!(back.repo_id, 12345);
        assert_eq!(back.state, SubscriptionState::Active);
    }

    #[test]
    fn subscription_new_applies_defaults() {
        let sub = Subscription::new(9999);
        assert_eq!(sub.repo_id, 9999);
        assert_eq!(sub.state, SubscriptionState::Active);
        assert_eq!(sub.tracking_mode, TrackingMode::Standard);
        assert_eq!(sub.digest_window, "24h");
        assert!(sub.notify_high_immediately);
        assert_eq!(sub.event_types, default_event_types());
        assert_eq!(sub.subscription_id.len(), 26);
    }

    #[test]
    fn set_digest_window_rejects_invalid_values() {
        let mut sub = Subscription::new(1);
        let err = sub.set_digest_window("48h").unwrap_err();
        assert_eq!(err, SubscriptionError::InvalidDigestWindow);
    }
}
