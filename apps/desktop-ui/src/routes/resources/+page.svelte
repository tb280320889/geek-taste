<script lang="ts">
  import { onMount } from "svelte";
  import {
    resources,
    resourcesLoading,
    resourcesError,
    availableResourceTags,
    loadResources,
    filterResources,
    toggleCurate,
    clearFilters,
  } from "$lib/stores/resources";
  import { authStatus } from "$lib/stores/auth";

  let activeTag = $state<string | null>(null);

  onMount(() => {
    void loadResources();
  });

  const handleTagFilter = (tag: string) => {
    if (activeTag === tag) {
      activeTag = null;
      void clearFilters();
    } else {
      activeTag = tag;
      void filterResources({ tag_value: tag, limit: 50 });
    }
  };

  const kindLabel = (kind: string): string => {
    switch (kind) {
      case "MCP_SERVER":
        return "MCP";
      case "SKILL":
        return "Skill";
      case "AGENT":
        return "Agent";
      default:
        return kind;
    }
  };

  const kindBadgeClass = (kind: string): string => {
    switch (kind) {
      case "MCP_SERVER":
        return "bg-blue-500/15 text-blue-400";
      case "SKILL":
        return "bg-green-500/15 text-green-400";
      case "AGENT":
        return "bg-purple-500/15 text-purple-400";
      default:
        return "bg-[rgba(255,255,255,0.06)] text-[color:var(--text-muted)]";
    }
  };
</script>

<section class="grid gap-3.5">
  <header>
    <h1 class="m-0 text-2xl font-semibold">Resources</h1>
    <p class="muted">发现 MCP / Skills / Agent 生产力资源。</p>
  </header>

  {#if $authStatus !== "authenticated"}
    <div class="card rounded-[var(--radius)] p-4">请先连接 GitHub 后浏览资源。</div>
  {:else}
    <!-- Tag filters -->
    {#if $availableResourceTags.length > 0}
      <div class="flex flex-wrap gap-1.5">
        {#each $availableResourceTags as tag (tag)}
          <button
            class="cursor-pointer rounded-full border px-2.5 py-1 text-xs transition-colors {activeTag === tag
              ? 'border-[color:var(--accent)] bg-[rgba(var(--accent-rgb),0.12)] text-[color:var(--accent)]'
              : 'border-[color:var(--border)] bg-transparent text-[color:var(--text-muted)] hover:text-[color:var(--text)]'}"
            onclick={() => handleTagFilter(tag)}
          >
            {tag}
          </button>
        {/each}
        {#if activeTag}
          <button
            class="cursor-pointer rounded-full border border-[color:var(--border)] bg-transparent px-2.5 py-1 text-xs text-[color:var(--text-muted)] hover:text-[color:var(--text)]"
            onclick={() => {
              activeTag = null;
              void clearFilters();
            }}
          >
            清除筛选
          </button>
        {/if}
      </div>
    {/if}

    <!-- Error -->
    {#if $resourcesError}
      <div class="card rounded-[var(--radius)] p-4 text-[color:var(--danger)]">
        {$resourcesError}
      </div>
    {/if}

    <!-- Loading -->
    {#if $resourcesLoading}
      <div class="card rounded-[var(--radius)] p-4 muted">加载资源中...</div>
    {:else if $resources.length === 0}
      <div class="card rounded-[var(--radius)] p-4 muted">
        暂无资源数据。需要先有订阅的仓库以推断语言兴趣。
      </div>
    {:else}
      <div class="grid gap-2.5">
        {#each $resources as resource (resource.resource_id)}
          <div class="card rounded-[var(--radius)] p-4">
            <div class="flex items-start justify-between gap-3">
              <div class="min-w-0 flex-1">
                <div class="flex items-center gap-2">
                  <span class="rounded-full px-2 py-0.5 text-xs font-medium {kindBadgeClass(resource.resource_kind)}">
                    {kindLabel(resource.resource_kind)}
                  </span>
                  {#if resource.is_curated}
                    <span class="rounded-full bg-amber-500/15 px-2 py-0.5 text-xs text-amber-400">精选</span>
                  {/if}
                </div>
                <a
                  href={resource.source_url}
                  class="mt-1.5 block font-medium text-[color:var(--accent)] hover:underline"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  {resource.title}
                </a>
                {#if resource.why_recommended.length > 0}
                  <p class="m-0 mt-1 text-sm muted">{resource.why_recommended[0]}</p>
                {/if}
                <div class="mt-2 flex flex-wrap gap-1.5">
                  {#each resource.languages as lang (lang)}
                    <span class="rounded-full bg-[rgba(255,255,255,0.06)] px-2 py-0.5 text-xs muted">{lang}</span>
                  {/each}
                  {#each resource.framework_tags as tag (tag)}
                    <span class="rounded-full bg-blue-500/10 px-2 py-0.5 text-xs text-blue-400">{tag}</span>
                  {/each}
                  {#each resource.agent_tags as tag (tag)}
                    <span class="rounded-full bg-purple-500/10 px-2 py-0.5 text-xs text-purple-400">{tag}</span>
                  {/each}
                </div>
              </div>
              <div class="flex shrink-0 flex-col items-end gap-1">
                <span class="font-mono text-sm text-[color:var(--accent)]">{resource.score.toFixed(1)}</span>
                <button
                  class="cursor-pointer rounded-[var(--radius-sm)] border px-2 py-1 text-xs transition-colors {resource.is_curated
                    ? 'border-amber-500/30 text-amber-400 hover:bg-amber-500/10'
                    : 'border-[color:var(--border)] text-[color:var(--text-muted)] hover:text-[color:var(--text)]'}"
                  onclick={() => void toggleCurate(resource.resource_id, resource.is_curated)}
                >
                  {resource.is_curated ? "取消精选" : "精选"}
                </button>
              </div>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  {/if}
</section>
