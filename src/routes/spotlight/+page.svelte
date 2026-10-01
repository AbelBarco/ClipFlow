<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import ClipboardList from '$lib/features/clipboard/components/ClipboardList.svelte';
  import SearchBar from '$lib/features/clipboard/components/SearchBar.svelte';
  import EmptyState from '$lib/features/ui/components/EmptyState.svelte';
  import Kbd from '$lib/features/ui/components/Kbd.svelte';
  import { localeStore } from '$lib/features/i18n/stores/locale.svelte';
  import { clipboardStore } from '$lib/features/clipboard/stores/clipboard.svelte';
  import type { ClipItem } from '$lib/features/clipboard/types/clipboard.types';
  import { invoke } from '@tauri-apps/api/core';

  const PAGE_SIZE = 40;
  let visibleCount = $state(PAGE_SIZE);
  let lastSig = '';

  $effect(() => {
    const sig = clipboardStore.filter.query;
    if (sig !== lastSig) {
      lastSig = sig;
      visibleCount = PAGE_SIZE;
    }
    // Selección válida al filtrar o al llegar items nuevos.
    const items = clipboardStore.filteredItems;
    if (items.length === 0) {
      clipboardStore.selectItem(null);
    } else if (!items.some((i) => i.id === clipboardStore.selectedId)) {
      clipboardStore.selectItem(items[0].id);
    }
  });

  function moveSelection(dir: 1 | -1) {
    const items = clipboardStore.filteredItems;
    if (items.length === 0) return;
    const idx = items.findIndex((i) => i.id === clipboardStore.selectedId);
    const next = idx === -1 ? 0 : (idx + dir + items.length) % items.length;
    if (next >= visibleCount) visibleCount = next + 1;
    clipboardStore.selectItem(items[next].id);
    document
      .querySelector(`[data-item-id="${items[next].id}"]`)
      ?.scrollIntoView({ block: 'nearest' });
  }

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
      } else if (e.key === 'ArrowDown') {
        e.preventDefault();
        moveSelection(1);
      } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        moveSelection(-1);
      } else if (e.key === 'Enter' && clipboardStore.selectedId) {
        e.preventDefault();
        void handlePaste(clipboardStore.selectedId);
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

<div class="flex flex-col h-full min-h-[320px] w-full max-w-[600px] mx-auto p-4">
  <div
    class="flex items-center gap-2 mb-2 select-none cursor-move"
    data-tauri-drag-region
    title={localeStore.t('spotDrag')}
  >
    <span class="text-surface-300 dark:text-surface-600 text-xs leading-none" aria-hidden="true">⋮⋮</span>
    <img src="/logo.png" alt="ClipFlow" class="w-6 h-6 rounded-md" />
    <span class="text-sm font-bold text-surface-900 dark:text-surface-50">ClipFlow</span>
    <span class="text-[11px] text-surface-400 ml-auto">{localeStore.t('spotHint')}</span>
  </div>

  <SearchBar bind:value={clipboardStore.filter.query} placeholder={localeStore.t('searchSpotlight')} autoFocus />

  <div class="flex-1 overflow-hidden mt-3 min-h-[120px]">
    {#if clipboardStore.filteredItems.length === 0}
      <EmptyState
        title={clipboardStore.filter.query
          ? localeStore.t('spotNoResultsTitle')
          : localeStore.t('spotEmptyTitle')}
        description={clipboardStore.filter.query
          ? localeStore.t('spotNoResultsDesc')
          : localeStore.t('spotEmptyDesc')}
        compact
      />
    {:else}
      <ClipboardList
        items={clipboardStore.filteredItems.slice(0, visibleCount)}
        selectedId={clipboardStore.selectedId}
        onSelect={(id) => clipboardStore.selectItem(id)}
        onPaste={handlePaste}
        compact
      />
      {#if clipboardStore.filteredItems.length > visibleCount}
        <button
          class="btn-secondary w-full mt-1.5 text-xs"
          onclick={() => (visibleCount += PAGE_SIZE)}
        >
          {localeStore.t('showMore', { remaining: clipboardStore.filteredItems.length - visibleCount })}
        </button>
      {/if}
    {/if}
  </div>

  <footer class="flex items-center gap-2 mt-3 pt-3 border-t border-surface-200 dark:border-surface-700 text-[11px] text-surface-500 dark:text-surface-400">
    <Kbd keys={['Enter']} /> <span>{localeStore.t('spotPaste')}</span>
    <Kbd keys={['↑', '↓']} /> <span>{localeStore.t('spotChoose')}</span>
    <Kbd keys={['Esc']} /> <span class="ml-auto">{localeStore.t('spotClose')}</span>
  </footer>
</div>
