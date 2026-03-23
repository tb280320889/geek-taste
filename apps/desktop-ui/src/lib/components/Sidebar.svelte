<script lang="ts">
  import { resolve } from "$app/paths";
  import { page } from "$app/stores";

  const navItems = [
    { href: "/", label: "Home", icon: "home" },
    { href: "/topk", label: "TopK", icon: "chart" },
    { href: "/subscriptions", label: "Subscriptions", icon: "stack" },
    { href: "/resources", label: "Resources", icon: "radar" },
    { href: "/rules", label: "Rules", icon: "rules" },
  ] as const;

  const isActive = (href: string, pathname: string): boolean => {
    if (href === "/") return pathname === "/";
    return pathname.startsWith(href);
  };
</script>

<aside
  class="card m-2 flex w-auto flex-col gap-3 rounded-[var(--radius)] bg-linear-to-b from-[rgba(17,26,44,0.95)] to-[rgba(13,21,36,0.95)] p-3 lg:m-3 lg:w-60 lg:p-4"
>
  <div class="flex items-center gap-2.5 px-2 py-1">
    <span
      class="grid h-8 w-8 place-items-center rounded-[10px] bg-linear-to-br from-[color:var(--accent)] to-[color:var(--accent-2)] text-xs font-bold text-[#09101f]"
      >GT</span
    >
    <div>
      <strong class="block text-sm leading-none">geek taste</strong>
      <p class="m-0 text-xs text-[color:var(--text-muted)]">tech radar desk</p>
    </div>
  </div>

  <nav class="grid gap-1.5 md:grid-cols-3 lg:grid-cols-1" aria-label="Main navigation">
    {#each navItems as item (item.href)}
      <a
        href={resolve(item.href)}
        class="flex items-center gap-2.5 rounded-[var(--radius-sm)] border border-transparent px-3 py-2.5 text-[color:var(--text-muted)] no-underline transition duration-150 ease-out hover:border-[rgba(106,179,255,0.2)] hover:bg-[rgba(106,179,255,0.08)] hover:text-[color:var(--text)] md:justify-center lg:justify-start"
        class:active={isActive(item.href, $page.url.pathname)}
        aria-current={isActive(item.href, $page.url.pathname) ? "page" : undefined}
      >
        <span class="w-4 text-center" aria-hidden="true">
          {#if item.icon === "home"}⌂{/if}
          {#if item.icon === "chart"}↗{/if}
          {#if item.icon === "stack"}≡{/if}
          {#if item.icon === "radar"}◎{/if}
          {#if item.icon === "rules"}⚑{/if}
        </span>
        <span>{item.label}</span>
      </a>
    {/each}
  </nav>

  <div class="flex-1"></div>

  <a
    href={resolve("/settings")}
    class="flex items-center gap-2.5 rounded-[var(--radius-sm)] border border-transparent px-3 py-2.5 text-[color:var(--text-muted)] no-underline transition duration-150 ease-out hover:border-[rgba(106,179,255,0.2)] hover:bg-[rgba(106,179,255,0.08)] hover:text-[color:var(--text)]"
    class:active={$page.url.pathname.startsWith("/settings")}
    >⚙ Settings</a
  >
</aside>

<style>
  a.active {
    color: var(--text);
    background: rgba(78, 201, 176, 0.12);
    border-color: rgba(78, 201, 176, 0.3);
  }
</style>
