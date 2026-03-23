<script lang="ts">
  import type { FiltersDto } from "$lib/types";

  let {
    filters,
    rankingMode,
    kValue,
    onApply,
  }: {
    filters: FiltersDto;
    rankingMode: string;
    kValue: number;
    onApply: (filters: FiltersDto, mode: string, k: number) => void;
  } = $props();

  const RANKING_MODES = [
    { value: "STARS_DESC", label: "Stars 降序" },
    { value: "UPDATED_DESC", label: "最近更新" },
    { value: "MOMENTUM_24H", label: "Momentum 24h" },
    { value: "MOMENTUM_7D", label: "Momentum 7d" },
  ] as const;

  const AVAILABLE_LANGUAGES = [
    "Rust", "TypeScript", "JavaScript", "Python", "Go",
    "Java", "C++", "Kotlin", "Swift", "Dart",
  ];

  let selectedLanguages = $state<string[]>([...filters.language]);
  let selectedMode = $state(rankingMode);
  let localK = $state(kValue);
  let localMinStars = $state<number | null>(filters.min_stars);
  let excludeArchived = $state(filters.exclude_archived);
  let excludeForks = $state(filters.exclude_forks);

  const toggleLanguage = (lang: string): void => {
    if (selectedLanguages.includes(lang)) {
      selectedLanguages = selectedLanguages.filter((l) => l !== lang);
    } else {
      selectedLanguages = [...selectedLanguages, lang];
    }
  };

  const handleApply = (): void => {
    onApply(
      {
        language: selectedLanguages,
        exclude_archived: excludeArchived,
        exclude_forks: excludeForks,
        min_stars: localMinStars,
        updated_since_days: filters.updated_since_days,
        topic: filters.topic,
      },
      selectedMode,
      localK,
    );
  };
</script>

<section class="card rounded-[var(--radius)] bg-[rgba(17,26,44,0.86)] p-4">
  <header>
    <h2 class="m-0 text-base font-semibold">筛选条件</h2>
    <p class="m-0 mt-1.5 text-sm text-[color:var(--text-muted)]">配置排名视图的筛选与排序参数</p>
  </header>

  <div class="mt-3.5 grid gap-4">
    <!-- Language -->
    <div>
      <label class="mb-1.5 block text-xs font-medium text-[color:var(--text-muted)]">Language</label>
      <div class="flex flex-wrap gap-1.5">
        {#each AVAILABLE_LANGUAGES as lang (lang)}
          <button
            type="button"
            class="lang-tag"
            class:selected={selectedLanguages.includes(lang)}
            onclick={() => toggleLanguage(lang)}
          >
            {lang}
          </button>
        {/each}
      </div>
    </div>

    <!-- Ranking Mode -->
    <div>
      <label class="mb-1.5 block text-xs font-medium text-[color:var(--text-muted)]" for="ranking-mode">排序模式</label>
      <select
        id="ranking-mode"
        class="filter-select"
        bind:value={selectedMode}
      >
        {#each RANKING_MODES as mode (mode.value)}
          <option value={mode.value}>{mode.label}</option>
        {/each}
      </select>
    </div>

    <!-- Min Stars -->
    <div>
      <label class="mb-1.5 block text-xs font-medium text-[color:var(--text-muted)]" for="min-stars">Min Stars</label>
      <input
        id="min-stars"
        type="number"
        class="filter-input"
        bind:value={localMinStars}
        min={0}
        placeholder="不限"
      />
    </div>

    <!-- K Value -->
    <div>
      <label class="mb-1.5 block text-xs font-medium text-[color:var(--text-muted)]" for="k-value">K 值（结果数量）</label>
      <div class="flex items-center gap-2.5">
        <input
          id="k-value"
          type="range"
          class="k-slider"
          bind:value={localK}
          min={1}
          max={1000}
        />
        <input
          type="number"
          class="filter-input k-number"
          bind:value={localK}
          min={1}
          max={1000}
        />
      </div>
    </div>

    <!-- Toggles -->
    <div class="flex flex-wrap gap-4">
      <label class="toggle-label">
        <input type="checkbox" bind:checked={excludeArchived} />
        <span>排除 Archived</span>
      </label>
      <label class="toggle-label">
        <input type="checkbox" bind:checked={excludeForks} />
        <span>排除 Forks</span>
      </label>
    </div>

    <!-- Apply -->
    <button
      type="button"
      class="apply-btn"
      onclick={handleApply}
    >
      应用筛选（创建新视图）
    </button>
  </div>
</section>

<style>
  .lang-tag {
    border-radius: 9999px;
    border: 1px solid var(--border);
    background: rgba(255, 255, 255, 0.02);
    padding: 0.375rem 0.75rem;
    color: var(--text);
    cursor: pointer;
    font-size: 0.75rem;
    transition: all 0.1s;
  }

  .lang-tag.selected {
    border-color: rgba(78, 201, 176, 0.45);
    background: rgba(78, 201, 176, 0.15);
  }

  .lang-tag:hover {
    border-color: rgba(78, 201, 176, 0.3);
  }

  .filter-select,
  .filter-input {
    width: 100%;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: rgba(0, 0, 0, 0.18);
    padding: 0.5rem 0.625rem;
    color: var(--text);
    font-size: 0.8125rem;
  }

  .filter-select option {
    background: #1a2436;
    color: var(--text);
  }

  .k-slider {
    flex: 1;
    accent-color: var(--accent);
    cursor: pointer;
  }

  .k-number {
    width: 72px;
    flex: none;
  }

  .toggle-label {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    cursor: pointer;
    font-size: 0.8125rem;
    color: var(--text);
  }

  .toggle-label input[type="checkbox"] {
    accent-color: var(--accent);
  }

  .apply-btn {
    border-radius: var(--radius-sm);
    border: none;
    background: linear-gradient(to bottom right, var(--accent), var(--accent-2));
    padding: 0.5rem 1rem;
    font-weight: 600;
    color: #02131a;
    cursor: pointer;
    font-size: 0.8125rem;
  }

  .apply-btn:hover {
    opacity: 0.9;
  }
</style>
