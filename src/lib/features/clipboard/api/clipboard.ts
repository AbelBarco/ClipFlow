import { invoke } from '@tauri-apps/api/core';
import type { ClipItem } from '../types/clipboard.types';

export async function getHistory(): Promise<ClipItem[]> {
  return invoke('clipboard_get_history');
}

export async function deleteItem(itemId: string): Promise<void> {
  return invoke('clipboard_delete_item', { itemId });
}

export async function clearHistory(): Promise<void> {
  return invoke('clipboard_clear_history');
}

export async function pasteItem(itemId: string): Promise<void> {
  return invoke('clipboard_paste_item', { itemId });
}

export async function copyToClipboard(content: string, type: string): Promise<void> {
  return invoke('clipboard_copy_to_clipboard', { content, type });
}