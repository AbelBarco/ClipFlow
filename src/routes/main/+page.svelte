<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import ClipboardList from '$lib/features/clipboard/components/ClipboardList.svelte';
  import SearchBar from '$lib/features/clipboard/components/SearchBar.svelte';
  import EmptyState from '$lib/features/ui/components/EmptyState.svelte';
  import { clipboardStore } from '$lib/features/clipboard/stores/clipboard.svelte';
  import type { ClipItem } from '$lib/features/clipboard/types/clipboard.types';

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

    return () => {
      unlisten?.();
    };
  });
</script>

<div class="flex flex-col h-full min-h-[400px] w-full max-w-[520px] mx-auto p-4">
  <header class="flex items-center justify-between mb-4">
    <h1 class="text-xl font-semibold text-surface-900 dark:text-surface-50">ClipFlow</h1>
    <span class="text-xs text-surface-500 dark:text-surface-400">
      {clipboardStore.items.length} items
    </span>
  </header>

  <SearchBar bind:value={clipboardStore.filter.query} placeholder="Search history..." />

  <div class="flex-1 overflow-hidden mt-4">
    {#if clipboardStore.filteredItems.length === 0}
      <EmptyState
        title={clipboardStore.filter.query ? 'No matches found' : 'Clipboard is empty'}
        description={clipboardStore.filter.query ? 'Try a different search term' : 'Copy something to get started'}
      />
    {:else}
      <ClipboardList items={clipboardStore.filteredItems} />
    {/if}
  </div>
</div>
