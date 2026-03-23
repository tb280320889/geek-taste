<script lang="ts">
  import type { RankingViewSpecDto } from "$lib/types";

  let {
    views,
    currentViewId,
    onSelect,
    onPin,
    onDelete,
  }: {
    views: RankingViewSpecDto[];
    currentViewId: string | null;
    onSelect: (viewId: string) => void;
    onPin: (viewId: string) => void;
    onDelete: (viewId: string) => void;
  } = $props();

  let open = $state(false);

  const currentName = $derived(
    views.find((v) => v.ranking_view_id === currentViewId)?.name ?? "选择视图",
  );

  const sortedViews = $derived(
    [...views].sort((a, b) => {
      if (a.is_pinned && !b.is_pinned) return -1;
      if (!a.is_pinned && b.is_pinned) return 1;
      return 0;
    }),
  );

  const handleSelect = (viewId: string): void => {
    onSelect(viewId);
    open = false;
  };

  const handleTogglePin = (e: MouseEvent, viewId: string): void => {
    e.stopPropagation();
    onPin(viewId);
  };

  const handleDelete = (e: MouseEvent, viewId: string): void => {
    e.stopPropagation();
    onDelete(viewId);
  };
</script>

<svelte:window
  onkeydown={(e) => {
    if (e.key === "Escape") open = false;
  }}
/>

<div class="view-selector">
  <button
    type="button"
    class="selector-trigger"
    onclick={() => (open = !open)}
  >
    <span class="selector-label">{currentName}</span>
    <span class="selector-arrow">{open ? "▲" : "▼"}</span>
  </button>

  {#if open}
    <button
      type="button"
      class="selector-backdrop"
      aria-label="关闭视图列表"
      onclick={() => (open = false)}
    ></button>
    <div class="selector-dropdown">
      {#if sortedViews.length === 0}
        <p class="selector-empty">暂无已保存视图</p>
      {:else}
        {#each sortedViews as view (view.ranking_view_id)}
          <div
            class="selector-item"
            class:active={view.ranking_view_id === currentViewId}
            role="button"
            tabindex="0"
            onclick={() => handleSelect(view.ranking_view_id)}
            onkeydown={(e) => {
              if (e.key === "Enter") handleSelect(view.ranking_view_id);
            }}
          >
            <span class="item-name">
              {#if view.is_pinned}📌{/if}
              {view.name}
            </span>
            <span class="item-meta">{view.ranking_mode} · k={view.k_value}</span>
            <div class="item-actions">
              <button
                type="button"
                class="action-btn"
                title={view.is_pinned ? "取消置顶" : "置顶"}
                onclick={(e) => handleTogglePin(e, view.ranking_view_id)}
              >
                {view.is_pinned ? "📌" : "📍"}
              </button>
              <button
                type="button"
                class="action-btn action-delete"
                title="删除视图"
                onclick={(e) => handleDelete(e, view.ranking_view_id)}
              >
                🗑
              </button>
            </div>
          </div>
        {/each}
      {/if}
      <div class="selector-footer">
        <span class="muted text-xs">在下方「筛选条件」中创建新视图</span>
      </div>
    </div>
  {/if}
</div>

<style>
  .view-selector {
    position: relative;
  }

  .selector-trigger {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 0.625rem 0.875rem;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--card-bg, rgba(17, 26, 44, 0.86));
    color: var(--text);
    cursor: pointer;
    font-size: 0.875rem;
  }

  .selector-trigger:hover {
    border-color: rgba(78, 201, 176, 0.3);
  }

  .selector-label {
    font-weight: 500;
  }

  .selector-arrow {
    font-size: 0.625rem;
    opacity: 0.6;
  }

  .selector-backdrop {
    position: fixed;
    inset: 0;
    z-index: 40;
    border: none;
    background: transparent;
    cursor: default;
  }

  .selector-dropdown {
    position: absolute;
    z-index: 50;
    top: calc(100% + 4px);
    left: 0;
    right: 0;
    min-width: 280px;
    border-radius: var(--radius);
    border: 1px solid var(--border);
    background: var(--card-bg, rgba(17, 26, 44, 0.95));
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
    overflow: hidden;
  }

  .selector-empty {
    padding: 1rem;
    text-align: center;
    color: var(--text-muted);
    font-size: 0.8125rem;
    margin: 0;
  }

  .selector-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    cursor: pointer;
    transition: background 0.1s;
    flex-wrap: wrap;
  }

  .selector-item:hover {
    background: rgba(78, 201, 176, 0.08);
  }

  .selector-item.active {
    background: rgba(78, 201, 176, 0.12);
  }

  .item-name {
    font-size: 0.8125rem;
    font-weight: 500;
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .item-meta {
    font-size: 0.6875rem;
    color: var(--text-muted);
    white-space: nowrap;
  }

  .item-actions {
    display: flex;
    gap: 0.25rem;
    opacity: 0;
    transition: opacity 0.15s;
  }

  .selector-item:hover .item-actions {
    opacity: 1;
  }

  .action-btn {
    padding: 0.125rem 0.375rem;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: transparent;
    cursor: pointer;
    font-size: 0.6875rem;
    line-height: 1;
  }

  .action-btn:hover {
    background: rgba(255, 255, 255, 0.06);
  }

  .action-delete:hover {
    background: rgba(239, 68, 68, 0.15);
    border-color: rgba(239, 68, 68, 0.3);
  }

  .selector-footer {
    padding: 0.5rem 0.75rem;
    border-top: 1px solid var(--border);
  }
</style>
