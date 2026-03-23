import { invoke } from "@tauri-apps/api/core";
import type {
  CreateRankingViewRequest,
  RankingItemDto,
  RankingViewSpecDto,
  RepoBasicInfo,
  SettingsDto,
  UpdateSettingsRequest,
  UserDto,
  ValidateTokenResponse,
} from "$lib/types";

export const validateToken = async (token: string): Promise<ValidateTokenResponse> =>
  invoke("validate_github_token", { token });

export const storeToken = async (token: string): Promise<void> =>
  invoke("store_github_token", { token });

export const loadToken = async (): Promise<string> => invoke("load_github_token");

export const removeToken = async (): Promise<void> => invoke("remove_github_token");

export const getCurrentUser = async (): Promise<UserDto | null> => invoke("get_current_user");

export const fetchRepoInfo = async (owner: string, repo: string): Promise<RepoBasicInfo> =>
  invoke("fetch_repo_info", { owner, repo });

export const getSettings = async (): Promise<SettingsDto> => invoke("get_settings");

export const updateSettings = async (settings: UpdateSettingsRequest): Promise<SettingsDto> =>
  invoke("update_settings", { settings });

// ── TopK IPC (Phase 2) ──────────────────────────────

export const listRankingViews = async (): Promise<RankingViewSpecDto[]> =>
  invoke("list_ranking_views");

export const createRankingView = async (
  request: CreateRankingViewRequest,
): Promise<RankingViewSpecDto> => invoke("create_ranking_view", { request });

export const deleteRankingView = async (viewId: string): Promise<void> =>
  invoke("delete_ranking_view", { viewId });

export const togglePinRankingView = async (viewId: string): Promise<void> =>
  invoke("toggle_pin_ranking_view", { viewId });

export const executeRanking = async (viewId: string): Promise<RankingItemDto[]> =>
  invoke("execute_ranking", { viewId });
