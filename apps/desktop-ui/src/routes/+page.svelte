<script lang="ts">
  import { resolve } from "$app/paths";
  import { currentUser } from "$lib/stores/auth";
</script>

<section class="grid gap-4">
  <div class="card flex items-center gap-3.5 rounded-[var(--radius)] p-[18px]">
    <div
      class="grid h-14 w-14 place-items-center overflow-hidden rounded-full border-2 border-[rgba(106,179,255,0.4)] bg-[rgba(255,255,255,0.03)] text-[color:var(--text-muted)]"
    >
      {#if $currentUser?.avatar_url}
        <img src={$currentUser.avatar_url} alt="avatar" class="h-full w-full object-cover" />
      {:else}
        <span>GT</span>
      {/if}
    </div>
    <div>
      <h1 class="m-0 text-2xl font-semibold">欢迎回来，{$currentUser?.name || $currentUser?.login || "Geek"}</h1>
      <p class="muted m-0 mt-1.5">今天先看趋势，再处理订阅信号。</p>
    </div>
  </div>

  <div class="grid gap-3 md:grid-cols-3">
    <a
      class="card rounded-[var(--radius)] border border-[color:var(--border)] p-4 text-[color:var(--text)] no-underline transition duration-150 ease-out hover:-translate-y-0.5 hover:border-[rgba(106,179,255,0.35)]"
      href={resolve("/topk")}
    >
      <h2 class="m-0 mb-1.5 text-base font-semibold">发现趋势</h2>
      <p class="muted m-0">探索仓库并查看基础信息</p>
    </a>
    <a
      class="card rounded-[var(--radius)] border border-[color:var(--border)] p-4 text-[color:var(--text)] no-underline transition duration-150 ease-out hover:-translate-y-0.5 hover:border-[rgba(106,179,255,0.35)]"
      href={resolve("/subscriptions")}
    >
      <h2 class="m-0 mb-1.5 text-base font-semibold">管理订阅</h2>
      <p class="muted m-0">维护你关注的项目列表</p>
    </a>
    <a
      class="card rounded-[var(--radius)] border border-[color:var(--border)] p-4 text-[color:var(--text)] no-underline transition duration-150 ease-out hover:-translate-y-0.5 hover:border-[rgba(106,179,255,0.35)]"
      href={resolve("/resources")}
    >
      <h2 class="m-0 mb-1.5 text-base font-semibold">Agent 资源</h2>
      <p class="muted m-0">按技术栈发现 MCP/Skills</p>
    </a>
  </div>
</section>
