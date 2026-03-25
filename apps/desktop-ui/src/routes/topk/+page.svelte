<script lang="ts">
  import { onMount } from "svelte";
  import RepoInfoModal from "$lib/components/RepoInfoModal.svelte";
  import { fetchRepoInfo, openExternal } from "$lib/ipc/tauri";
  import { authStatus } from "$lib/stores/auth";
  import {
    rankingViews,
    currentViewId,
    rankingItems,
    topkLoading,
    topkError,
    topkWarmup,
    ensureDefaultViews,
    selectView,
    addView,
    removeView,
    refreshCurrentView,
  } from "$lib/stores/topk";
  import { subscribedRepoIds } from "$lib/stores/subscriptions";
  import { addSubscription } from "$lib/stores/subscriptions";
  import type { RepoBasicInfo } from "$lib/types";

  let query = $state("");
  let loading = $state(false);
  let error = $state("");
  let repoInfo = $state<RepoBasicInfo | null>(null);

  onMount(() => {
    if ($authStatus === "authenticated") {
      void ensureDefaultViews();
    }
  });

  const parseInput = (input: string): { owner: string; repo: string } | null => {
    const trimmed = input.trim();
    if (!trimmed) return null;

    const noProtocol = trimmed.replace(/^https?:\/\//, "");
    const noHost = noProtocol.replace(/^github\.com\//, "");
    const normalized = noHost.replace(/\/$/, "");
    const parts = normalized.split("/").filter(Boolean);

    if (parts.length < 2) return null;
    return { owner: parts[0], repo: parts[1] };
  };

  const onSearch = async (): Promise<void> => {
    error = "";
    const parsed = parseInput(query);
    if (!parsed) {
      error = "请输入 owner/repo 或 GitHub 仓库 URL";
      return;
    }

    loading = true;
    try {
      repoInfo = await fetchRepoInfo(parsed.owner, parsed.repo);
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      if (msg.includes("404")) {
        error = "未找到该仓库";
      } else if (msg.toLowerCase().includes("network")) {
        error = "无法连接 GitHub";
      } else {
        error = "查询失败，请稍后重试";
      }
    } finally {
      loading = false;
    }
  };

  const handleSubscribe = (repoId: number) => {
    void addSubscription(repoId);
  };

  const rankBadge = (change: number | null): string => {
    if (change === null || change === 0) return "";
    if (change > 0) return `↑${change}`;
    return `↓${Math.abs(change)}`;
  };

  const rankBadgeClass = (change: number | null): string => {
    if (change === null || change === 0) return "text-[color:var(--text-muted)]";
    if (change > 0) return "text-[color:var(--success)]";
    return "text-[color:var(--danger)]";
  };
</script>

<section class="grid gap-3.5">
  <header>
    <h1 class="m-0 text-2xl font-semibold">TopK 探索</h1>
    <p class="muted">发现趋势项目，快速查看仓库信息。</p>
  </header>

  {#if $authStatus !== "authenticated"}
    <div class="card rounded-[var(--radius)] p-4">请先连接 GitHub 后再查询仓库信息。</div>
  {:else}
    <!-- Quick lookup -->
    <form
      class="card rounded-[var(--radius)] p-4"
      onsubmit={(e) => {
        e.preventDefault();
        void onSearch();
      }}
    >
      <label for="repo-input" class="mb-2 block text-sm text-[color:var(--text-muted)]">快速查看</label>
      <div class="grid grid-cols-1 gap-2.5 sm:grid-cols-[1fr_auto]">
        <input
          id="repo-input"
          bind:value={query}
          class="rounded-[var(--radius-sm)] border border-[color:var(--border)] bg-[rgba(0,0,0,0.18)] px-2.5 py-2.5 text-[color:var(--text)]"
          placeholder="输入仓库名称，如 facebook/react"
          autocomplete="off"
        />
        <button
          type="submit"
          class="cursor-pointer rounded-[var(--radius-sm)] border-0 bg-linear-to-br from-[color:var(--accent)] to-[color:var(--accent-2)] px-3.5 py-2.5 font-bold text-[#02131a] disabled:cursor-not-allowed disabled:opacity-80"
          disabled={loading}>{loading ? "查询中..." : "探索"}</button
        >
      </div>
      {#if error}
        <p class="mt-2.5 text-[color:var(--danger)]">{error}</p>
      {/if}
    </form>

    <!-- View tabs -->
    {#if $rankingViews.length > 0}
      <div class="flex gap-2 overflow-x-auto pb-1">
        {#each $rankingViews as view (view.ranking_view_id)}
          <button
            class="cursor-pointer whitespace-nowrap rounded-full border px-3.5 py-1.5 text-sm transition-colors {$currentViewId === view.ranking_view_id
              ? 'border-[color:var(--accent)] bg-[rgba(var(--accent-rgb),0.12)] text-[color:var(--accent)]'
              : 'border-[color:var(--border)] bg-transparent text-[color:var(--text-muted)] hover:text-[color:var(--text)]'}"
            onclick={() => void selectView(view.ranking_view_id)}
          >
            {view.name}
          </button>
        {/each}
        <button
          class="cursor-pointer whitespace-nowrap rounded-full border border-dashed border-[color:var(--border)] bg-transparent px-3.5 py-1.5 text-sm text-[color:var(--text-muted)] hover:text-[color:var(--text)]"
          onclick={() => void refreshCurrentView()}
          title="刷新当前视图"
        >
          ⟳
        </button>
      </div>
    {/if}

    <!-- Ranking list -->
    {#if $topkError}
      <div class="card rounded-[var(--radius)] p-4 text-[color:var(--danger)]">
        {$topkError}
      </div>
    {/if}

    {#if $topkLoading}
      <div class="card rounded-[var(--radius)] p-4 muted">加载排名数据...</div>
    {:else if $rankingItems.length > 0}
      <div class="grid gap-2">
        {#each $rankingItems as item (item.repo_id)}
          <div class="card rounded-[var(--radius)] p-4">
            <div class="flex items-start gap-3">
              <span class="mt-0.5 w-8 shrink-0 text-right text-lg font-bold text-[color:var(--text-muted)]">
                #{item.rank}
              </span>
              <div class="min-w-0 flex-1">
                <div class="flex items-center gap-2">
                  <a
                    href={item.html_url}
                    class="cursor-pointer font-medium text-[color:var(--accent)] hover:underline"
                    role="button"
                    tabindex="0"
                    onclick={(e) => {
                      e.preventDefault();
                      void openExternal(item.html_url);
                    }}
                    onkeydown={(e) => {
                      if (e.key === "Enter" || e.key === " ") {
                        e.preventDefault();
                        void openExternal(item.html_url);
                      }
                    }}
                  >
                    {item.full_name}
                  </a>
                  {#if item.rank_change !== null}
                    <span class="text-xs {rankBadgeClass(item.rank_change)}">{rankBadge(item.rank_change)}</span>
                  {/if}
                </div>
                {#if item.description}
                  <p class="m-0 mt-1 text-sm muted line-clamp-2">{item.description}</p>
                {/if}
                <div class="mt-1.5 flex flex-wrap items-center gap-2 text-xs muted">
                  {#if item.primary_language}
                    <span class="rounded-full bg-[rgba(255,255,255,0.06)] px-2 py-0.5">{item.primary_language}</span>
                  {/if}
                  <span>★ {item.stars.toLocaleString()}</span>
                  <span>⑂ {item.forks.toLocaleString()}</span>
                  <span class="ml-auto font-mono text-[color:var(--accent)]">{item.score.toFixed(1)}</span>
                </div>
              </div>
              <div class="shrink-0">
                {#if $subscribedRepoIds.has(item.repo_id)}
                  <span class="rounded-full bg-[rgba(var(--accent-rgb),0.12)] px-2.5 py-1 text-xs text-[color:var(--accent)]">已订阅</span>
                {:else}
                  <button
                    class="cursor-pointer rounded-[var(--radius-sm)] border border-[color:var(--accent)] bg-transparent px-2.5 py-1.5 text-xs text-[color:var(--accent)] hover:bg-[rgba(var(--accent-rgb),0.08)]"
                    onclick={() => handleSubscribe(item.repo_id)}
                  >
                    订阅
                  </button>
                {/if}
              </div>
            </div>
          </div>
        {/each}
        {#if $topkWarmup}
          <p class="text-center text-xs muted">首次加载数据，可能较慢...</p>
        {/if}
      </div>
    {:else if $currentViewId}
      <div class="card rounded-[var(--radius)] p-4 muted">该视图暂无排名数据</div>
    {/if}
  {/if}

  <RepoInfoModal repoInfo={repoInfo} onClose={() => (repoInfo = null)} />
</section>
