export type UserDto = {
  login: string;
  name: string | null;
  avatar_url: string;
  html_url: string;
};

export type ValidateTokenResponse = {
  success: boolean;
  user: UserDto | null;
  error: string | null;
};

export type NotificationFrequency = "realtime" | "digest12h" | "digest24h" | "muted";

export type QuietHoursDto = {
  start: string;
  end: string;
};

export type SettingsDto = {
  notification_frequency: NotificationFrequency;
  language_interests: string[];
  quiet_hours: QuietHoursDto | null;
};

export type UpdateSettingsRequest = {
  notification_frequency?: NotificationFrequency;
  language_interests?: string[];
  quiet_hours?: QuietHoursDto | null;
};

export type RepoBasicInfo = {
  full_name: string;
  description: string | null;
  stargazers_count: number;
  forks_count: number;
  language: string | null;
  topics: string[];
  html_url: string;
};
