<script lang="ts">
  import type { ResourceCardDto } from "$lib/types";

  let {
    resource,
    onToggleCurate,
  }: {
    resource: ResourceCardDto;
    onToggleCurate?: (id: string, isCurated: boolean) => void | Promise<void>;
  } = $props();

  const kindIcons: Record<string, string> = {
    MCP_SERVER: "🔌",
    SKILL: "⚡",
    AGENT_FRAMEWORK: "🤖",
  };

  const icon = $derived(kindIcons[resource.resource_kind] ?? "📦");
  const scorePercent = $derived(Math.round(resource.score * 100));
  const allTags = $derived([
    ...resource.languages.map((l) => ({ type: "language", value: l })),
    ...resource.framework_tags.map((t) => ({ type: "framework", value: t })),
    ...resource.agent_tags.map((t) => ({ type: "agent", value: t })),
  ]);

  const handleCurate = (): void => {
    void onToggleCurate?.(resource.resource_id, resource.is_curated);
  };
</script>

<article class="resource-card">
  <div class="rc-icon">
    <span class="rc-icon-emoji">{icon}</span>
    <span
      class="rc-score"
      class:rc-score--high={resource.score >= 0.7}
      class:rc-score--mid={resource.score >= 0.4 && resource.score < 0.7}
    >
      {scorePercent}%
    </span>
  </div>

  <div class="rc-body">
    <div class="rc-header">
      <h3 class="rc-title">{resource.title}</h3>
      {#if resource.is_curated}
        <span class="rc-badge">⭐ 精选</span>
      {/if}
    </div>

    {#if allTags.length > 0}
      <div class="rc-tags">
        {#each allTags as tag}
          <span
            class="rc-tag"
            class:rc-tag--lang={tag.type === "language"}
            class:rc-tag--fw={tag.type === "framework"}
            class:rc-tag--agent={tag.type === "agent"}
          >
            {tag.value}
          </span>
        {/each}
      </div>
    {/if}

    {#if resource.why_recommended.length > 0}
      <p class="rc-why">
        <span class="rc-why-label">为什么推荐：</span>
        {resource.why_recommended.join(" · ")}
      </p>
    {/if}

    <div class="rc-actions">
      <a href={resource.source_url} target="_blank" rel="noopener" class="rc-link">
        查看源码 ↗
      </a>
      {#if onToggleCurate}
        <button type="button" class="rc-curate-btn" onclick={handleCurate}>
          {resource.is_curated ? "取消精选" : "加入精选"}
        </button>
      {/if}
    </div>
  </div>
</article>

<style>
  .resource-card {
    display: flex;
    gap: 0.75rem;
    padding: 0.875rem;
    border-radius: var(--radius);
    border: 1px solid var(--border);
    background: var(--card-bg, rgba(17, 26, 44, 0.86));
    color: var(--text);
    transition: border-color 0.15s;
  }

  .resource-card:hover {
    border-color: rgba(78, 201, 176, 0.2);
  }

  .rc-icon {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.25rem;
    min-width: 3rem;
    flex-shrink: 0;
  }

  .rc-icon-emoji {
    font-size: 1.5rem;
  }

  .rc-score {
    font-size: 0.8125rem;
    font-weight: 600;
    color: var(--text-muted);
  }

  .rc-score--high {
    color: #22c55e;
  }

  .rc-score--mid {
    color: #eab308;
  }

  .rc-body {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
  }

  .rc-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .rc-title {
    margin: 0;
    font-size: 0.9375rem;
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .rc-badge {
    font-size: 0.625rem;
    padding: 0.125rem 0.375rem;
    border-radius: var(--radius-sm);
    background: rgba(234, 179, 8, 0.1);
    color: #eab308;
    white-space: nowrap;
  }

  .rc-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
  }

  .rc-tag {
    font-size: 0.6875rem;
    padding: 0.125rem 0.375rem;
    border-radius: var(--radius-sm);
  }

  .rc-tag--lang {
    background: rgba(59, 130, 246, 0.1);
    color: #60a5fa;
  }

  .rc-tag--fw {
    background: rgba(168, 85, 247, 0.1);
    color: #c084fc;
  }

  .rc-tag--agent {
    background: rgba(34, 197, 94, 0.1);
    color: #4ade80;
  }

  .rc-why {
    margin: 0;
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .rc-why-label {
    color: var(--text-muted);
    opacity: 0.7;
  }

  .rc-actions {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-top: 0.125rem;
  }

  .rc-link {
    font-size: 0.8125rem;
    color: #60a5fa;
    text-decoration: none;
  }

  .rc-link:hover {
    text-decoration: underline;
  }

  .rc-curate-btn {
    font-size: 0.75rem;
    padding: 0.1875rem 0.5rem;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: transparent;
    color: var(--text);
    cursor: pointer;
    transition: background 0.1s;
  }

  .rc-curate-btn:hover {
    background: rgba(78, 201, 176, 0.12);
  }
</style>
