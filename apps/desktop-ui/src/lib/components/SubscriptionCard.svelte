<script lang="ts">
  import { open } from "@tauri-apps/plugin-shell";
  import type { SubscriptionRowDto } from "$lib/types";

  let {
    subscription,
    onTogglePause,
    onDelete,
  }: {
    subscription: SubscriptionRowDto;
    onTogglePause: (subscriptionId: string) => void;
    onDelete: (subscriptionId: string) => void;
  } = $props();

  let confirmDelete = $state(false);
  let confirmTimer: ReturnType<typeof setTimeout> | null = null;

  const isActive = $derived(subscription.state === "ACTIVE");

  const stateLabel = $derived(
    subscription.state === "ACTIVE"
      ? "活跃"
      : subscription.state === "PAUSED"
        ? "已暂停"
        : "已归档",
  );

  const formatSyncTime = (iso: string | null): string => {
    if (!iso) return "从未同步";
    const d = new Date(iso);
    const now = Date.now();
    const diff = now - d.getTime();
    const mins = Math.floor(diff / 60_000);
    if (mins < 1) return "刚刚";
    if (mins < 60) return `${mins} 分钟前`;
    const hours = Math.floor(mins / 60);
    if (hours < 24) return `${hours} 小时前`;
    const days = Math.floor(hours / 24);
    return `${days} 天前`;
  };

  const openRepo = async (): Promise<void> => {
    await open(subscription.html_url);
  };

  const handleDeleteClick = (): void => {
    if (confirmDelete) {
      onDelete(subscription.subscription_id);
      confirmDelete = false;
    } else {
      confirmDelete = true;
      if (confirmTimer) clearTimeout(confirmTimer);
      confirmTimer = setTimeout(() => {
        confirmDelete = false;
      }, 3000);
    }
  };
</script>

<div class="sub-card">
  <!-- Header: name + state badge -->
  <div class="card-header">
    <button
      type="button"
      class="repo-name"
      onclick={() => void openRepo()}
    >
      {subscription.full_name}
    </button>
    <span
      class="state-badge"
      class:active={isActive}
      class:paused={subscription.state === "PAUSED"}
    >
      {stateLabel}
    </span>
  </div>

  <!-- Description -->
  {#if subscription.description}
    <p class="card-desc">{subscription.description}</p>
  {/if}

  <!-- Meta row -->
  <div class="card-meta">
    {#if subscription.primary_language}
      <span class="lang-badge">{subscription.primary_language}</span>
    {/if}
    <span class="star-count">⭐ {subscription.stargazers_count.toLocaleString()}</span>
    <span class="sync-time">⏱ {formatSyncTime(subscription.last_successful_sync_at)}</span>
  </div>

  <!-- Actions -->
  <div class="card-actions">
    <button
      type="button"
      class="action-btn pause-btn"
      onclick={() => onTogglePause(subscription.subscription_id)}
    >
      {isActive ? "⏸ 暂停" : "▶ 恢复"}
    </button>
    <button
      type="button"
      class="action-btn delete-btn"
      class:confirm={confirmDelete}
      onclick={handleDeleteClick}
    >
      {confirmDelete ? "确认删除?" : "🗑 删除"}
    </button>
  </div>
</div>

<style>
  .sub-card {
    display: grid;
    gap: 0.375rem;
    padding: 0.75rem 0.875rem;
    border-radius: var(--radius);
    border: 1px solid var(--border);
    background: var(--card-bg, rgba(17, 26, 44, 0.86));
    transition: border-color 0.15s;
  }

  .sub-card:hover {
    border-color: rgba(78, 201, 176, 0.2);
  }

  .card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
  }

  .repo-name {
    display: block;
    border: none;
    background: none;
    padding: 0;
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text);
    cursor: pointer;
    text-align: left;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .repo-name:hover {
    color: var(--accent);
  }

  .state-badge {
    border-radius: 9999px;
    padding: 0.125rem 0.5rem;
    font-size: 0.625rem;
    font-weight: 500;
    white-space: nowrap;
    flex-shrink: 0;
    border: 1px solid var(--border);
    background: rgba(255, 255, 255, 0.04);
    color: var(--text-muted);
  }

  .state-badge.active {
    border-color: rgba(52, 211, 153, 0.4);
    background: rgba(52, 211, 153, 0.12);
    color: #34d399;
  }

  .state-badge.paused {
    border-color: rgba(251, 191, 36, 0.4);
    background: rgba(251, 191, 36, 0.12);
    color: #fbbf24;
  }

  .card-desc {
    margin: 0;
    font-size: 0.75rem;
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    line-height: 1.4;
  }

  .card-meta {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  .lang-badge {
    border-radius: 9999px;
    border: 1px solid rgba(106, 179, 255, 0.4);
    background: rgba(106, 179, 255, 0.16);
    padding: 0.125rem 0.5rem;
    font-size: 0.625rem;
    color: var(--text);
    white-space: nowrap;
  }

  .star-count {
    font-size: 0.6875rem;
    color: var(--text-muted);
    white-space: nowrap;
  }

  .sync-time {
    font-size: 0.6875rem;
    color: var(--text-muted);
    white-space: nowrap;
  }

  .card-actions {
    display: flex;
    gap: 0.5rem;
    margin-top: 0.125rem;
  }

  .action-btn {
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: transparent;
    padding: 0.3125rem 0.75rem;
    font-size: 0.6875rem;
    font-weight: 500;
    color: var(--text);
    cursor: pointer;
    white-space: nowrap;
    transition: all 0.1s;
  }

  .pause-btn:hover {
    border-color: rgba(78, 201, 176, 0.45);
    background: rgba(78, 201, 176, 0.1);
  }

  .delete-btn:hover {
    border-color: rgba(239, 68, 68, 0.45);
    background: rgba(239, 68, 68, 0.1);
    color: #f87171;
  }

  .delete-btn.confirm {
    border-color: rgba(239, 68, 68, 0.6);
    background: rgba(239, 68, 68, 0.2);
    color: #f87171;
  }
</style>
