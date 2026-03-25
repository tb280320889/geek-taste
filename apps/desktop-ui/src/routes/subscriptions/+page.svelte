<script lang="ts">
  import { onMount } from "svelte";
  import {
    subscriptions,
    subscriptionsLoading,
    subscriptionsError,
    loadSubscriptions,
    togglePause,
    removeSubscription,
    triggerSync,
    syncInProgress,
  } from "$lib/stores/subscriptions";
  import { authStatus } from "$lib/stores/auth";

  let searchQuery = $state("");

  const filteredSubscriptions = $derived(
    searchQuery.trim()
      ? $subscriptions.filter(
          (s) =>
            s.full_name.toLowerCase().includes(searchQuery.toLowerCase()) ||
            (s.description?.toLowerCase().includes(searchQuery.toLowerCase()) ?? false),
        )
      : $subscriptions,
  );

  onMount(() => {
    if ($authStatus === "authenticated") {
      void loadSubscriptions();
    }
  });

  const handleSearch = (e: Event) => {
    const target = e.target as HTMLInputElement;
    searchQuery = target.value;
  };

  const formatTime = (iso: string | null): string => {
    if (!iso) return "从未同步";
    const d = new Date(iso);
    return d.toLocaleString();
  };
</script>

<section class="grid gap-3.5">
  <header class="flex items-center justify-between">
    <div>
      <h1 class="m-0 text-2xl font-semibold">Subscriptions</h1>
      <p class="muted">管理你订阅的 GitHub 仓库，跟踪可用更新。</p>
    </div>
    <button
      class="cursor-pointer rounded-[var(--radius-sm)] border border-[color:var(--border)] bg-transparent px-3 py-2 text-sm text-[color:var(--text)] hover:bg-[rgba(255,255,255,0.05)] disabled:cursor-not-allowed disabled:opacity-50"
      onclick={() => void triggerSync()}
      disabled={$syncInProgress}
    >
      {$syncInProgress ? "同步中..." : "同步"}
    </button>
  </header>

  {#if $authStatus !== "authenticated"}
    <div class="card rounded-[var(--radius)] p-4">请先连接 GitHub 后管理订阅。</div>
  {:else}
    <div class="card rounded-[var(--radius)] p-4">
      <input
        type="text"
        placeholder="搜索仓库..."
        value={searchQuery}
        oninput={handleSearch}
        class="w-full rounded-[var(--radius-sm)] border border-[color:var(--border)] bg-[rgba(0,0,0,0.18)] px-2.5 py-2.5 text-[color:var(--text)] placeholder:text-[color:var(--text-muted)]"
        autocomplete="off"
      />
    </div>

    {#if $subscriptionsError}
      <div class="card rounded-[var(--radius)] p-4 text-[color:var(--danger)]">
        {$subscriptionsError}
      </div>
    {/if}

    {#if $subscriptionsLoading}
      <div class="card rounded-[var(--radius)] p-4 muted">加载中...</div>
    {:else if filteredSubscriptions.length === 0}
      <div class="card rounded-[var(--radius)] p-4 muted">
        {searchQuery.trim() ? "没有匹配的订阅" : "暂无订阅，从 TopK 页面订阅仓库"}
      </div>
    {:else}
      <div class="grid gap-2.5">
        {#each filteredSubscriptions as sub (sub.subscription_id)}
          <div class="card rounded-[var(--radius)] p-4">
            <div class="flex items-start justify-between gap-3">
              <div class="min-w-0 flex-1">
                <a
                  href={sub.html_url}
                  class="text-[color:var(--accent)] hover:underline"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  {sub.full_name}
                </a>
                {#if sub.description}
                  <p class="m-0 mt-1 text-sm muted truncate">{sub.description}</p>
                {/if}
                <div class="mt-1.5 flex flex-wrap gap-1.5 text-xs muted">
                  {#if sub.primary_language}
                    <span class="rounded-full bg-[rgba(255,255,255,0.06)] px-2 py-0.5"
                      >{sub.primary_language}</span
                    >
                  {/if}
                  <span class="rounded-full bg-[rgba(255,255,255,0.06)] px-2 py-0.5"
                    >★ {sub.stargazers_count.toLocaleString()}</span
                  >
                  <span class="rounded-full bg-[rgba(255,255,255,0.06)] px-2 py-0.5"
                    >{sub.state}</span
                  >
                  <span class="rounded-full bg-[rgba(255,255,255,0.06)] px-2 py-0.5"
                    >上次同步: {formatTime(sub.last_successful_sync_at)}</span
                  >
                </div>
              </div>
              <div class="flex shrink-0 gap-1.5">
                <button
                  class="cursor-pointer rounded-[var(--radius-sm)] border border-[color:var(--border)] bg-transparent px-2.5 py-1.5 text-xs text-[color:var(--text)] hover:bg-[rgba(255,255,255,0.05)]"
                  onclick={() => void togglePause(sub.subscription_id)}
                >
                  {sub.state === "ACTIVE" ? "暂停" : "恢复"}
                </button>
                <button
                  class="cursor-pointer rounded-[var(--radius-sm)] border border-[color:var(--danger)] bg-transparent px-2.5 py-1.5 text-xs text-[color:var(--danger)] hover:bg-[rgba(255,50,50,0.1)]"
                  onclick={() => void removeSubscription(sub.subscription_id)}
                >
                  删除
                </button>
              </div>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  {/if}
</section>
