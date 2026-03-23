<script lang="ts">
  import { addSubscription } from "$lib/stores/subscriptions";

  let {
    repoId,
    fullName,
    visible,
    onConfirm,
    onClose,
  }: {
    repoId: number;
    fullName: string;
    visible: boolean;
    onConfirm: (repoId: number, settings: SubscribeSettings) => void | Promise<void>;
    onClose: () => void;
  } = $props();

  type SubscribeSettings = {
    tracking_mode: string;
    digest_window: string;
    notify_high_immediately: boolean;
    event_types: string[];
  };

  let digestWindow = $state<"12h" | "24h">("12h");
  let notifyHigh = $state(true);
  let submitting = $state(false);

  const handleConfirm = async (): Promise<void> => {
    if (submitting) return;
    submitting = true;
    const settings: SubscribeSettings = {
      tracking_mode: "STANDARD",
      digest_window: digestWindow,
      notify_high_immediately: notifyHigh,
      event_types: ["RELEASE_PUBLISHED", "TAG_PUBLISHED"],
    };
    try {
      if (onConfirm) {
        await onConfirm(repoId, settings);
      } else {
        await addSubscription(repoId, {
          repo_id: repoId,
          tracking_mode: settings.tracking_mode,
          digest_window: settings.digest_window,
          notify_high_immediately: settings.notify_high_immediately,
          event_types: settings.event_types,
        });
      }
      onClose();
    } finally {
      submitting = false;
    }
  };
</script>

{#if visible}
  <button
    type="button"
    class="popover-backdrop"
    aria-label="关闭订阅面板"
    onclick={onClose}
  ></button>
  <div class="popover" role="dialog">
    <header class="popover-header">
      <h3 class="popover-title">订阅 {fullName}</h3>
      <button
        type="button"
        class="popover-close"
        onclick={onClose}
        aria-label="关闭">×</button
      >
    </header>

    <div class="popover-body">
      <div class="field">
        <label class="field-label" for="digest-window">通知频率</label>
        <select
          id="digest-window"
          class="field-select"
          bind:value={digestWindow}
        >
          <option value="12h">每 12 小时摘要</option>
          <option value="24h">每 24 小时摘要</option>
        </select>
      </div>

      <div class="field">
        <label class="toggle-label">
          <input type="checkbox" bind:checked={notifyHigh} />
          <span>高优先级事件立即通知</span>
        </label>
      </div>

      <p class="hint">
        监听事件：Release、Tag（排除 Branch Digest）
      </p>
    </div>

    <footer class="popover-footer">
      <button
        type="button"
        class="confirm-btn"
        onclick={() => void handleConfirm()}
        disabled={submitting}
      >
        {submitting ? "订阅中…" : "确认订阅"}
      </button>
    </footer>
  </div>
{/if}

<style>
  .popover-backdrop {
    position: fixed;
    inset: 0;
    z-index: 40;
    border: none;
    background: transparent;
    cursor: default;
  }

  .popover {
    position: absolute;
    z-index: 50;
    right: 0;
    top: calc(100% + 4px);
    width: 280px;
    border-radius: var(--radius);
    border: 1px solid var(--border);
    background: var(--card-bg, rgba(17, 26, 44, 0.95));
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.45);
    overflow: hidden;
  }

  .popover-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.625rem 0.75rem;
    border-bottom: 1px solid var(--border);
  }

  .popover-title {
    margin: 0;
    font-size: 0.8125rem;
    font-weight: 600;
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .popover-close {
    border: none;
    background: none;
    color: var(--text-muted);
    font-size: 1rem;
    cursor: pointer;
    padding: 0 0.25rem;
    line-height: 1;
  }

  .popover-body {
    padding: 0.75rem;
    display: grid;
    gap: 0.625rem;
  }

  .field {
    display: grid;
    gap: 0.25rem;
  }

  .field-label {
    font-size: 0.6875rem;
    font-weight: 500;
    color: var(--text-muted);
  }

  .field-select {
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: rgba(0, 0, 0, 0.18);
    padding: 0.375rem 0.5rem;
    color: var(--text);
    font-size: 0.75rem;
  }

  .field-select option {
    background: #1a2436;
    color: var(--text);
  }

  .toggle-label {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    cursor: pointer;
    font-size: 0.75rem;
    color: var(--text);
  }

  .toggle-label input[type="checkbox"] {
    accent-color: var(--accent);
  }

  .hint {
    margin: 0;
    font-size: 0.625rem;
    color: var(--text-muted);
    line-height: 1.4;
  }

  .popover-footer {
    padding: 0.5rem 0.75rem;
    border-top: 1px solid var(--border);
  }

  .confirm-btn {
    width: 100%;
    border: none;
    border-radius: var(--radius-sm);
    background: linear-gradient(to bottom right, var(--accent), var(--accent-2));
    padding: 0.4375rem;
    font-size: 0.75rem;
    font-weight: 600;
    color: #02131a;
    cursor: pointer;
  }

  .confirm-btn:hover:not(:disabled) {
    opacity: 0.9;
  }

  .confirm-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

</style>
