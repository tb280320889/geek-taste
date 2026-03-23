<script lang="ts">
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-shell";
  import { storeToken, validateToken } from "$lib/ipc/tauri";
  import { authStatus, initAuth } from "$lib/stores/auth";
  import type { UserDto } from "$lib/types";

  let token = $state("");
  let showToken = $state(false);
  let loading = $state(false);
  let error = $state("");
  let validatedUser = $state<UserDto | null>(null);

  onMount(() => {
    if ($authStatus === "authenticated") {
      void goto(resolve("/"));
    }
  });

  const mapError = (message: string): string => {
    if (message.includes("401") || message.includes("InvalidToken")) {
      return "Token 无效，请检查是否正确复制";
    }
    if (message.includes("403") || message.includes("RateLimited")) {
      return "Token 权限不足，需要 repo 和 user 范围";
    }
    if (message.toLowerCase().includes("network")) {
      return "无法连接 GitHub，请检查网络";
    }
    return "验证失败，请重试";
  };

  const onValidate = async (): Promise<void> => {
    error = "";
    validatedUser = null;
    if (!token.trim()) {
      error = "请输入 GitHub PAT";
      return;
    }

    loading = true;
    try {
      const response = await validateToken(token.trim());
      if (!response.success || !response.user) {
        error = mapError(response.error || "unknown");
        return;
      }
      validatedUser = response.user;
    } catch (err) {
      error = mapError(err instanceof Error ? err.message : String(err));
    } finally {
      loading = false;
    }
  };

  const onConfirm = async (): Promise<void> => {
    if (!validatedUser) return;
    loading = true;
    error = "";
    try {
      await storeToken(token.trim());
      await initAuth();
      await goto(resolve("/"));
    } catch (err) {
      error = err instanceof Error ? err.message : "存储失败，请重试";
    } finally {
      loading = false;
    }
  };
</script>

<section class="card w-full max-w-[620px] rounded-[var(--radius)] p-[22px]">
  <header>
    <p class="m-0 mb-1.5 text-xs tracking-[0.08em] text-[color:var(--accent)] uppercase">Step 1 / 2</p>
    <h1 class="m-0 text-2xl font-semibold">连接 GitHub</h1>
    <p class="muted">输入 Personal Access Token 以启用仓库探索和订阅能力。</p>
  </header>

  {#if !validatedUser}
    <label for="pat" class="mt-4 block text-sm text-[color:var(--text-muted)]">GitHub PAT</label>
    <div class="mt-2 grid grid-cols-1 gap-2 sm:grid-cols-[1fr_auto]">
      <input
        id="pat"
        bind:value={token}
        type={showToken ? "text" : "password"}
        class="rounded-[var(--radius-sm)] border border-[color:var(--border)] bg-[rgba(0,0,0,0.2)] px-2.5 py-2.5 text-[color:var(--text)]"
        placeholder="ghp_xxxxxxxxx"
        autocomplete="off"
      />
      <button
        type="button"
        class="cursor-pointer rounded-[var(--radius-sm)] border border-[color:var(--border)] bg-[rgba(255,255,255,0.02)] px-3 py-2.5 text-[color:var(--text-muted)]"
        onclick={() => (showToken = !showToken)}>
        {showToken ? "隐藏" : "显示"}
      </button>
    </div>

    <div class="mt-2">
      <button
        type="button"
        class="cursor-pointer border-0 bg-transparent p-0 text-[color:var(--accent-2)]"
        onclick={() => open("https://github.com/settings/tokens?type=beta")}
      >
        如何创建 GitHub PAT?
      </button>
    </div>

    {#if error}
      <p class="mt-2.5 text-[color:var(--danger)]">{error}</p>
    {/if}

    <button
      type="button"
      class="mt-3.5 cursor-pointer rounded-[var(--radius-sm)] border-0 bg-linear-to-br from-[color:var(--accent)] to-[color:var(--accent-2)] px-3.5 py-2.5 font-bold text-[#031217] disabled:cursor-not-allowed disabled:opacity-80"
      disabled={loading}
      onclick={() => void onValidate()}>
      {loading ? "验证中..." : "验证并继续"}
    </button>
  {:else}
    <p class="m-0 mb-1.5 text-xs tracking-[0.08em] text-[color:var(--accent)] uppercase">Step 2 / 2</p>
    <div class="card mt-3 flex items-center gap-2.5 rounded-[var(--radius)] p-3.5">
      <img src={validatedUser.avatar_url} alt="avatar" class="h-10.5 w-10.5 rounded-full" />
      <div>
        <strong>{validatedUser.name || validatedUser.login}</strong>
        <p class="muted m-0 mt-0.5">@{validatedUser.login}</p>
      </div>
    </div>

    {#if error}
      <p class="mt-2.5 text-[color:var(--danger)]">{error}</p>
    {/if}

    <div class="mt-3 flex gap-2">
      <button
        type="button"
        class="cursor-pointer rounded-[var(--radius-sm)] border border-[color:var(--border)] bg-[rgba(255,255,255,0.02)] px-3.5 py-2.5 text-[color:var(--text-muted)]"
        onclick={() => (validatedUser = null)}>返回修改</button
      >
      <button
        type="button"
        class="cursor-pointer rounded-[var(--radius-sm)] border-0 bg-linear-to-br from-[color:var(--accent)] to-[color:var(--accent-2)] px-3.5 py-2.5 font-bold text-[#031217] disabled:cursor-not-allowed disabled:opacity-80"
        disabled={loading}
        onclick={() => void onConfirm()}>
        {loading ? "保存中..." : "确认并继续"}
      </button>
    </div>
  {/if}
</section>
