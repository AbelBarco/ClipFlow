import { invoke } from "@tauri-apps/api/core";
import type { AppSettings } from "../types/settings.types";

export async function getSettings(): Promise<AppSettings> {
  return invoke("settings_get");
}

export async function setSettings(config: AppSettings): Promise<void> {
  return invoke("settings_set", { config });
}

export async function resetSettings(): Promise<AppSettings> {
  return invoke("settings_reset");
}

/** Migrate every stored row and persist the flag. Returns rows converted. */
export async function setEncryption(enabled: boolean): Promise<number> {
  return invoke("security_set_encryption", { enabled });
}

export async function getEncryptionStatus(): Promise<boolean> {
  return invoke("security_status");
}

export interface DictMatchDto {
  index: number;
  length: number;
  word: string;
  suggestions: string[];
}

/** Unknown words + backend suggestions (Word-style, offline). Empty when the
 *  dictionary is unavailable (dev) or the language has no bundled list. */
export async function checkDictText(
  text: string,
  lang: string,
): Promise<DictMatchDto[]> {
  return invoke("corrector_check_text", { text, lang });
}

/** Teach the checker a word. Returns the updated custom word list. */
export async function learnWord(word: string): Promise<string[]> {
  return invoke("corrector_learn_word", { word });
}
