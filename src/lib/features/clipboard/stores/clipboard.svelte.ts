import { invoke } from '@tauri-apps/api/core';
import type { ClipItem, ClipboardFilter, ClipboardState } from '../types/clipboard.types';
import { Fuse } from 'fuse.js';

class ClipboardStore implements ClipboardState {
  items: ClipItem[] = $state([]);
  selectedId: string | null = $state(null);
  filter: ClipboardFilter = $state({ query: '', types: [] });
  isLoading: boolean = $state(false);

  private fuse: Fuse<ClipItem>;

  constructor() {
    this.fuse = new Fuse(this.items, {
      keys: ['content', 'preview', 'ocrText'],
      threshold: 0.4,
      includeScore: true,
      minMatchCharLength: 2
    });
  }

  get filteredItems(): ClipItem[] {
    if (!this.filter.query.trim()) {
      return this.items;
    }
    const results = this.fuse.search(this.filter.query);
    return results.map(r => r.item);
  }

  async loadHistory(): Promise<void> {
    this.isLoading = true;
    try {
      const items = await invoke<ClipItem[]>('clipboard_get_history');
      this.items = items;
      this.fuse = new Fuse(this.items, {
        keys: ['content', 'preview', 'ocrText'],
        threshold: 0.4,
        includeScore: true,
        minMatchCharLength: 2
      });
    } catch (error) {
      console.error('Failed to load clipboard history:', error);
    } finally {
      this.isLoading = false;
    }
  }

  async deleteItem(id: string): Promise<void> {
    await invoke('clipboard_delete_item', { itemId: id });
    this.items = this.items.filter(item => item.id !== id);
    this.fuse = new Fuse(this.items, {
      keys: ['content', 'preview', 'ocrText'],
      threshold: 0.4,
      includeScore: true,
      minMatchCharLength: 2
    });
  }

  async clearHistory(): Promise<void> {
    await invoke('clipboard_clear_history');
    this.items = [];
    this.fuse = new Fuse(this.items, {
      keys: ['content', 'preview', 'ocrText'],
      threshold: 0.4,
      includeScore: true,
      minMatchCharLength: 2
    });
  }

  selectItem(id: string | null): void {
    this.selectedId = id;
  }

  setFilter(filter: Partial<ClipboardFilter>): void {
    this.filter = { ...this.filter, ...filter };
  }

  addItem(item: ClipItem): void {
    this.items = [item, ...this.items];
    this.fuse = new Fuse(this.items, {
      keys: ['content', 'preview', 'ocrText'],
      threshold: 0.4,
      includeScore: true,
      minMatchCharLength: 2
    });
  }

  updateItem(id: string, updates: Partial<ClipItem>): void {
    const index = this.items.findIndex(item => item.id === id);
    if (index !== -1) {
      this.items[index] = { ...this.items[index], ...updates };
      this.fuse = new Fuse(this.items, {
        keys: ['content', 'preview', 'ocrText'],
        threshold: 0.4,
        includeScore: true,
        minMatchCharLength: 2
      });
    }
  }
}

export const clipboardStore = new ClipboardStore();