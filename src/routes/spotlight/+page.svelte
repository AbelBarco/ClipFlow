<script lang="ts">
  import { onMount } from 'svelte';
  import ClipboardList from '$lib/features/clipboard/components/ClipboardList.svelte';
  import SearchBar from '$lib/features/clipboard/components/SearchBar.svelte';
  import EmptyState from '$lib/features/ui/components/EmptyState.svelte';
  import { clipboardStore } from '$lib/features/clipboard/stores/clipboard.svelte';
  import { invoke } from '@tauri-apps/api/core';

  let searchInput: HTMLInputElement;

  onMount(() => {
    clipboardStore.loadHistory();
    searchInput?.focus();
  });

  async function handlePaste(itemId: string) {
    await invoke('clipboard_paste_item', { itemId });
    window.close();
  }
</script>

<div class="flex flex-col h-full min-h-[300px] w-[560px] p-4" style="box-shadow: var(--shadow-spotlight)">
  <SearchBar bind:value={clipboardStore.filter} placeholder="Search clipboard..." bind:inputElement={searchInput} autoFocus />

  <div class="flex-1 overflow-hidden mt-3">
    {#if clipboardStore.filteredItems.length === 0}
      <EmptyState
        title={clipboardStore.filter ? 'No matches found' : 'Clipboard is empty'}
        description={clipboardStore.filter ? 'Try a different search term' : 'Copy something to get started'}
        compact
      />
    {:else}
      <ClipboardList items={clipboardStore.filteredItems} onPaste={handlePaste} compact />
    {/if}
  </div>

  <footer class="flex items-center justify-between mt-3 pt-3 border-t border-surface-200 dark:border-surface-700">
    <kbd class="text-xs text-surface-500 dark:text-surface-400 px-2 py-0.5 bg-surface-100 dark:bg-surface-800 rounded">
      Enter
    </kbd>
    <span class="text-xs text-surface-500 dark:text-surface-400">Paste selected</span>
  </footer>
</div>