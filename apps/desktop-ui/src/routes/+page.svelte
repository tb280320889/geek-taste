<script lang="ts">
  import { resolve } from "$app/paths";
  import { onMount } from "svelte";
  import {
    homeSignals,
    unreadCounts,
    signalsLoading,
    loadHomeSignals,
    ackSignalAction,
    markSeenAction,
  } from "$lib/stores/signals";
  import {
    activeSubscriptions,
    loadSubscriptions,
  } from "$lib/stores/subscriptions";
  import { isStale, isOnline } from "$lib/stores/network";
  import SignalCard from "$lib/components/SignalCard.svelte";

  const totalUnread = $derived($unreadCounts.total);
  const highPrioritySignals = $derived($homeSignals.filter((s) => s.priority === "HIGH"));
  const normalPrioritySignals = $derived($homeSignals.filter((s) => s.priority !== "HIGH"));

  // STALE 状态检测
  const signalsStale = $derived(!$isOnline || isStale("signals"));
  const topkStale = $derived(!$isOnline || isStale("topk"));

  onMount(() => {
    void loadHomeSignals();
    void loadSubscriptions();
  });

  const handleAck = (id: string): void => {
    void ackSignalAction(id);
  };

  const handleSeen = (id: string): void => {
    void markSeenAction(id);
  };

</script>

<section class="grid gap-4">
  <!-- Title -->
  <header class="flex items-center gap-2.5">
    <h1 class="m-0 text-2xl font-semibold">Today</h1>
    {#if totalUnread > 0}
      <span class="unread-badge">{totalUnread}</span>
    {/if}
    {#if signalsStale}
      <span class="stale-dot" title="信号数据已过期"></span>
    {/if}
  </header>

  <!-- Signal Feed -->
  {#if $signalsLoading && $homeSignals.length === 0}
    {#each Array(3) as _, i (i)}
      <div class="skeleton-card">
        <div class="skeleton-bar"></div>
        <div class="skeleton-body">
          <div class="skeleton-line w-16"></div>
          <div class="skeleton-line w-48"></div>
          <div class="skeleton-line w-32"></div>
        </div>
      </div>
    {/each}
  {:else if $homeSignals.length === 0}
    <div class="empty-state">
      <p class="empty-title">暂无新信号</p>
      <p class="empty-desc">订阅你感兴趣的项目，开始接收更新信号</p>
      <div class="empty-actions">
        <a class="empty-link" href={resolve("/topk")}>发现趋势</a>
        <a class="empty-link" href={resolve("/subscriptions")}>管理订阅</a>
      </div>
    </div>
  {:else}
    {#if highPrioritySignals.length > 0}
      {#each highPrioritySignals as signal (signal.signal_id)}
        <SignalCard {signal} onAck={handleAck} onSeen={handleSeen} />
      {/each}
    {/if}

    {#each normalPrioritySignals as signal (signal.signal_id)}
      <SignalCard {signal} onAck={handleAck} onSeen={handleSeen} />
    {/each}
  {/if}

  <!-- Quick Links -->
  <div class="quick-links">
    <a class="quick-link" href={resolve("/topk")}>
      <span class="quick-link-label">
        TopK
        {#if topkStale}
          <span class="stale-dot" title="排名数据已过期"></span>
        {/if}
      </span>
      <span class="quick-link-desc">发现趋势</span>
    </a>
    <a class="quick-link" href={resolve("/subscriptions")}>
      <span class="quick-link-label">订阅</span>
      <span class="quick-link-desc">{$activeSubscriptions.length} 个活跃</span>
    </a>
    <a class="quick-link" href={resolve("/resources")}>
      <span class="quick-link-label">资源</span>
      <span class="quick-link-desc">Agent 资源</span>
    </a>
  </div>
</section>

<style>
  .unread-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 1.375rem;
    height: 1.375rem;
    border-radius: 9999px;
    background: var(--accent);
    padding: 0 0.375rem;
    font-size: 0.6875rem;
    font-weight: 700;
    color: #02131a;
  }

  .empty-state {
    border-radius: var(--radius);
    border: 1px solid var(--border);
    background: var(--card-bg, rgba(17, 26, 44, 0.86));
    padding: 2rem 1.5rem;
    text-align: center;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.375rem;
  }

  .empty-title {
    margin: 0;
    font-size: 0.9375rem;
    font-weight: 600;
    color: var(--text);
  }

  .empty-desc {
    margin: 0;
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .empty-actions {
    display: flex;
    gap: 0.5rem;
    margin-top: 0.75rem;
  }

  .empty-link {
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: transparent;
    padding: 0.375rem 0.875rem;
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--text);
    text-decoration: none;
    transition: all 0.15s;
  }

  .empty-link:hover {
    border-color: rgba(78, 201, 176, 0.35);
    background: rgba(78, 201, 176, 0.08);
  }

  .quick-links {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 0.5rem;
  }

  .quick-link {
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
    border-radius: var(--radius);
    border: 1px solid var(--border);
    background: var(--card-bg, rgba(17, 26, 44, 0.86));
    padding: 0.625rem 0.75rem;
    text-decoration: none;
    color: var(--text);
    transition: border-color 0.15s;
  }

  .quick-link:hover {
    border-color: rgba(106, 179, 255, 0.3);
  }

  .quick-link-label {
    font-size: 0.8125rem;
    font-weight: 600;
  }

  .quick-link-desc {
    font-size: 0.6875rem;
    color: var(--text-muted);
  }

  /* Skeleton */
  .skeleton-card {
    display: flex;
    align-items: stretch;
    gap: 0.75rem;
    padding: 0.625rem 0.875rem;
    border-radius: var(--radius);
    border: 1px solid var(--border);
    background: var(--card-bg, rgba(17, 26, 44, 0.86));
  }

  .skeleton-bar {
    width: 3px;
    border-radius: 2px;
    background: rgba(255, 255, 255, 0.06);
  }

  .skeleton-body {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
  }

  .skeleton-line {
    height: 0.625rem;
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

  .w-16 { width: 4rem; }
  .w-32 { width: 8rem; }
  .w-48 { width: 12rem; }

  @keyframes shimmer {
    0% { background-position: 200% 0; }
    100% { background-position: -200% 0; }
  }
</style>
