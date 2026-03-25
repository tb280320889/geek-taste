<script lang="ts">
  import type { ResourceListRequest } from "$lib/types";

  let {
    availableTags,
    currentFilters,
    onFilter,
    onClear,
  }: {
    availableTags: string[];
    currentFilters: ResourceListRequest;
    onFilter: (filters: ResourceListRequest) => void | Promise<void>;
    onClear: () => void | Promise<void>;
  } = $props();

  const resourceKinds = [
    { value: "", label: "全部类型" },
    { value: "MCP_SERVER", label: "🔌 MCP Server" },
    { value: "SKILL", label: "⚡ Skill" },
    { value: "AGENT_FRAMEWORK", label: "🤖 Agent Framework" },
  ];

  let selectedKind = $state(currentFilters.resource_kind ?? "");
  let selectedTag = $state(currentFilters.tag_value ?? "");
  let selectedLanguage = $state(currentFilters.language ?? "");

  function applyFilters(): void {
    const filters: ResourceListRequest = {};
    if (selectedKind) filters.resource_kind = selectedKind;
    if (selectedTag) {
      filters.tag_type = "free";
      filters.tag_value = selectedTag;
    }
    if (selectedLanguage) filters.language = selectedLanguage;
    void onFilter(filters);
  }

  function handleClear(): void {
    selectedKind = "";
    selectedTag = "";
    selectedLanguage = "";
    void onClear();
  }

  const hasActiveFilters = $derived(!!(selectedKind || selectedTag || selectedLanguage));
</script>

<div class="rf-bar">
  <select
    class="rf-select"
    bind:value={selectedKind}
    onchange={applyFilters}
  >
    {#each resourceKinds as kind}
      <option value={kind.value}>{kind.label}</option>
    {/each}
  </select>

  <select
    class="rf-select"
    bind:value={selectedLanguage}
    onchange={applyFilters}
  >
    <option value="">全部语言</option>
    {#each availableTags as tag}
      <option value={tag}>{tag}</option>
    {/each}
  </select>

  {#if availableTags.length > 0}
    <div class="rf-tag-chips">
      {#each availableTags.slice(0, 8) as tag}
        <button
          type="button"
          class="rf-chip"
          class:rf-chip--active={selectedTag === tag}
          onclick={() => {
            selectedTag = selectedTag === tag ? "" : tag;
            applyFilters();
          }}
        >
          {tag}
        </button>
      {/each}
    </div>
  {/if}

  {#if hasActiveFilters}
    <button type="button" class="rf-clear" onclick={handleClear}>
      清除筛选
    </button>
  {/if}
</div>

<style>
  .rf-bar {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem;
    border-radius: var(--radius);
    background: var(--surface, rgba(17, 26, 44, 0.5));
  }

  .rf-select {
    font-size: 0.8125rem;
    padding: 0.25rem 0.5rem;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--bg, #0f172a);
    color: var(--text);
    cursor: pointer;
  }

  .rf-tag-chips {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
  }

  .rf-chip {
    font-size: 0.6875rem;
    padding: 0.1875rem 0.5rem;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: transparent;
    color: var(--text-muted);
    cursor: pointer;
    transition: background 0.1s, border-color 0.1s;
  }

  .rf-chip:hover {
    background: var(--surface, rgba(17, 26, 44, 0.5));
  }

  .rf-chip--active {
    background: rgba(78, 201, 176, 0.12);
    border-color: var(--accent, rgba(78, 201, 176, 0.5));
    color: var(--accent);
  }

  .rf-clear {
    font-size: 0.6875rem;
    border: none;
    background: transparent;
    color: var(--text-muted);
    cursor: pointer;
    padding: 0.1875rem 0.375rem;
  }

  .rf-clear:hover {
    color: var(--text);
  }
</style>
