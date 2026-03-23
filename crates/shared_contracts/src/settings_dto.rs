use serde::{Deserialize, Serialize};

/// 通知频率 DTO
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationFrequencyDto {
    Realtime,
    Digest12h,
    Digest24h,
    Muted,
}

/// 安静时段 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuietHoursDto {
    pub start: String,
    pub end: String,
}

/// 设置 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsDto {
    pub notification_frequency: NotificationFrequencyDto,
    pub language_interests: Vec<String>,
    pub quiet_hours: Option<QuietHoursDto>,
}

impl From<domain::settings::Settings> for SettingsDto {
    fn from(s: domain::settings::Settings) -> Self {
        Self {
            notification_frequency: match s.notification_frequency {
                domain::settings::NotificationFrequency::Realtime => {
                    NotificationFrequencyDto::Realtime
                }
                domain::settings::NotificationFrequency::Digest12h => {
                    NotificationFrequencyDto::Digest12h
                }
                domain::settings::NotificationFrequency::Digest24h => {
                    NotificationFrequencyDto::Digest24h
                }
                domain::settings::NotificationFrequency::Muted => NotificationFrequencyDto::Muted,
            },
            language_interests: s.language_interests,
            quiet_hours: s.quiet_hours.map(|q| QuietHoursDto {
                start: q.start,
                end: q.end,
            }),
        }
    }
}

impl From<SettingsDto> for domain::settings::Settings {
    fn from(d: SettingsDto) -> Self {
        Self {
            notification_frequency: match d.notification_frequency {
                NotificationFrequencyDto::Realtime => {
                    domain::settings::NotificationFrequency::Realtime
                }
                NotificationFrequencyDto::Digest12h => {
                    domain::settings::NotificationFrequency::Digest12h
                }
                NotificationFrequencyDto::Digest24h => {
                    domain::settings::NotificationFrequency::Digest24h
                }
                NotificationFrequencyDto::Muted => domain::settings::NotificationFrequency::Muted,
            },
            language_interests: d.language_interests,
            quiet_hours: d.quiet_hours.map(|q| domain::settings::QuietHours {
                start: q.start,
                end: q.end,
            }),
        }
    }
}

/// 更新设置请求
#[derive(Debug, Deserialize)]
pub struct UpdateSettingsRequest {
    pub notification_frequency: Option<NotificationFrequencyDto>,
    pub language_interests: Option<Vec<String>>,
    pub quiet_hours: Option<Option<QuietHoursDto>>,
}
