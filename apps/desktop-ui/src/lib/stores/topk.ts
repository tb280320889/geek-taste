import { writable, derived } from "svelte/store";
import {
  listRankingViews,
  createRankingView,
  deleteRankingView,
  togglePinRankingView,
  executeRanking,
} from "$lib/ipc/tauri";
import { handleIpcError } from "$lib/stores/network";
import { settings } from "$lib/stores/settings";
import type {
  RankingViewSpecDto,
  RankingItemDto,
  CreateRankingViewRequest,
} from "$lib/types";

// ── State ──────────────────────────────────────────

export const rankingViews = writable<RankingViewSpecDto[]>([]);
export const currentViewId = writable<string | null>(null);
export const rankingItems = writable<RankingItemDto[]>([]);
export const topkLoading = writable(false);
export const topkError = writable<string | null>(null);
export const topkWarmup = writable(false);

// ── Derived ────────────────────────────────────────

export const currentView = derived(
  [rankingViews, currentViewId],
  ([$views, $id]) => $views.find((v) => v.ranking_view_id === $id) ?? null,
);

export const pinnedViews = derived(rankingViews, ($views) =>
  $views.filter((v) => v.is_pinned),
);

// ── Actions ────────────────────────────────────────

export const loadViews = async (): Promise<void> => {
  try {
    const views = await listRankingViews();
    rankingViews.set(views);
  } catch (err) {
    handleIpcError(err);
    topkError.set(err instanceof Error ? err.message : String(err));
  }
};

export const selectView = async (viewId: string): Promise<void> => {
  currentViewId.set(viewId);
  topkLoading.set(true);
  topkError.set(null);
  topkWarmup.set(false);
  try {
    const result = await executeRanking(viewId);
    rankingItems.set(result.items);
    topkWarmup.set(result.warmup);
  } catch (err) {
    handleIpcError(err);
    topkError.set(err instanceof Error ? err.message : String(err));
    rankingItems.set([]);
  } finally {
    topkLoading.set(false);
  }
};

export const addView = async (request: CreateRankingViewRequest): Promise<void> => {
  try {
    const view = await createRankingView(request);
    rankingViews.update((views) => [...views, view]);
    await selectView(view.ranking_view_id);
  } catch (err) {
    handleIpcError(err);
    topkError.set(err instanceof Error ? err.message : String(err));
  }
};

export const removeView = async (viewId: string): Promise<void> => {
  try {
    await deleteRankingView(viewId);
    rankingViews.update((views) =>
      views.filter((v) => v.ranking_view_id !== viewId),
    );
    currentViewId.update((id) => (id === viewId ? null : id));
    rankingItems.set([]);
  } catch (err) {
    handleIpcError(err);
    topkError.set(err instanceof Error ? err.message : String(err));
  }
};

export const pinView = async (viewId: string): Promise<void> => {
  try {
    await togglePinRankingView(viewId);
    rankingViews.update((views) =>
      views.map((v) =>
        v.ranking_view_id === viewId ? { ...v, is_pinned: !v.is_pinned } : v,
      ),
    );
  } catch (err) {
    handleIpcError(err);
    topkError.set(err instanceof Error ? err.message : String(err));
  }
};

export const refreshCurrentView = async (): Promise<void> => {
  let id: string | null = null;
  currentViewId.subscribe((v) => (id = v))();
  if (id) await selectView(id);
};

const DEFAULT_VIEWS: CreateRankingViewRequest[] = [
  {
    name: "Trending",
    filters: {
      language: [],
      exclude_archived: true,
      exclude_forks: false,
      min_stars: null,
      updated_since_days: 7,
      topic: [],
    },
    ranking_mode: "STARS_DESC",
    k_value: 25,
  },
  {
    name: "Most Starred",
    filters: {
      language: [],
      exclude_archived: true,
      exclude_forks: true,
      min_stars: 1000,
      updated_since_days: null,
      topic: [],
    },
    ranking_mode: "STARS_DESC",
    k_value: 25,
  },
  {
    name: "Recently Updated",
    filters: {
      language: [],
      exclude_archived: true,
      exclude_forks: false,
      min_stars: null,
      updated_since_days: 30,
      topic: [],
    },
    ranking_mode: "UPDATED_DESC",
    k_value: 25,
  },
];

export const ensureDefaultViews = async (): Promise<void> => {
  await loadViews();
  let views: RankingViewSpecDto[] = [];
  rankingViews.subscribe((v) => (views = v))();
  if (views.length > 0) {
    await selectView(views[0].ranking_view_id);
    return;
  }
  // API 关闭时，首次只创建一个默认视图，避免空页面且不触发大量远程请求
  let apiEnabled = true;
  settings.subscribe((s) => (apiEnabled = s.github_api_enabled))();
  const presets = apiEnabled ? DEFAULT_VIEWS : [DEFAULT_VIEWS[0]];

  for (const preset of presets) {
    try {
      const view = await createRankingView(preset);
      rankingViews.update((v) => [...v, view]);
    } catch (err) {
      handleIpcError(err);
    }
  }
  let created: RankingViewSpecDto[] = [];
  rankingViews.subscribe((v) => (created = v))();
  if (created.length > 0) {
    await selectView(created[0].ranking_view_id);
  }
};
