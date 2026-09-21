import { invoke } from '@tauri-apps/api/core';
import type { ClipItem, ClipboardFilter, ClipboardState } from '../types/clipboard.types';
import Fuse from 'fuse.js';

function createFuse(items: ClipItem[]): Fuse<ClipItem> {
  return new Fuse(items, {
    keys: ['content', 'preview', 'ocrText'],
    threshold: 0.4,
    includeScore: true,
    minMatchCharLength: 2
  });
}

class ClipboardStore implements ClipboardState {
  items: ClipItem[] = $state([]);
  selectedId: string | null = $state(null);
  filter: ClipboardFilter = $state({ query: '', types: [] });
  isLoading: boolean = $state(false);

  private fuse: Fuse<ClipItem> = createFuse([]);

  get filteredItems(): ClipItem[] {
    const query = this.filter.query.trim();
    const types = this.filter.types;

    let base = this.items;
    if (types.length > 0) {
      base = base.filter((item) => types.includes(item.type));
    }
    if (!query) {
      return base;
    }
    // Fuse busca sobre todo el historial; luego aplicamos el filtro de tipo.
    const ids = new Set(this.fuse.search(query).map((r) => r.item.id));
    return base.filter((item) => ids.has(item.id));
  }

  async loadHistory(): Promise<void> {
    this.isLoading = true;
    try {
      const items = await invoke<ClipItem[]>('clipboard_get_history');
      this.items = items;
      this.fuse = createFuse(this.items);
    } catch (error) {
      console.error('Failed to load clipboard history:', error);
    } finally {
      this.isLoading = false;
    }
  }

  async deleteItem(id: string): Promise<void> {
    await invoke('clipboard_delete_item', { itemId: id });
    this.items = this.items.filter((item) => item.id !== id);
    this.fuse = createFuse(this.items);
  }

  async clearHistory(): Promise<void> {
    await invoke('clipboard_clear_history');
    this.items = [];
    this.fuse = createFuse(this.items);
  }

  selectItem(id: string | null): void {
    this.selectedId = id;
  }

  setFilter(filter: Partial<ClipboardFilter>): void {
    this.filter = { ...this.filter, ...filter };
    // Rebuild fuse index only when items change; query changes don't need it.
  }

  addItem(item: ClipItem): void {
    this.items = [item, ...this.items];
    this.fuse = createFuse(this.items);
  }

  updateItem(id: string, updates: Partial<ClipItem>): void {
    const index = this.items.findIndex((item) => item.id === id);
    if (index !== -1) {
      this.items[index] = { ...this.items[index], ...updates };
      this.fuse = createFuse(this.items);
    }
  }
}

export const clipboardStore = new ClipboardStore();
