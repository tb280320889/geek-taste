import { writable } from "svelte/store";
import { getSettings, updateSettings as updateSettingsIpc } from "$lib/ipc/tauri";
import type { SettingsDto, UpdateSettingsRequest } from "$lib/types";

const DEFAULT_SETTINGS: SettingsDto = {
  notification_frequency: "digest12h",
  language_interests: [],
  quiet_hours: null,
  github_api_enabled: true,
};

export const settings = writable<SettingsDto>(DEFAULT_SETTINGS);

export const loadSettings = async (): Promise<SettingsDto> => {
  const next = await getSettings();
  settings.set(next);
  return next;
};

export const updateSettings = async (
  patch: UpdateSettingsRequest,
): Promise<SettingsDto> => {
  const next = await updateSettingsIpc(patch);
  settings.set(next);
  return next;
};
