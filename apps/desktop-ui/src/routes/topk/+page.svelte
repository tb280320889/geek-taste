<script lang="ts">
  import RepoInfoModal from "$lib/components/RepoInfoModal.svelte";
  import { fetchRepoInfo } from "$lib/ipc/tauri";
  import { authStatus } from "$lib/stores/auth";
  import type { RepoBasicInfo } from "$lib/types";

  let query = $state("");
  let loading = $state(false);
  let error = $state("");
  let repoInfo = $state<RepoBasicInfo | null>(null);

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
</script>

<section class="grid gap-3.5">
  <header>
    <h1 class="m-0 text-2xl font-semibold">TopK 探索</h1>
    <p class="muted">先输入仓库快速查看，再进入阶段 2 排行榜。</p>
  </header>

  {#if $authStatus !== "authenticated"}
    <div class="card rounded-[var(--radius)] p-4">请先连接 GitHub 后再查询仓库信息。</div>
  {:else}
    <form
      class="card rounded-[var(--radius)] p-4"
      onsubmit={(e) => {
        e.preventDefault();
        void onSearch();
      }}
    >
      <label for="repo-input" class="mb-2 block text-sm text-[color:var(--text-muted)]">仓库</label>
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
  {/if}

  <RepoInfoModal repoInfo={repoInfo} onClose={() => (repoInfo = null)} />
</section>
