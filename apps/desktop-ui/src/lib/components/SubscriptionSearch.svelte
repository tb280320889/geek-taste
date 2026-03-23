<script lang="ts">
  import { open } from "@tauri-apps/plugin-shell";
  import { fetchRepoInfo } from "$lib/ipc/tauri";
  import type { RepoBasicInfo } from "$lib/types";

  let {
    onSubscribe,
  }: {
    onSubscribe: (repoId: number) => void | Promise<void>;
  } = $props();

  let query = $state("");
  let result = $state<RepoBasicInfo | null>(null);
  let searching = $state(false);
  let subscribing = $state(false);
  let searchError = $state<string | null>(null);
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  const ownerRepoPattern = /^[\w.-]+\/[\w.-]+$/;

  const handleInput = (e: Event): void => {
    const value = (e.target as HTMLInputElement).value;
    query = value;
    result = null;
    searchError = null;

    if (debounceTimer) clearTimeout(debounceTimer);

    if (!ownerRepoPattern.test(value.trim())) return;

    debounceTimer = setTimeout(() => {
      void doSearch(value.trim());
    }, 400);
  };

  const doSearch = async (input: string): Promise<void> => {
    const [owner, repo] = input.split("/");
    searching = true;
    searchError = null;
    try {
      result = await fetchRepoInfo(owner, repo);
    } catch (e: unknown) {
      searchError = e instanceof Error ? e.message : "查询失败";
      result = null;
    } finally {
      searching = false;
    }
  };

  const openRepo = async (): Promise<void> => {
    if (result) await open(result.html_url);
  };

  const subscribeRepo = async (): Promise<void> => {
    if (!result || subscribing) return;
    searchError = null;
    subscribing = true;
    try {
      await onSubscribe(result.repo_id);
    } catch (e: unknown) {
      searchError = e instanceof Error ? e.message : "订阅失败";
    } finally {
      subscribing = false;
    }
  };
</script>

<div class="search-wrap">
  <div class="search-input-row">
    <span class="search-icon">🔍</span>
    <input
      type="text"
      class="search-input"
      placeholder="输入 owner/repo 查看项目信息（如 vercel/next.js）"
      value={query}
      oninput={handleInput}
      spellcheck="false"
    />
    {#if searching}
      <span class="search-spinner">⏳</span>
    {/if}
  </div>

  {#if searchError}
    <div class="search-error">{searchError}</div>
  {/if}

  {#if result}
    <div class="search-result">
      <div class="result-info">
        <button
          type="button"
          class="result-name"
          onclick={() => void openRepo()}
        >
          {result.full_name}
        </button>
        {#if result.description}
          <span class="result-desc">{result.description}</span>
        {/if}
        <div class="result-meta">
          {#if result.language}
            <span class="lang-badge">{result.language}</span>
          {/if}
          <span class="star-count">⭐ {result.stargazers_count.toLocaleString()}</span>
          <span class="fork-count">🔀 {result.forks_count.toLocaleString()}</span>
        </div>
        <div class="result-actions">
          <button
            type="button"
            class="subscribe-btn"
            onclick={() => void subscribeRepo()}
            disabled={subscribing}
          >
            {subscribing ? "订阅中..." : "订阅此仓库"}
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .search-wrap {
    position: relative;
    display: grid;
    gap: 0.5rem;
  }

  .search-input-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    border-radius: var(--radius);
    border: 1px solid var(--border);
    background: var(--card-bg, rgba(17, 26, 44, 0.86));
    padding: 0 0.75rem;
    transition: border-color 0.15s;
  }

  .search-input-row:focus-within {
    border-color: rgba(78, 201, 176, 0.4);
  }

  .search-icon {
    font-size: 0.875rem;
    flex-shrink: 0;
  }

  .search-spinner {
    font-size: 0.875rem;
    flex-shrink: 0;
    animation: pulse 1s infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.4; }
  }

  .search-input {
    flex: 1;
    border: none;
    background: none;
    padding: 0.625rem 0;
    color: var(--text);
    font-size: 0.8125rem;
    outline: none;
  }

  .search-input::placeholder {
    color: var(--text-muted);
    opacity: 0.7;
  }

  .search-error {
    font-size: 0.75rem;
    color: var(--danger);
    padding-left: 0.25rem;
  }

  .search-result {
    padding: 0.625rem 0.875rem;
    border-radius: var(--radius);
    border: 1px solid var(--border);
    background: var(--card-bg, rgba(17, 26, 44, 0.86));
  }

  .result-info {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .result-name {
    display: inline-block;
    border: none;
    background: none;
    padding: 0;
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--accent);
    cursor: pointer;
    text-align: left;
    width: fit-content;
  }

  .result-name:hover {
    text-decoration: underline;
  }

  .result-desc {
    font-size: 0.6875rem;
    color: var(--text-muted);
    line-height: 1.4;
  }

  .result-meta {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .lang-badge {
    border-radius: 9999px;
    border: 1px solid rgba(106, 179, 255, 0.4);
    background: rgba(106, 179, 255, 0.16);
    padding: 0.125rem 0.5rem;
    font-size: 0.625rem;
    color: var(--text);
    white-space: nowrap;
  }

  .star-count, .fork-count {
    font-size: 0.6875rem;
    color: var(--text-muted);
    white-space: nowrap;
  }

  .result-actions {
    margin-top: 0.25rem;
  }

  .subscribe-btn {
    border-radius: var(--radius-sm);
    border: 1px solid rgba(78, 201, 176, 0.35);
    background: rgba(78, 201, 176, 0.1);
    color: var(--text);
    padding: 0.3125rem 0.625rem;
    font-size: 0.6875rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s;
  }

  .subscribe-btn:hover:not(:disabled) {
    border-color: rgba(78, 201, 176, 0.5);
    background: rgba(78, 201, 176, 0.18);
  }

  .subscribe-btn:disabled {
    opacity: 0.65;
    cursor: not-allowed;
  }
</style>
