<script lang="ts">
  import { onMount } from "svelte";
  import {
    resources,
    resourcesLoading,
    resourcesError,
    resourceFilters,
    availableResourceTags,
    loadResources,
    filterResources,
    toggleCurate,
    clearFilters,
  } from "$lib/stores/resources";
  import ResourceCard from "$lib/components/ResourceCard.svelte";
  import ResourceFilters from "$lib/components/ResourceFilters.svelte";

  onMount(() => {
    void loadResources();
  });
</script>

<div class="space-y-4">
  <div class="flex items-center justify-between">
    <h1 class="m-0 text-2xl font-semibold">Resources</h1>
    <span class="text-sm muted">{$resources.length} 个资源</span>
  </div>

  <ResourceFilters
    availableTags={$availableResourceTags}
    currentFilters={$resourceFilters}
    onFilter={filterResources}
    onClear={clearFilters}
  />

  {#if $resourcesLoading}
    <div class="text-center py-8 muted">加载中...</div>
  {:else if $resourcesError}
    <div class="text-center py-8 text-red-400">{$resourcesError}</div>
  {:else if $resources.length === 0}
    <div class="text-center py-8 muted">
      <p class="text-lg mb-2">暂无资源</p>
      <p class="text-sm">添加感兴趣的 MCP / Skills / Agent 仓库到精选列表开始使用</p>
    </div>
  {:else}
    <div class="space-y-3">
      {#each $resources as resource (resource.resource_id)}
        <ResourceCard
          {resource}
          onToggleCurate={toggleCurate}
        />
      {/each}
    </div>
  {/if}
</div>
