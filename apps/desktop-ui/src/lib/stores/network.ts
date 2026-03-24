import { writable } from "svelte/store";
import { getSyncStatus } from "$lib/ipc/tauri";

export const isOnline = writable(true);
export const lastSyncedAt = writable<Record<string, string | null>>({});

/** STALE 阈值(毫秒): 排名 1h, 信号 30min */
const STALE_THRESHOLDS: Record<string, number> = {
  topk: 60 * 60 * 1000,
  signals: 30 * 60 * 1000,
};

/** 判断指定数据源是否过期 */
export function isStale(source: string): boolean {
  let result = false;
  lastSyncedAt.subscribe((times) => {
    const ts = times[source];
    if (!ts) {
      result = true;
      return;
    }
    const elapsed = Date.now() - new Date(ts).getTime();
    result = elapsed > (STALE_THRESHOLDS[source] ?? Infinity);
  })();
  return result;
}

/** 从 IPC 错误判断网络状态 */
export function handleIpcError(err: unknown): void {
  const msg = String(err);
  if (msg.includes("NETWORK_ERROR") || msg.includes("network") || msg.includes("fetch failed")) {
    isOnline.set(false);
  }
}

/** 应用启动时检测网络状态 */
export async function checkNetworkStatus(): Promise<void> {
  try {
    const status = await getSyncStatus();
    isOnline.set(status.is_online);
    lastSyncedAt.set({
      topk: status.last_topk_sync,
      signals: status.last_signal_sync,
    });
  } catch {
    isOnline.set(false);
  }
}
