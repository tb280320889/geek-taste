/// 通知频率
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationFrequency {
    Realtime,
    Digest12h,
    Digest24h,
    Muted,
}

impl Default for NotificationFrequency {
    fn default() -> Self {
        Self::Digest12h
    }
}

/// 安静时段
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QuietHours {
    pub start: String, // "HH:MM"
    pub end: String,   // "HH:MM"
}

/// 用户设置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Settings {
    pub notification_frequency: NotificationFrequency,
    pub language_interests: Vec<String>,
    pub quiet_hours: Option<QuietHours>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            notification_frequency: NotificationFrequency::default(),
            language_interests: Vec::new(),
            quiet_hours: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings() {
        let s = Settings::default();
        assert_eq!(s.notification_frequency, NotificationFrequency::Digest12h);
        assert!(s.language_interests.is_empty());
        assert!(s.quiet_hours.is_none());
    }

    #[test]
    fn settings_serde_roundtrip() {
        let settings = Settings {
            notification_frequency: NotificationFrequency::Realtime,
            language_interests: vec!["Rust".into(), "TypeScript".into()],
            quiet_hours: Some(QuietHours {
                start: "22:00".into(),
                end: "08:00".into(),
            }),
        };
        let json = serde_json::to_string(&settings).unwrap();
        let back: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(back.notification_frequency, NotificationFrequency::Realtime);
        assert_eq!(back.language_interests.len(), 2);
    }
}
