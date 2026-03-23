//! Subscription DTO — 前端展示用契约对象

use serde::{Deserialize, Serialize};

use domain::subscription::{Subscription, SubscriptionState, TrackingMode};

/// 订阅展示 DTO（含 repo 信息）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionDto {
    pub subscription_id: String,
    pub repo_id: i64,
    pub full_name: String,
    pub html_url: String,
    pub description: Option<String>,
    pub primary_language: Option<String>,
    pub stargazers_count: i64,
    pub state: String,
    pub tracking_mode: String,
    pub event_types: Vec<String>,
    pub digest_window: String,
    pub notify_high_immediately: bool,
    pub last_successful_sync_at: Option<String>,
    pub created_at: String,
}

impl From<Subscription> for SubscriptionDto {
    fn from(sub: Subscription) -> Self {
        Self {
            subscription_id: sub.subscription_id,
            repo_id: sub.repo_id,
            full_name: String::new(), // 需要 JOIN 后填充
            html_url: String::new(),
            description: None,
            primary_language: None,
            stargazers_count: 0,
            state: match sub.state {
                SubscriptionState::Active => "ACTIVE".into(),
                SubscriptionState::Paused => "PAUSED".into(),
                SubscriptionState::Archived => "ARCHIVED".into(),
            },
            tracking_mode: match sub.tracking_mode {
                TrackingMode::Standard => "STANDARD".into(),
                TrackingMode::Advanced => "ADVANCED".into(),
            },
            event_types: sub.event_types,
            digest_window: sub.digest_window,
            notify_high_immediately: sub.notify_high_immediately,
            last_successful_sync_at: sub.last_successful_sync_at,
            created_at: sub.created_at,
        }
    }
}

/// 创建订阅请求 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSubscriptionRequest {
    pub repo_id: i64,
    pub tracking_mode: Option<String>,
    pub event_types: Option<Vec<String>>,
    pub digest_window: Option<String>,
    pub notify_high_immediately: Option<bool>,
}

/// 更新订阅请求 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSubscriptionRequest {
    pub state: Option<String>,
    pub event_types: Option<Vec<String>>,
    pub digest_window: Option<String>,
    pub notify_high_immediately: Option<bool>,
}

/// 订阅行 DTO（list_subscriptions JOIN 结果）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionRowDto {
    pub subscription_id: String,
    pub repo_id: i64,
    pub full_name: String,
    pub html_url: String,
    pub description: Option<String>,
    pub primary_language: Option<String>,
    pub stargazers_count: i64,
    pub state: String,
    pub tracking_mode: String,
    pub event_types: Vec<String>,
    pub digest_window: String,
    pub notify_high_immediately: bool,
    pub last_successful_sync_at: Option<String>,
    pub created_at: String,
}
