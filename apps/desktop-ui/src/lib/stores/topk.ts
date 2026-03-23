import { writable, derived } from "svelte/store";
import {
  listRankingViews,
  createRankingView,
  deleteRankingView,
  togglePinRankingView,
  executeRanking,
} from "$lib/ipc/tauri";
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
    topkError.set(err instanceof Error ? err.message : String(err));
  }
};

export const selectView = async (viewId: string): Promise<void> => {
  currentViewId.set(viewId);
  topkLoading.set(true);
  topkError.set(null);
  try {
    const items = await executeRanking(viewId);
    rankingItems.set(items);
  } catch (err) {
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
    topkError.set(err instanceof Error ? err.message : String(err));
  }
};

export const refreshCurrentView = async (): Promise<void> => {
  let id: string | null = null;
  currentViewId.subscribe((v) => (id = v))();
  if (id) await selectView(id);
};
