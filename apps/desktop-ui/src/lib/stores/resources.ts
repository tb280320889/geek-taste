import { writable, derived } from "svelte/store";
import type { ResourceCardDto, ResourceListRequest } from "$lib/types";
import { listResources, searchResources, curateResource } from "$lib/ipc/tauri";

// ── State ──────────────────────────────────────────────

type ResourcesState = {
  items: ResourceCardDto[];
  loading: boolean;
  error: string | null;
  filters: ResourceListRequest;
  availableTags: string[];
};

const initialState: ResourcesState = {
  items: [],
  loading: false,
  error: null,
  filters: {},
  availableTags: [],
};

// ── Store ──────────────────────────────────────────────

const _state = writable<ResourcesState>(initialState);

export const resources = derived(_state, ($s) => $s.items);
export const resourcesLoading = derived(_state, ($s) => $s.loading);
export const resourcesError = derived(_state, ($s) => $s.error);
export const resourceFilters = derived(_state, ($s) => $s.filters);
export const availableResourceTags = derived(_state, ($s) => $s.availableTags);

// ── Actions ──────────────────────────────────────────────

export async function loadResources(): Promise<void> {
  _state.update((s) => ({ ...s, loading: true, error: null }));
  try {
    const items = await listResources(50, 0);
    const tags = extractAllTags(items);
    _state.update((s) => ({ ...s, items, loading: false, availableTags: tags }));
  } catch (e) {
    _state.update((s) => ({ ...s, loading: false, error: String(e) }));
  }
}

export async function filterResources(filters: ResourceListRequest): Promise<void> {
  _state.update((s) => ({ ...s, loading: true, error: null, filters }));
  try {
    const items = await searchResources(filters);
    _state.update((s) => ({ ...s, items, loading: false }));
  } catch (e) {
    _state.update((s) => ({ ...s, loading: false, error: String(e) }));
  }
}

export async function toggleCurate(resourceId: string, isCurated: boolean): Promise<void> {
  try {
    const updated = await curateResource({
      resource_id: resourceId,
      action: isCurated ? "remove" : "add",
    });
    _state.update((s) => ({
      ...s,
      items: s.items.map((item) =>
        item.resource_id === resourceId ? updated : item,
      ),
    }));
  } catch (e) {
    _state.update((s) => ({ ...s, error: String(e) }));
  }
}

export function clearFilters(): void {
  _state.update((s) => ({ ...s, filters: {} }));
  void loadResources();
}

// ── Helpers ──────────────────────────────────────────────

function extractAllTags(items: ResourceCardDto[]): string[] {
  const tagSet = new Set<string>();
  for (const item of items) {
    for (const l of item.languages) tagSet.add(l);
    for (const t of item.framework_tags) tagSet.add(t);
    for (const t of item.agent_tags) tagSet.add(t);
  }
  return Array.from(tagSet).sort();
}
