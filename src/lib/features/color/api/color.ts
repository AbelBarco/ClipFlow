import { invoke } from "@tauri-apps/api/core";
import type { ColorConversion } from "../types/color.types";

export async function convertColor(input: string): Promise<ColorConversion> {
  return invoke("color_convert", { input });
}

export async function detectColor(input: string): Promise<boolean> {
  return invoke("color_detect", { input });
}
