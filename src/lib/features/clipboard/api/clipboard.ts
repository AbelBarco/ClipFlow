import { invoke } from "@tauri-apps/api/core";
import type { ClipItem } from "../types/clipboard.types";

export async function getHistory(): Promise<ClipItem[]> {
  return invoke("clipboard_get_history");
}

export async function deleteItem(itemId: string): Promise<void> {
  return invoke("clipboard_delete_item", { itemId });
}

export async function clearHistory(): Promise<void> {
  return invoke("clipboard_clear_history");
}

export async function pasteItem(itemId: string): Promise<boolean> {
  return invoke("clipboard_paste_item", { itemId });
}

export async function copyToClipboard(
  content: string,
  type: string,
): Promise<void> {
  return invoke("clipboard_copy_to_clipboard", { content, type });
}

/** Devuelve la imagen del historial como `data:` URL lista para <img>. */
export async function getImageDataUrl(itemId: string): Promise<string> {
  return invoke("clipboard_get_image", { itemId });
}

const imageCache = new Map<string, string>();

/** Versión con caché en memoria para miniaturas de la lista. */
export async function getImageDataUrlCached(itemId: string): Promise<string> {
  const cached = imageCache.get(itemId);
  if (cached) return cached;
  const url = await getImageDataUrl(itemId);
  imageCache.set(itemId, url);
  // Las capturas en data URL pesan MBs: caché pequeña para no comer RAM.
  if (imageCache.size > 30) {
    const first = imageCache.keys().next().value;
    if (first) imageCache.delete(first);
  }
  return url;
}

export function forgetImage(itemId: string): void {
  imageCache.delete(itemId);
}

export async function runOcr(itemId: string): Promise<string> {
  return invoke("ocr_run_ocr", { itemId });
}
