<script lang="ts">
  import { onMount } from "svelte";
  import SubscriptionSearch from "$lib/components/SubscriptionSearch.svelte";
  import SubscriptionCard from "$lib/components/SubscriptionCard.svelte";
  import {
    subscriptions,
    subscriptionsLoading,
    subscriptionsError,
    syncInProgress,
    loadSubscriptions,
    addSubscription,
    removeSubscription,
    togglePause,
    triggerSync,
  } from "$lib/stores/subscriptions";
  import { authStatus } from "$lib/stores/auth";

  onMount(() => {
    if ($authStatus === "authenticated") {
      void loadSubscriptions();
    }
  });

  const handleTogglePause = (subscriptionId: string): void => {
    void togglePause(subscriptionId);
  };

  const handleDelete = (subscriptionId: string): void => {
    void removeSubscription(subscriptionId);
  };

  const handleSync = (): void => {
    void triggerSync();
  };

  const handleSubscribe = async (repoId: number): Promise<void> => {
    await addSubscription(repoId, { repo_id: repoId });
  };
</script>

<section class="grid gap-3.5">
  <header class="flex items-center justify-between">
    <div>
      <h1 class="m-0 text-2xl font-semibold">订阅管理</h1>
      <p class="muted">跟踪你关注的开源项目动态</p>
    </div>
    <button
      type="button"
      class="sync-btn"
      onclick={handleSync}
      disabled={$syncInProgress}
    >
      {$syncInProgress ? "同步中..." : "🔄 同步"}
    </button>
  </header>

  {#if $authStatus !== "authenticated"}
    <div class="card rounded-[var(--radius)] p-4">请先连接 GitHub 后再管理订阅。</div>
  {:else}
    <!-- 搜索查看 -->
    <SubscriptionSearch onSubscribe={handleSubscribe} />

    <!-- 错误提示 -->
    {#if $subscriptionsError}
      <div class="card rounded-[var(--radius)] p-3 text-[color:var(--danger)]">
        {$subscriptionsError}
      </div>
    {/if}

    <!-- 订阅列表 -->
    {#if $subscriptionsLoading}
      <div class="card-list">
        {#each Array(3) as _, i (i)}
          <div class="skeleton-card">
            <div class="skeleton-line w-48"></div>
            <div class="skeleton-line w-64"></div>
            <div class="skeleton-line w-32"></div>
          </div>
        {/each}
      </div>
    {:else if $subscriptions.length === 0}
      <div class="empty-state">
        <span class="empty-icon">📡</span>
        <p class="empty-title">还没有订阅</p>
        <p class="empty-desc">在 TopK 中发现感兴趣的项目，点击订阅按钮开始跟踪</p>
      </div>
    {:else}
      <div class="card-list">
        {#each $subscriptions as sub (sub.subscription_id)}
          <SubscriptionCard
            subscription={sub}
            onTogglePause={handleTogglePause}
            onDelete={handleDelete}
          />
        {/each}
      </div>
    {/if}
  {/if}
</section>

<style>
  .sync-btn {
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: transparent;
    padding: 0.375rem 0.875rem;
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--text);
    cursor: pointer;
    transition: all 0.1s;
  }

  .sync-btn:hover:not(:disabled) {
    border-color: rgba(78, 201, 176, 0.45);
    background: rgba(78, 201, 176, 0.1);
  }

  .sync-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .card-list {
    display: grid;
    gap: 0.5rem;
  }

  /* Skeleton */
  .skeleton-card {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 0.75rem 0.875rem;
    border-radius: var(--radius);
    border: 1px solid var(--border);
    background: var(--card-bg, rgba(17, 26, 44, 0.86));
  }

  .skeleton-line {
    height: 0.75rem;
    border-radius: 4px;
    background: linear-gradient(
      90deg,
      rgba(255, 255, 255, 0.04) 25%,
      rgba(255, 255, 255, 0.08) 50%,
      rgba(255, 255, 255, 0.04) 75%
    );
    background-size: 200% 100%;
    animation: shimmer 1.5s infinite;
  }

  .w-32 { width: 8rem; }
  .w-48 { width: 12rem; }
  .w-64 { width: 16rem; }

  @keyframes shimmer {
    0% { background-position: 200% 0; }
    100% { background-position: -200% 0; }
  }

  /* Empty state */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.375rem;
    padding: 3rem 1rem;
    text-align: center;
  }

  .empty-icon {
    font-size: 2.5rem;
    opacity: 0.5;
  }

  .empty-title {
    margin: 0;
    font-size: 0.9375rem;
    font-weight: 500;
    color: var(--text);
  }

  .empty-desc {
    margin: 0;
    font-size: 0.8125rem;
    color: var(--text-muted);
  }
</style>
