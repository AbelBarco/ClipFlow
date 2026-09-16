<script lang="ts">
  import ClipboardList from '$lib/features/clipboard/components/ClipboardList.svelte';
  import SearchBar from '$lib/features/clipboard/components/SearchBar.svelte';
  import EmptyState from '$lib/features/ui/components/EmptyState.svelte';
  import { clipboardStore } from '$lib/features/clipboard/stores/clipboard.svelte';
</script>

<div class="flex flex-col h-full min-h-[400px] w-[520px] p-4">
  <header class="flex items-center justify-between mb-4">
    <h1 class="text-xl font-semibold text-surface-900 dark:text-surface-50">ClipFlow</h1>
    <span class="text-xs text-surface-500 dark:text-surface-400">
      {clipboardStore.items.length} items
    </span>
  </header>

  <SearchBar bind:value={clipboardStore.filter} placeholder="Search history..." />

  <div class="flex-1 overflow-hidden mt-4">
    {#if clipboardStore.filteredItems.length === 0}
      <EmptyState
        title={clipboardStore.filter ? 'No matches found' : 'Clipboard is empty'}
        description={clipboardStore.filter ? 'Try a different search term' : 'Copy something to get started'}
      />
    {:else}
      <ClipboardList items={clipboardStore.filteredItems} />
    {/if}
  </div>
</div>