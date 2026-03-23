import { writable, derived } from 'svelte/store';
import { listSubscriptions, subscribe, unsubscribe, pauseSubscription, syncSubscriptions } from '$lib/ipc/tauri';
import type { SubscriptionRowDto, CreateSubscriptionRequest } from '$lib/types';

export const subscriptions = writable<SubscriptionRowDto[]>([]);
export const subscriptionsLoading = writable(false);
export const subscriptionsError = writable<string | null>(null);
export const syncInProgress = writable(false);

export const activeSubscriptions = derived(subscriptions, ($subs) =>
  $subs.filter(s => s.state === 'ACTIVE')
);

export const subscribedRepoIds = derived(subscriptions, ($subs) =>
  new Set($subs.filter(s => s.state === 'ACTIVE').map(s => s.repo_id))
);

export async function loadSubscriptions() {
  subscriptionsLoading.set(true);
  subscriptionsError.set(null);
  try {
    const list = await listSubscriptions();
    subscriptions.set(list);
  } catch (e: unknown) {
    subscriptionsError.set(e instanceof Error ? e.message : 'Failed to load subscriptions');
  } finally {
    subscriptionsLoading.set(false);
  }
}

export async function addSubscription(repoId: number, options?: CreateSubscriptionRequest) {
  subscriptionsError.set(null);
  try {
    const sub = await subscribe(repoId, options);
    subscriptions.update(list => {
      const idx = list.findIndex(item => item.repo_id === sub.repo_id);
      if (idx === -1) {
        return [...list, sub];
      }

      const next = [...list];
      next[idx] = sub;
      return next;
    });
    return sub;
  } catch (e: unknown) {
    subscriptionsError.set(e instanceof Error ? e.message : 'Failed to create subscription');
    throw e;
  }
}

export async function removeSubscription(subscriptionId: string) {
  await unsubscribe(subscriptionId);
  subscriptions.update(list => list.filter(s => s.subscription_id !== subscriptionId));
}

export async function togglePause(subscriptionId: string) {
  await pauseSubscription(subscriptionId);
  subscriptions.update(list => list.map(s =>
    s.subscription_id === subscriptionId
      ? { ...s, state: s.state === 'ACTIVE' ? 'PAUSED' as const : 'ACTIVE' as const }
      : s
  ));
}

export async function triggerSync() {
  syncInProgress.set(true);
  try {
    await syncSubscriptions();
    await loadSubscriptions();
  } finally {
    syncInProgress.set(false);
  }
}
