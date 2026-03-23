<script lang="ts">
  import { open } from "@tauri-apps/plugin-shell";
  import type { RankingItemDto } from "$lib/types";
  import { subscribedRepoIds } from "$lib/stores/subscriptions";

  let {
    items,
    loading,
    error,
    onSubscribe,
  }: {
    items: RankingItemDto[];
    loading: boolean;
    error: string | null;
    onSubscribe: (repoId: number, fullName: string) => void;
  } = $props();

  const isMomentumMode = (item: RankingItemDto): boolean =>
    item.score_breakdown !== null;

  const formatScore = (score: number): string => score.toFixed(2);

  const openRepo = async (url: string): Promise<void> => {
    await open(url);
  };

  const isSubscribed = (item: RankingItemDto): boolean =>
    $subscribedRepoIds.has(item.repo_id) || item.is_subscribed;
</script>

<div class="ranking-list">
  {#if loading}
    {#each Array(5) as _, i (i)}
      <div class="skeleton-card">
        <div class="skeleton-line w-8"></div>
        <div class="skeleton-line w-48"></div>
        <div class="skeleton-line w-16"></div>
        <div class="skeleton-line w-20"></div>
      </div>
    {/each}
  {:else if items.length === 0}
    <div class="card rounded-[var(--radius)] p-6 text-center text-sm text-[color:var(--text-muted)]">
      暂无结果。选择一个视图或调整筛选条件。
    </div>
  {:else}
    {#each items as item (item.repo_id)}
      <div class="rank-card">
        <!-- Rank -->
        <span class="rank-number">#{item.rank}</span>

        <!-- Name + Description -->
        <div class="rank-info">
          <button
            type="button"
            class="rank-name"
            onclick={() => void openRepo(item.html_url)}
          >
            {item.full_name}
          </button>
          {#if item.description}
            <p class="rank-desc">{item.description}</p>
          {/if}
        </div>

        <!-- Meta: language + stars -->
        <div class="rank-meta">
          {#if item.primary_language}
            <span class="lang-badge">{item.primary_language}</span>
          {/if}
          <span class="star-count">⭐ {item.stars.toLocaleString()}</span>
        </div>

        <!-- Score (Momentum only) -->
        {#if isMomentumMode(item)}
          <div class="rank-score-wrap">
            <span class="rank-score">{formatScore(item.score)}</span>
            {#if item.score_breakdown}
              <div class="score-tooltip">
                <span>⭐ star_delta: {item.score_breakdown.star_delta.toFixed(2)}</span>
                <span>🔀 fork_delta: {item.score_breakdown.fork_delta.toFixed(2)}</span>
                <span>⏱ recency: {item.score_breakdown.updated_recency.toFixed(2)}</span>
              </div>
            {/if}
          </div>
        {/if}

        <!-- Rank Change -->
        <div class="rank-change">
          {#if item.rank_change !== null && item.rank_change > 0}
            <span class="rank-up">+↑{item.rank_change}</span>
          {:else if item.rank_change !== null && item.rank_change < 0}
            <span class="rank-down">-↓{Math.abs(item.rank_change)}</span>
          {:else if item.rank_change === 0}
            <span class="rank-same">—</span>
          {/if}
          <!-- rank_change === null: 首次快照，不显示 -->
        </div>

        <!-- Subscribe -->
        <div class="rank-action">
          {#if isSubscribed(item)}
            <button type="button" class="sub-btn subscribed" disabled>已订阅</button>
          {:else}
            <button
              type="button"
              class="sub-btn"
              onclick={() => onSubscribe(item.repo_id, item.full_name)}
            >
              订阅
            </button>
          {/if}
        </div>
      </div>
    {/each}
  {/if}
</div>

<style>
  .ranking-list {
    display: grid;
    gap: 0.5rem;
  }

  /* Skeleton */
  .skeleton-card {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.75rem 1rem;
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

  .w-8 { width: 2rem; }
  .w-16 { width: 4rem; }
  .w-20 { width: 5rem; }
  .w-48 { width: 12rem; }

  @keyframes shimmer {
    0% { background-position: 200% 0; }
    100% { background-position: -200% 0; }
  }

  /* Card */
  .rank-card {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.625rem 0.875rem;
    border-radius: var(--radius);
    border: 1px solid var(--border);
    background: var(--card-bg, rgba(17, 26, 44, 0.86));
    transition: border-color 0.15s;
  }

  .rank-card:hover {
    border-color: rgba(78, 201, 176, 0.2);
  }

  /* Rank number */
  .rank-number {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--text-muted);
    min-width: 2rem;
    text-align: center;
  }

  /* Info */
  .rank-info {
    flex: 1;
    min-width: 0;
  }

  .rank-name {
    display: block;
    border: none;
    background: none;
    padding: 0;
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--text);
    cursor: pointer;
    text-align: left;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .rank-name:hover {
    color: var(--accent);
  }

  .rank-desc {
    margin: 0.125rem 0 0;
    font-size: 0.6875rem;
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 400px;
  }

  /* Meta */
  .rank-meta {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-shrink: 0;
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

  /* Score + Tooltip */
  .rank-score-wrap {
    position: relative;
    flex-shrink: 0;
  }

  .rank-score {
    font-size: 0.8125rem;
    font-weight: 600;
    color: var(--accent);
    cursor: default;
  }

  .score-tooltip {
    display: none;
    position: absolute;
    bottom: calc(100% + 6px);
    right: 0;
    z-index: 30;
    flex-direction: column;
    gap: 0.25rem;
    padding: 0.5rem 0.625rem;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: rgba(10, 16, 28, 0.95);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
    font-size: 0.6875rem;
    color: var(--text);
    white-space: nowrap;
    pointer-events: none;
  }

  .rank-score-wrap:hover .score-tooltip {
    display: flex;
  }

  /* Rank Change */
  .rank-change {
    min-width: 3rem;
    text-align: center;
    flex-shrink: 0;
  }

  .rank-up {
    font-size: 0.6875rem;
    font-weight: 600;
    color: #34d399;
  }

  .rank-down {
    font-size: 0.6875rem;
    font-weight: 600;
    color: #f87171;
  }

  .rank-same {
    font-size: 0.6875rem;
    color: var(--text-muted);
  }

  /* Subscribe */
  .rank-action {
    flex-shrink: 0;
  }

  .sub-btn {
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

  .sub-btn:hover:not(:disabled) {
    border-color: rgba(78, 201, 176, 0.45);
    background: rgba(78, 201, 176, 0.1);
  }

  .sub-btn.subscribed {
    opacity: 0.6;
    cursor: default;
  }
</style>
