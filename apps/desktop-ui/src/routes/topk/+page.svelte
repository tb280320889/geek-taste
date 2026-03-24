<script lang="ts">
  import { onMount } from "svelte";
  import ViewSelector from "$lib/components/ViewSelector.svelte";
  import FilterPanel from "$lib/components/FilterPanel.svelte";
  import RankingList from "$lib/components/RankingList.svelte";
  import SubscribePopover from "$lib/components/SubscribePopover.svelte";
  import {
    rankingViews,
    currentViewId,
    currentView,
    rankingItems,
    topkLoading,
    topkError,
    topkWarmup,
    loadViews,
    selectView,
    addView,
    removeView,
    pinView,
  } from "$lib/stores/topk";
  import { authStatus } from "$lib/stores/auth";
  import { addSubscription, loadSubscriptions } from "$lib/stores/subscriptions";
  import type { FiltersDto } from "$lib/types";

  let showFilter = $state(false);
  let subscribeTarget = $state<{ repoId: number; fullName: string } | null>(null);

  const defaultFilters: FiltersDto = {
    language: [],
    exclude_archived: false,
    exclude_forks: false,
    min_stars: null,
    updated_since_days: null,
    topic: [],
  };

  // 初始化：加载已保存视图和订阅列表
  onMount(() => {
    if ($authStatus === "authenticated") {
      void loadViews();
      void loadSubscriptions();
    }
  });

  // 视图切换
  const handleSelectView = (viewId: string): void => {
    void selectView(viewId);
  };

  // 应用筛选 → 创建新视图
  const handleApplyFilter = (filters: FiltersDto, mode: string, k: number): void => {
    void addView({
      name: `自定义 ${new Date().toLocaleString()}`,
      filters,
      ranking_mode: mode,
      k_value: k,
    });
    showFilter = false;
  };

  // 订阅
  const handleSubscribe = (repoId: number, fullName: string): void => {
    subscribeTarget = { repoId, fullName };
  };

  // 订阅确认
  const handleSubscribeConfirm = async (_repoId: number, settings: {
    tracking_mode: string;
    digest_window: string;
    notify_high_immediately: boolean;
    event_types: string[];
  }): Promise<void> => {
    if (!subscribeTarget) return;

    await addSubscription(subscribeTarget.repoId, {
      repo_id: subscribeTarget.repoId,
      tracking_mode: settings.tracking_mode,
      digest_window: settings.digest_window,
      notify_high_immediately: settings.notify_high_immediately,
      event_types: settings.event_types,
    });

    await loadSubscriptions();
    subscribeTarget = null;
  };
</script>

<section class="grid gap-3.5">
  <header class="flex items-center justify-between">
    <div>
      <h1 class="m-0 text-2xl font-semibold">TopK 发现</h1>
      <p class="muted">按趋势发现值得关注的开源项目</p>
    </div>
    <button
      type="button"
      onclick={() => (showFilter = !showFilter)}
      class="cursor-pointer rounded-[var(--radius-sm)] border border-[color:var(--border)] bg-transparent px-3 py-1.5 text-sm"
    >
      {showFilter ? "收起筛选" : "筛选条件"}
    </button>
  </header>

  {#if $authStatus !== "authenticated"}
    <div class="card rounded-[var(--radius)] p-4">请先连接 GitHub 后再使用 TopK。</div>
  {:else}
    <!-- 视图选择器 -->
    <ViewSelector
      views={$rankingViews}
      currentViewId={$currentViewId}
      onSelect={handleSelectView}
      onPin={(id) => void pinView(id)}
      onDelete={(id) => void removeView(id)}
    />

    <!-- 筛选面板（可折叠） -->
    {#if showFilter}
      <FilterPanel
        filters={$currentView?.filters ?? defaultFilters}
        rankingMode={$currentView?.ranking_mode ?? "STARS_DESC"}
        kValue={$currentView?.k_value ?? 50}
        onApply={handleApplyFilter}
      />
    {/if}

    <!-- 错误提示 -->
    {#if $topkError}
      <div class="card rounded-[var(--radius)] p-3 text-[color:var(--danger)]">{$topkError}</div>
    {/if}

    <!-- 暖机提示 -->
    {#if $topkWarmup}
      <div class="warmup-hint">Momentum 评分将在首次快照后生效，当前按最近更新排序</div>
    {/if}

    <!-- 排名列表 -->
    <div class="relative">
      <RankingList
        items={$rankingItems}
        loading={$topkLoading}
        error={$topkError}
        onSubscribe={handleSubscribe}
      />

      <!-- 订阅 Popover -->
      {#if subscribeTarget}
        <SubscribePopover
          repoId={subscribeTarget.repoId}
          fullName={subscribeTarget.fullName}
          visible={true}
          onConfirm={handleSubscribeConfirm}
          onClose={() => (subscribeTarget = null)}
        />
      {/if}
    </div>
  {/if}
</section>
