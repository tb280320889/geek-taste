<script lang="ts">
  import { open } from "@tauri-apps/plugin-shell";
  import type { RepoBasicInfo } from "$lib/types";

  let { repoInfo, onClose }: { repoInfo: RepoBasicInfo | null; onClose: () => void } = $props();

  const openRepo = async (): Promise<void> => {
    if (!repoInfo?.html_url) return;
    await open(repoInfo.html_url);
  };
</script>

{#if repoInfo}
  <div class="fixed inset-0 z-40 grid place-items-center bg-[rgba(4,10,18,0.72)] p-5">
    <button
      type="button"
      class="absolute inset-0 cursor-pointer border-0 bg-transparent"
      aria-label="关闭详情弹窗"
      onclick={onClose}
    ></button>
    <section
      class="card relative z-10 w-full max-w-[680px] animate-[pop_180ms_ease] rounded-[var(--radius)] p-[18px]"
      role="dialog"
      aria-modal="true"
    >
      <header class="flex items-center justify-between gap-3">
        <h2 class="m-0 text-xl font-semibold">{repoInfo.full_name}</h2>
        <button
          type="button"
          class="h-[34px] w-[34px] rounded-[var(--radius-sm)] border border-[color:var(--border)] bg-transparent text-[color:var(--text-muted)]"
          onclick={onClose}
          aria-label="关闭">×</button
        >
      </header>

      <p class="muted my-3">{repoInfo.description || "暂无仓库描述"}</p>

      <div class="mb-2.5 flex flex-wrap gap-2.5">
        <span class="rounded-full border border-[color:var(--border)] px-2.5 py-1.5"
          >⭐ {repoInfo.stargazers_count.toLocaleString()}</span
        >
        <span class="rounded-full border border-[color:var(--border)] px-2.5 py-1.5"
          >🔀 {repoInfo.forks_count.toLocaleString()}</span
        >
      </div>

      <div class="grid gap-2.5">
        <span
          class="w-fit rounded-full border border-[rgba(106,179,255,0.4)] bg-[rgba(106,179,255,0.16)] px-2.5 py-1 text-xs text-[color:var(--text)]"
          >{repoInfo.language || "Unknown"}</span
        >
        <div class="flex flex-wrap gap-2">
          {#if repoInfo.topics.length === 0}
            <span class="muted rounded-full border border-[color:var(--border)] px-2 py-1 text-xs"
              >No topics</span
            >
          {:else}
            {#each repoInfo.topics as topic (topic)}
              <span class="rounded-full border border-[color:var(--border)] px-2 py-1 text-xs"
                >#{topic}</span
              >
            {/each}
          {/if}
        </div>
      </div>

      <div class="mt-4">
        <button
          type="button"
          class="cursor-pointer rounded-[var(--radius-sm)] border-0 bg-linear-to-br from-[color:var(--accent)] to-[color:var(--accent-2)] px-3 py-2 font-semibold text-[#031015]"
          onclick={openRepo}>在 GitHub 打开</button
        >
      </div>
    </section>
  </div>
{/if}

<style>
  @keyframes pop {
    from {
      opacity: 0;
      transform: scale(0.98) translateY(6px);
    }
    to {
      opacity: 1;
      transform: scale(1) translateY(0);
    }
  }
</style>
