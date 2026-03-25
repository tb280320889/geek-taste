<script lang="ts">
  import { onMount } from "svelte";
  import { resolve } from "$app/paths";
  import { currentUser } from "$lib/stores/auth";
  import { subscriptions, subscriptionsLoading, loadSubscriptions } from "$lib/stores/subscriptions";
  import { lastSyncedAt, isStale } from "$lib/stores/network";

  onMount(() => {
    void loadSubscriptions();
  });

  const isEmpty = $derived($subscriptions.length === 0 && !$subscriptionsLoading);
  const topkStale = $derived(isStale("topk"));
</script>

<section class="grid gap-4">
  <div class="card flex items-center gap-3.5 rounded-[var(--radius)] p-[18px]">
    <div
      class="grid h-14 w-14 place-items-center overflow-hidden rounded-full border-2 border-[rgba(106,179,255,0.4)] bg-[rgba(255,255,255,0.03)] text-[color:var(--text-muted)]"
    >
      {#if $currentUser?.avatar_url}
        <img src={$currentUser.avatar_url} alt="avatar" class="h-full w-full object-cover" />
      {:else}
        <span>GT</span>
      {/if}
    </div>
    <div>
      <h1 class="m-0 text-2xl font-semibold">
        欢迎回来，{$currentUser?.name || $currentUser?.login || "Geek"}
        {#if topkStale}
          <span class="stale-dot" title="数据可能已过期"></span>
        {/if}
      </h1>
      <p class="muted m-0 mt-1.5">
        {#if isEmpty}
          开始发现你关注的技术项目。
        {:else}
          今天先看趋势，再处理订阅信号。
        {/if}
      </p>
    </div>
  </div>

  {#if isEmpty}
    <div class="empty-state grid gap-4 place-items-center rounded-[var(--radius)] border border-dashed border-[color:var(--border)] p-8 text-center">
      <div class="text-3xl">📡</div>
      <div>
        <h2 class="m-0 text-xl font-semibold">开始追踪你的技术雷达</h2>
        <p class="muted m-0 mt-2 max-w-md">订阅你关注的 GitHub 仓库，Geek Taste 会为你聚合高价值的技术信号（Release、Tag、分支变更）。</p>
      </div>
      <div class="mt-2 flex flex-wrap justify-center gap-3">
        <a
          class="inline-block rounded-[var(--radius-sm)] bg-linear-to-br from-[color:var(--accent)] to-[color:var(--accent-2)] px-4 py-2 font-semibold text-[#02131a] no-underline transition hover:opacity-90"
          href={resolve("/topk")}
        >
          去 TopK 发现趋势项目
        </a>
        <a
          class="inline-block rounded-[var(--radius-sm)] border border-[color:var(--accent)] px-4 py-2 font-semibold text-[color:var(--accent)] no-underline transition hover:bg-[rgba(var(--accent-rgb),0.08)]"
          href={resolve("/subscriptions")}
        >
          搜索并订阅仓库
        </a>
      </div>
    </div>
  {:else}
    <div class="grid gap-3 md:grid-cols-3">
      <a
        class="card rounded-[var(--radius)] border border-[color:var(--border)] p-4 text-[color:var(--text)] no-underline transition duration-150 ease-out hover:-translate-y-0.5 hover:border-[rgba(106,179,255,0.35)]"
        href={resolve("/topk")}
      >
        <h2 class="m-0 mb-1.5 text-base font-semibold">发现趋势</h2>
        <p class="muted m-0">探索仓库并查看基础信息</p>
      </a>
      <a
        class="card rounded-[var(--radius)] border border-[color:var(--border)] p-4 text-[color:var(--text)] no-underline transition duration-150 ease-out hover:-translate-y-0.5 hover:border-[rgba(106,179,255,0.35)]"
        href={resolve("/subscriptions")}
      >
        <h2 class="m-0 mb-1.5 text-base font-semibold">管理订阅</h2>
        <p class="muted m-0">维护你关注的项目列表</p>
      </a>
      <a
        class="card rounded-[var(--radius)] border border-[color:var(--border)] p-4 text-[color:var(--text)] no-underline transition duration-150 ease-out hover:-translate-y-0.5 hover:border-[rgba(106,179,255,0.35)]"
        href={resolve("/resources")}
      >
        <h2 class="m-0 mb-1.5 text-base font-semibold">Agent 资源</h2>
        <p class="muted m-0">按技术栈发现 MCP/Skills</p>
      </a>
    </div>
  {/if}
</section>
