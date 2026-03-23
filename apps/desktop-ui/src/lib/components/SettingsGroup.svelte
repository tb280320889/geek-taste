<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    title,
    description = "",
    children,
  }: { title: string; description?: string; children: Snippet } = $props();
</script>

<section class="card rounded-[var(--radius)] bg-[rgba(17,26,44,0.86)] p-4">
  <header>
    <h2 class="m-0 text-base font-semibold">{title}</h2>
    {#if description}
      <p class="m-0 mt-1.5 text-sm text-[color:var(--text-muted)]">{description}</p>
    {/if}
  </header>
  <div class="mt-3.5">
    {@render children()}
  </div>
</section>
