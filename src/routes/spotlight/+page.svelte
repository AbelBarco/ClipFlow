<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import ClipboardList from '$lib/features/clipboard/components/ClipboardList.svelte';
  import SearchBar from '$lib/features/clipboard/components/SearchBar.svelte';
  import EmptyState from '$lib/features/ui/components/EmptyState.svelte';
  import { clipboardStore } from '$lib/features/clipboard/stores/clipboard.svelte';
  import type { ClipItem } from '$lib/features/clipboard/types/clipboard.types';
  import { invoke } from '@tauri-apps/api/core';



  onMount(() => {
    void clipboardStore.loadHistory();

    let unlisten: (() => void) | undefined;
    listen<ClipItem>('clipboard-item-added', (event) => {
      clipboardStore.addItem(event.payload);
    })
      .then((fn) => {
        unlisten = fn;
      })
      .catch((e) => console.error('Failed to listen for clipboard events:', e));

    function handleKeydown(e: KeyboardEvent) {
      if (e.key === 'Escape') {
        void closeWindow();
      } else if (e.key === 'Enter' && clipboardStore.filteredItems.length > 0) {
        const first = clipboardStore.filteredItems[0];
        void handlePaste(first.id);
      }
    }
    window.addEventListener('keydown', handleKeydown);

    return () => {
      unlisten?.();
      window.removeEventListener('keydown', handleKeydown);
    };
  });

  async function closeWindow() {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      await getCurrentWindow().hide();
    } catch {
      // Not in Tauri — nothing to do.
    }
  }

  async function handlePaste(itemId: string) {
    try {
      await invoke('clipboard_paste_item', { itemId });
    } catch (e) {
      console.error('Paste failed:', e);
    }
    await closeWindow();
  }
</script>

<div class="flex flex-col h-full min-h-[300px] w-full max-w-[560px] mx-auto p-4">
  <SearchBar bind:value={clipboardStore.filter.query} placeholder="Search clipboard..." autoFocus />

  <div class="flex-1 overflow-hidden mt-3">
    {#if clipboardStore.filteredItems.length === 0}
      <EmptyState
        title={clipboardStore.filter.query ? 'No matches found' : 'Clipboard is empty'}
        description={clipboardStore.filter.query ? 'Try a different search term' : 'Copy something to get started'}
        compact
      />
    {:else}
      <ClipboardList items={clipboardStore.filteredItems} onPaste={handlePaste} compact />
    {/if}
  </div>

  <footer class="flex items-center gap-2 mt-3 pt-3 border-t border-surface-200 dark:border-surface-700">
    <kbd class="text-xs text-surface-500 dark:text-surface-400 px-2 py-0.5 bg-surface-100 dark:bg-surface-800 rounded">
      Enter
    </kbd>
    <span class="text-xs text-surface-500 dark:text-surface-400">Paste selected</span>
    <span class="text-xs text-surface-400 dark:text-surface-500 ml-auto">Esc to close</span>
  </footer>
</div>
