import { invoke } from "@tauri-apps/api/core";
import type { TransformerType } from "../types/transformers.types";

export async function applyTransform(
  text: string,
  transformer: TransformerType,
): Promise<string> {
  return invoke("transform_apply", { text, transformer });
}

export async function getTransformers(): Promise<string[]> {
  return invoke("transform_list");
}
