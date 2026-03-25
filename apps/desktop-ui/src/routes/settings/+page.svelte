<script lang="ts">
  import { goto } from "$app/navigation";
  import LanguagePicker from "$lib/components/LanguagePicker.svelte";
  import SettingsGroup from "$lib/components/SettingsGroup.svelte";
  import { settings, loadSettings, updateSettings } from "$lib/stores/settings";
  import { logout } from "$lib/stores/auth";
  import type { NotificationFrequency, QuietHoursDto } from "$lib/types";

  const languageOptions = [
    "Rust",
    "TypeScript",
    "JavaScript",
    "Python",
    "Go",
    "Java",
    "C++",
    "C#",
    "Swift",
    "Kotlin",
    "Ruby",
    "PHP",
    "Dart",
    "Shell",
  ];

  let toast = $state("");
  let toastType = $state<"success" | "error">("success");
  let saving = $state(false);
  let loading = $state(true);

  const runSave = async (
    patch: Partial<{
      notification_frequency: NotificationFrequency;
      language_interests: string[];
      quiet_hours: QuietHoursDto | null;
    }>,
  ): Promise<void> => {
    saving = true;
    toast = "";
    try {
      await updateSettings(patch);
      toast = "已保存";
      toastType = "success";
      window.setTimeout(() => {
        if (toast === "已保存") toast = "";
      }, 1200);
    } catch {
      toast = "保存失败";
      toastType = "error";
    } finally {
      saving = false;
    }
  };

  const handleLogout = async (): Promise<void> => {
    await logout();
    await goto("/onboarding");
  };

  const onFrequencyChange = async (value: NotificationFrequency): Promise<void> => {
    await runSave({ notification_frequency: value });
  };

  const onLanguageChange = async (next: string[]): Promise<void> => {
    await runSave({ language_interests: next });
  };

  const onQuietEnabledChange = async (enabled: boolean): Promise<void> => {
    await runSave({ quiet_hours: enabled ? { start: "22:00", end: "08:00" } : null });
  };

  const onQuietTimeChange = async (which: "start" | "end", value: string): Promise<void> => {
    const current = $settings.quiet_hours ?? { start: "22:00", end: "08:00" };
    await runSave({
      quiet_hours: {
        ...current,
        [which]: value,
      },
    });
  };

  void loadSettings().finally(() => {
    loading = false;
  });
</script>

<section class="grid gap-3.5">
  <header>
    <h1 class="m-0 text-2xl font-semibold">设置</h1>
    <p class="muted">通知频率、语言兴趣和安静时段会自动保存到本地。</p>
  </header>

  {#if loading}
    <div class="card rounded-[var(--radius)] p-3.5">加载设置中...</div>
  {:else}
    <SettingsGroup title="通知频率" description="影响信号推送节奏">
      <div class="grid grid-cols-[repeat(auto-fit,minmax(140px,1fr))] gap-2">
        {#each [
          { label: "实时", value: "realtime" },
          { label: "12h Digest", value: "digest12h" },
          { label: "24h Digest", value: "digest24h" },
          { label: "静音", value: "muted" },
        ] as option (option.value)}
          <label
            class="flex cursor-pointer items-center gap-2 rounded-[10px] border border-[color:var(--border)] px-2.5 py-2.5"
            class:selected={$settings.notification_frequency === option.value}
          >
            <input
              type="radio"
              name="frequency"
              checked={$settings.notification_frequency === option.value}
              onchange={() => void onFrequencyChange(option.value as NotificationFrequency)}
              disabled={saving}
            />
            <span>{option.label}</span>
          </label>
        {/each}
      </div>
    </SettingsGroup>

    <SettingsGroup title="语言兴趣" description="用于个性化 TopK 与资源推荐">
      <LanguagePicker
        available={languageOptions}
        selected={$settings.language_interests}
        onChange={(next) => void onLanguageChange(next)}
      />
    </SettingsGroup>

    <SettingsGroup title="安静时段" description="减少夜间打扰">
      <div class="mb-3">
        <label>
          <input
            type="checkbox"
            checked={$settings.quiet_hours !== null}
            onchange={(e) => void onQuietEnabledChange((e.currentTarget as HTMLInputElement).checked)}
          />
          启用安静时段
        </label>
      </div>

      {#if $settings.quiet_hours}
        <div class="grid grid-cols-1 gap-2.5 sm:grid-cols-2">
          <label class="grid gap-1.5 text-sm text-[color:var(--text-muted)]">
            开始
            <input
              type="time"
              class="rounded-[var(--radius-sm)] border border-[color:var(--border)] bg-[rgba(0,0,0,0.2)] px-2 py-2 text-[color:var(--text)]"
              value={$settings.quiet_hours.start}
              onchange={(e) =>
                void onQuietTimeChange("start", (e.currentTarget as HTMLInputElement).value)}
            />
          </label>
          <label class="grid gap-1.5 text-sm text-[color:var(--text-muted)]">
            结束
            <input
              type="time"
              class="rounded-[var(--radius-sm)] border border-[color:var(--border)] bg-[rgba(0,0,0,0.2)] px-2 py-2 text-[color:var(--text)]"
              value={$settings.quiet_hours.end}
              onchange={(e) => void onQuietTimeChange("end", (e.currentTarget as HTMLInputElement).value)}
            />
          </label>
        </div>
      {/if}
    </SettingsGroup>

    <div class="mt-4">
      <button
        class="cursor-pointer rounded-[var(--radius-sm)] border border-[color:var(--border)] bg-transparent px-3 py-2 text-sm text-[color:var(--text-muted)] transition hover:border-[color:var(--danger)] hover:text-[color:var(--danger)]"
        onclick={() => void handleLogout()}
      >
        注销
      </button>
    </div>
  {/if}
</section>

{#if toast}
  <div
    class="toast-notification rounded-[var(--radius-sm)] border border-[color:var(--border)] px-4 py-2.5 text-sm shadow-lg"
    class:toast-success={toastType === "success"}
    class:toast-error={toastType === "error"}
  >
    {toast}
  </div>
{/if}

<style>
  label.selected {
    border-color: rgba(78, 201, 176, 0.45);
    background: rgba(78, 201, 176, 0.14);
  }

  .toast-notification {
    position: fixed;
    bottom: 1rem;
    right: 1rem;
    z-index: 1000;
    animation: slideIn 0.3s ease;
    backdrop-filter: blur(8px);
  }

  .toast-success {
    background: rgba(78, 201, 176, 0.15);
    border-color: rgba(78, 201, 176, 0.45);
    color: var(--ok);
  }

  .toast-error {
    background: rgba(255, 50, 50, 0.15);
    border-color: rgba(255, 50, 50, 0.45);
    color: var(--danger);
  }

  @keyframes slideIn {
    from {
      transform: translateY(8px);
      opacity: 0;
    }
    to {
      transform: translateY(0);
      opacity: 1;
    }
  }
</style>
