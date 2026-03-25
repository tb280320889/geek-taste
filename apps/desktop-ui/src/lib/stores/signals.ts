import { writable } from 'svelte/store';
import { listHomeSignals, ackSignal, markSignalSeen, getUnreadCounts } from '$lib/ipc/tauri';
import { handleIpcError } from '$lib/stores/network';
import type { SignalDto, UnreadCountsDto } from '$lib/types';

const HOME_SIGNALS_LAST_VISIT_KEY = 'home_signals_last_visit';

function loadLastVisit(): string | null {
  if (typeof window === 'undefined') {
    return null;
  }
  return window.localStorage.getItem(HOME_SIGNALS_LAST_VISIT_KEY);
}

function saveLastVisit(value: string): void {
  if (typeof window === 'undefined') {
    return;
  }
  window.localStorage.setItem(HOME_SIGNALS_LAST_VISIT_KEY, value);
}

export const homeSignals = writable<SignalDto[]>([]);
export const unreadCounts = writable<UnreadCountsDto>({ total: 0, high: 0, medium: 0, low: 0 });
export const signalsLoading = writable(false);
export const homeSignalsLastVisit = writable<string | null>(loadLastVisit());

export async function loadHomeSignals() {
  signalsLoading.set(true);
  const since = loadLastVisit() ?? undefined;
  try {
    const [signals, counts] = await Promise.all([
      listHomeSignals(since),
      getUnreadCounts()
    ]);
    homeSignals.set(signals);
    unreadCounts.set(counts);
    const nextLastVisit = new Date().toISOString();
    saveLastVisit(nextLastVisit);
    homeSignalsLastVisit.set(nextLastVisit);
  } catch (err) {
    handleIpcError(err);
  } finally {
    signalsLoading.set(false);
  }
}

export async function ackSignalAction(signalId: string) {
  await ackSignal(signalId);
  homeSignals.update(list => list.filter(s => s.signal_id !== signalId));
  unreadCounts.update(c => ({ ...c, total: Math.max(0, c.total - 1) }));
}

export async function markSeenAction(signalId: string) {
  await markSignalSeen(signalId);
  homeSignals.update(list => list.map(s =>
    s.signal_id === signalId ? { ...s, state: 'SEEN' as const } : s
  ));
}
