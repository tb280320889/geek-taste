<script lang="ts">
  let {
    available,
    selected,
    onChange,
  }: {
    available: string[];
    selected: string[];
    onChange: (next: string[]) => void;
  } = $props();

  const toggle = (language: string): void => {
    if (selected.includes(language)) {
      onChange(selected.filter((item) => item !== language));
      return;
    }
    onChange([...selected, language]);
  };
</script>

<div class="grid grid-cols-[repeat(auto-fill,minmax(120px,1fr))] gap-2">
  {#each available as language (language)}
    <button
      type="button"
      class="cursor-pointer rounded-full border border-[color:var(--border)] bg-[rgba(255,255,255,0.02)] px-3 py-2 text-[color:var(--text)]"
      class:selected={selected.includes(language)}
      onclick={() => toggle(language)}
    >
      {language}
    </button>
  {/each}
</div>

<style>
  button.selected {
    border-color: rgba(78, 201, 176, 0.45);
    background: rgba(78, 201, 176, 0.15);
  }
</style>
