export type ColorFormat = "hex" | "rgb" | "hsl" | "css";

export interface ColorValue {
  r: number;
  g: number;
  b: number;
  a?: number;
}

export interface ParsedColor {
  format: ColorFormat;
  value: ColorValue;
  original: string;
}

export interface ColorConversion {
  hex: string;
  rgb: string;
  hsl: string;
  css: string;
}
