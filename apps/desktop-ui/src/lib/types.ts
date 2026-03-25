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
  repo_id: number;
  full_name: string;
  description: string | null;
  stargazers_count: number;
  forks_count: number;
  language: string | null;
  topics: string[];
  html_url: string;
};

// -- TopK types (Phase 2) -----------------------------

export type FiltersDto = {
  language: string[];
  exclude_archived: boolean;
  exclude_forks: boolean;
  min_stars: number | null;
  updated_since_days: number | null;
  topic: string[];
};

export type RankingViewSpecDto = {
  ranking_view_id: string;
  name: string;
  view_kind: string;
  filters: FiltersDto;
  ranking_mode: string;
  k_value: number;
  is_pinned: boolean;
  created_at: string;
};

export type ScoreBreakdownDto = {
  star_delta: number;
  fork_delta: number;
  updated_recency: number;
};

export type RankingItemDto = {
  repo_id: number;
  full_name: string;
  html_url: string;
  description: string | null;
  primary_language: string | null;
  stars: number;
  forks: number;
  rank: number;
  score: number;
  score_breakdown: ScoreBreakdownDto | null;
  rank_change: number | null;
  is_subscribed: boolean;
};

export type CreateRankingViewRequest = {
  name: string;
  filters: FiltersDto;
  ranking_mode: string;
  k_value: number;
};

export type RankingResultDto = {
  items: RankingItemDto[];
  warmup: boolean;
};

// -- Subscription types (Phase 3) ---------------------

export type SubscriptionRowDto = {
  subscription_id: string;
  repo_id: number;
  full_name: string;
  html_url: string;
  description: string | null;
  primary_language: string | null;
  stargazers_count: number;
  state: "ACTIVE" | "PAUSED" | "ARCHIVED";
  tracking_mode: string;
  event_types: string[];
  digest_window: string;
  notify_high_immediately: boolean;
  last_successful_sync_at: string | null;
  created_at: string;
};

export type CreateSubscriptionRequest = {
  repo_id: number;
  tracking_mode?: string;
  event_types?: string[];
  digest_window?: string;
  notify_high_immediately?: boolean;
};

// -- Signal types (Phase 3) ---------------------------

export type SignalDto = {
  signal_id: string;
  signal_type: string;
  priority: "HIGH" | "MEDIUM" | "LOW";
  state: "NEW" | "SEEN" | "ACKED" | "ARCHIVED";
  source_kind: string;
  repo_id: number | null;
  full_name: string | null;
  title: string;
  summary: string | null;
  evidence: Record<string, unknown>;
  occurred_at: string;
  created_at: string;
};

export type UnreadCountsDto = {
  total: number;
  high: number;
  medium: number;
  low: number;
};

// -- Error / Sync types (Phase 5) ---------------------

export type ErrorKind =
  | "AUTH_EXPIRED"
  | "NETWORK_ERROR"
  | "RATE_LIMITED"
  | "NOT_FOUND"
  | "INTERNAL";

export type AppError = {
  code: ErrorKind;
  message: string;
};

export type SyncStatusDto = {
  is_online: boolean;
  last_topk_sync: string | null;
  last_signal_sync: string | null;
};

// -- Resource types (Phase 4) -------------------------

export type ResourceCardDto = {
  resource_id: string;
  resource_kind: string;
  title: string;
  source_repo_id: number | null;
  source_url: string;
  languages: string[];
  framework_tags: string[];
  agent_tags: string[];
  score: number;
  why_recommended: string[];
  is_curated: boolean;
  is_active: boolean;
};

export type ResourceListRequest = {
  tag_type?: string;
  tag_value?: string;
  resource_kind?: string;
  language?: string;
  limit?: number;
  offset?: number;
};

export type CurateResourceRequest = {
  resource_id: string;
  action: "add" | "remove";
};
