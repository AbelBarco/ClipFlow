export type ClipType = "text" | "url" | "code" | "color" | "image";

export interface ClipItem {
  id: string;
  type: ClipType;
  content: string;
  preview: string;
  timestamp: number;
  size: number;
  ocrText?: string;
  metadata?: Record<string, unknown>;
}

export interface ClipboardFilter {
  query: string;
  types: ClipType[];
  dateRange?: { start: number; end: number };
}

export interface ClipboardState {
  items: ClipItem[];
  selectedId: string | null;
  filter: ClipboardFilter;
  isLoading: boolean;
}
