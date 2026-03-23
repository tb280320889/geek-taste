import { writable } from "svelte/store";
import { getCurrentUser, removeToken } from "$lib/ipc/tauri";
import type { UserDto } from "$lib/types";

export type AuthStatus = "loading" | "authenticated" | "unauthenticated";

export const authStatus = writable<AuthStatus>("loading");
export const currentUser = writable<UserDto | null>(null);

export const initAuth = async (): Promise<void> => {
  authStatus.set("loading");
  try {
    const user = await getCurrentUser();
    if (user) {
      currentUser.set(user);
      authStatus.set("authenticated");
      return;
    }
  } catch {
    // ignore and mark as unauthenticated
  }

  currentUser.set(null);
  authStatus.set("unauthenticated");
};

export const logout = async (): Promise<void> => {
  await removeToken();
  currentUser.set(null);
  authStatus.set("unauthenticated");
};
