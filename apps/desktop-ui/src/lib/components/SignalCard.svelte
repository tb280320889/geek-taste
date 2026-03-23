<script lang="ts">
  import type { SignalDto } from "$lib/types";

  let {
    signal,
    onAck,
    onSeen,
  }: {
    signal: SignalDto;
    onAck: (signalId: string) => void | Promise<void>;
    onSeen: (signalId: string) => void | Promise<void>;
  } = $props();

  const PRIORITY_COLORS: Record<string, string> = {
    HIGH: "#ef4444",
    MEDIUM: "#f59e0b",
    LOW: "#6b7280",
  };

  const resolveTypeLabel = (signalType: string): "Release" | "Tag" | "Commit" | "Other" => {
    const normalized = signalType.toUpperCase();
    if (normalized.includes("RELEASE")) return "Release";
    if (normalized.includes("TAG")) return "Tag";
    if (normalized.includes("COMMIT") || normalized.includes("PUSH")) return "Commit";
    return "Other";
  };

  const formatRelativeTime = (isoDate: string): string => {
    const now = Date.now();
    const then = new Date(isoDate).getTime();
    const diffSec = Math.floor((now - then) / 1000);
    if (diffSec < 60) return "刚刚";
    const diffMin = Math.floor(diffSec / 60);
    if (diffMin < 60) return `${diffMin}m`;
    const diffHr = Math.floor(diffMin / 60);
    if (diffHr < 24) return `${diffHr}h`;
    const diffDay = Math.floor(diffHr / 24);
    return `${diffDay}d`;
  };

  const handleBodyClick = (): void => {
    if (signal.state === "NEW") {
      void onSeen(signal.signal_id);
    }
  };

  const handleAck = (e: MouseEvent): void => {
    e.stopPropagation();
    void onAck(signal.signal_id);
  };

  const barColor = $derived(PRIORITY_COLORS[signal.priority] ?? PRIORITY_COLORS.LOW);
  const typeLabel = $derived(resolveTypeLabel(signal.signal_type));
  const isNew = $derived(signal.state === "NEW");
  const isAcked = $derived(signal.state === "ACKED");
</script>

<article
  class="signal-card"
  class:signal-card--new={isNew}
  class:signal-card--acked={isAcked}
>
  <div class="signal-bar" style="background-color: {barColor}"></div>

  <button type="button" class="signal-body" onclick={handleBodyClick}>
    <div class="signal-headline-row">
      <span class="signal-type">{typeLabel}</span>
      {#if isNew}
        <span class="signal-new-badge">NEW</span>
      {/if}
    </div>
    <span class="signal-title" class:signal-title--bold={isNew}>{signal.title}</span>
    {#if signal.summary}
      <p class="signal-summary">{signal.summary}</p>
    {/if}
  </button>

  <div class="signal-right">
    <span class="signal-time">{formatRelativeTime(signal.occurred_at)}</span>
    {#if signal.state !== "ACKED"}
      <button
        type="button"
        class="signal-ack-btn"
        onclick={handleAck}
      >
        标记已处理
      </button>
    {/if}
  </div>
</article>

<style>
  .signal-card {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.625rem 0.875rem;
    border-radius: var(--radius);
    border: 1px solid var(--border);
    background: var(--card-bg, rgba(17, 26, 44, 0.86));
    text-align: left;
    color: var(--text);
    transition: border-color 0.15s;
    width: 100%;
  }

  .signal-card:hover {
    border-color: rgba(78, 201, 176, 0.2);
  }

  .signal-card--new {
    background: var(--card-bg, rgba(17, 26, 44, 0.94));
    border-color: rgba(78, 201, 176, 0.25);
  }

  .signal-card--acked {
    opacity: 0.55;
  }

  .signal-bar {
    width: 3px;
    align-self: stretch;
    border-radius: 2px;
    flex-shrink: 0;
  }

  .signal-body {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
    border: none;
    background: transparent;
    padding: 0;
    text-align: left;
    cursor: pointer;
  }

  .signal-headline-row {
    display: flex;
    align-items: center;
    gap: 0.375rem;
  }

  .signal-type {
    font-size: 0.625rem;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--text-muted);
    letter-spacing: 0.04em;
  }

  .signal-new-badge {
    border-radius: 9999px;
    background: rgba(78, 201, 176, 0.16);
    border: 1px solid rgba(78, 201, 176, 0.34);
    color: var(--accent);
    font-size: 0.5625rem;
    line-height: 1;
    padding: 0.125rem 0.375rem;
    font-weight: 700;
    letter-spacing: 0.03em;
  }

  .signal-title {
    font-size: 0.8125rem;
    font-weight: 400;
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .signal-title--bold {
    font-weight: 600;
  }

  .signal-summary {
    margin: 0;
    font-size: 0.6875rem;
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .signal-right {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 0.375rem;
    flex-shrink: 0;
  }

  .signal-time {
    font-size: 0.6875rem;
    color: var(--text-muted);
    white-space: nowrap;
  }

  .signal-ack-btn {
    border: none;
    border-radius: var(--radius-sm);
    background: rgba(78, 201, 176, 0.12);
    padding: 0.1875rem 0.5rem;
    font-size: 0.625rem;
    font-weight: 500;
    color: var(--accent);
    cursor: pointer;
    white-space: nowrap;
    transition: background 0.1s;
  }

  .signal-ack-btn:hover {
    background: rgba(78, 201, 176, 0.22);
  }
</style>
