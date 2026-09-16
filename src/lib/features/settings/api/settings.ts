import { invoke } from '@tauri-apps/api/core';
import type { AppSettings } from '../types/settings.types';

export async function getSettings(): Promise<AppSettings> {
  return invoke('settings_get');
}

export async function setSettings(config: AppSettings): Promise<void> {
  return invoke('settings_set', { config });
}

export async function resetSettings(): Promise<AppSettings> {
  return invoke('settings_reset');
}