<script lang="ts">
  import "../app.css";
  import { onMount } from "svelte";
  import { resolve } from "$app/paths";
  import { page } from "$app/stores";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import { authStatus, initAuth } from "$lib/stores/auth";
  import { isOnline, checkNetworkStatus } from "$lib/stores/network";
  import type { Snippet } from "svelte";

  let { children }: { children: Snippet } = $props();

  onMount(() => {
    void initAuth();
    void checkNetworkStatus();
  });

  const isOnboarding = $derived($page.url.pathname.startsWith("/onboarding"));
</script>

{#if isOnboarding}
  <main class="grid min-h-screen place-items-center p-6">
    {@render children()}
  </main>
{:else}
  <div class="page-shell flex min-h-screen flex-col lg:flex-row">
    <Sidebar />
    <main class="flex-1 overflow-auto p-3 lg:p-5">
      {#if !$isOnline}
        <div class="offline-banner">
          <span>网络不可用，显示缓存数据</span>
          <button type="button" class="dismiss-btn" onclick={() => isOnline.set(true)}>×</button>
        </div>
      {/if}
      {#if $authStatus === "unauthenticated"}
        <div
          class="card mx-auto mt-7 w-full max-w-xl rounded-[var(--radius)] border border-[color:var(--border)] p-6 lg:mt-28"
        >
          <h2 class="m-0 text-2xl font-semibold">请先连接 GitHub</h2>
          <p class="muted mt-3">完成认证后即可使用 TopK、订阅和资源雷达功能。</p>
          <a
            href={resolve("/onboarding")}
            class="mt-4 inline-block rounded-[var(--radius-sm)] bg-linear-to-br from-[color:var(--accent)] to-[color:var(--accent-2)] px-3 py-2 font-semibold text-[#04131a] no-underline"
            >前往认证</a
          >
        </div>
      {:else}
        {@render children()}
      {/if}
    </main>
  </div>
{/if}
