<script lang="ts">
  import { onMount } from 'svelte';
  import { listen, emit } from '@tauri-apps/api/event';
  import ClipboardList from '$lib/features/clipboard/components/ClipboardList.svelte';
  import SearchBar from '$lib/features/clipboard/components/SearchBar.svelte';
  import EmptyState from '$lib/features/ui/components/EmptyState.svelte';
  import PreviewPanel from '$lib/features/clipboard/components/PreviewPanel.svelte';
  import Kbd from '$lib/features/ui/components/Kbd.svelte';
  import { localeStore } from '$lib/features/i18n/stores/locale.svelte';
  import { clipboardStore } from '$lib/features/clipboard/stores/clipboard.svelte';
  import type { ClipItem } from '$lib/features/clipboard/types/clipboard.types';
  import type { ClipType } from '$lib/features/clipboard/types/clipboard.types';
  import { copyToClipboard, forgetImage } from '$lib/features/clipboard/api/clipboard';

  const FILTERS = $derived<{ id: ClipType | 'all'; label: string }[]>([
    { id: 'all', label: localeStore.t('filterAll') },
    { id: 'text', label: localeStore.t('filterText') },
    { id: 'url', label: localeStore.t('filterLinks') },
    { id: 'code', label: localeStore.t('filterCode') },
    { id: 'color', label: localeStore.t('filterColors') },
    { id: 'image', label: localeStore.t('filterImages') }
  ]);

  let toast: string | null = $state(null);
  let toastTimer: ReturnType<typeof setTimeout> | undefined = $state(undefined);

  // Paginación: renderizar cientos de nodos de golpe congela la ventana.
  const PAGE_SIZE = 60;
  let visibleCount = $state(PAGE_SIZE);
  let lastSig = '';

  function notify(msg: string) {
    toast = msg;
    if (toastTimer) clearTimeout(toastTimer);
    toastTimer = setTimeout(() => {
      toast = null;
    }, 2200);
  }

  function toggleType(id: ClipType | 'all') {
    if (id === 'all') {
      clipboardStore.setFilter({ types: [] });
    } else {
      const current = clipboardStore.filter.types;
      clipboardStore.setFilter({
        types: current.includes(id) ? current.filter((t) => t !== id) : [...current, id]
      });
    }
  }

  function isActive(id: ClipType | 'all'): boolean {
    if (id === 'all') return clipboardStore.filter.types.length === 0;
    return clipboardStore.filter.types.includes(id);
  }

  $effect(() => {
    // Al cambiar la búsqueda o los filtros se empieza desde arriba.
    const sig = `${clipboardStore.filter.query}|${clipboardStore.filter.types.join(',')}`;
    if (sig !== lastSig) {
      lastSig = sig;
      visibleCount = PAGE_SIZE;
    }
    // Mantiene una selección válida al filtrar o recibir items nuevos.
    const items = clipboardStore.filteredItems;
    if (items.length === 0) {
      clipboardStore.selectItem(null);
    } else if (!items.some((i) => i.id === clipboardStore.selectedId)) {
      clipboardStore.selectItem(items[0].id);
    }
  });

  function selectedItem(): ClipItem | null {
    return clipboardStore.items.find((i) => i.id === clipboardStore.selectedId) ?? null;
  }

  function moveSelection(dir: 1 | -1) {
    const items = clipboardStore.filteredItems;
    if (items.length === 0) return;
    const idx = items.findIndex((i) => i.id === clipboardStore.selectedId);
    const next = idx === -1 ? (dir === 1 ? 0 : items.length - 1) : (idx + dir + items.length) % items.length;
    if (next >= visibleCount) visibleCount = next + 1;
    clipboardStore.selectItem(items[next].id);
    document
      .querySelector(`[data-item-id="${items[next].id}"]`)
      ?.scrollIntoView({ block: 'nearest' });
  }

  async function handleCopy(id: string) {
    const item = clipboardStore.items.find((i) => i.id === id);
    if (!item) return;
    try {
      await copyToClipboard(item.content, item.type);
      notify(item.type === 'image' ? localeStore.t('toastCopiedImage') : localeStore.t('toastCopied'));
    } catch (e) {
      console.error('Copy failed:', e);
      notify(localeStore.t('toastCopyFailed'));
    }
  }

  async function handleDelete(id: string) {
    try {
      forgetImage(id);
      await clipboardStore.deleteItem(id);
      notify(localeStore.t('toastDeleted'));
    } catch (e) {
      console.error('Delete failed:', e);
    }
  }

  async function handleClear() {
    try {
      await clipboardStore.clearHistory();
      notify(localeStore.t('toastCleared'));
    } catch (e) {
      console.error('Clear failed:', e);
    }
  }

  function handleKeys(e: KeyboardEvent) {
    const target = e.target as HTMLElement | null;
    const typing = target && (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.tagName === 'SELECT');
    if (e.key === 'ArrowDown' && !typing) {
      e.preventDefault();
      moveSelection(1);
    } else if (e.key === 'ArrowUp' && !typing) {
      e.preventDefault();
      moveSelection(-1);
    } else if (e.key === 'Enter' && !typing && clipboardStore.selectedId) {
      e.preventDefault();
      void handleCopy(clipboardStore.selectedId);
    } else if ((e.key === 'Delete' || e.key === 'Backspace') && !typing && clipboardStore.selectedId) {
      e.preventDefault();
      void handleDelete(clipboardStore.selectedId);
    } else if (e.key === 'Escape' && typing && target instanceof HTMLInputElement) {
      target.blur();
      clipboardStore.setFilter({ query: '' });
    }
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

    window.addEventListener('keydown', handleKeys);
    return () => {
      unlisten?.();
      window.removeEventListener('keydown', handleKeys);
      if (toastTimer) clearTimeout(toastTimer);
    };
  });
</script>

<div class="flex flex-col h-screen w-full max-w-[600px] mx-auto p-4">
  <header class="flex items-center gap-2.5 mb-3">
    <img src="/logo.png" alt="ClipFlow" class="w-8 h-8 rounded-lg shadow-item" />
    <div class="flex-1 min-w-0">
      <h1 class="text-lg font-bold text-surface-900 dark:text-surface-50 leading-tight">ClipFlow</h1>
      <p class="text-[11px] text-surface-500 dark:text-surface-400 leading-tight">
        {localeStore.t('headerSubtitle', { count: clipboardStore.items.length })}
      </p>
    </div>
    <button class="btn-ghost p-2" title={localeStore.t('headerReload')} onclick={() => void clipboardStore.loadHistory()} aria-label={localeStore.t('headerReload')}>
      🔄
    </button>
    <button class="btn-ghost p-2" title={localeStore.t('headerSettings')} onclick={() => void emit('show-settings', {})} aria-label={localeStore.t('headerSettings')}>
      ⚙️
    </button>
    <button
      class="btn-ghost p-2 text-red-500"
      title={localeStore.t('headerClear')}
      onclick={() => void handleClear()}
      disabled={clipboardStore.items.length === 0}
      aria-label={localeStore.t('headerClear')}
    >
      🗑️
    </button>
  </header>

  <SearchBar bind:value={clipboardStore.filter.query} placeholder={localeStore.t('searchMain')} />

  <div class="flex gap-1.5 mt-2.5 overflow-x-auto pb-1">
    {#each FILTERS as f}
      <button
        class="flex-shrink-0 px-2.5 py-1 text-xs font-medium rounded-full border transition-colors
          {isActive(f.id)
            ? 'bg-primary-600 border-primary-600 text-white'
            : 'bg-white dark:bg-surface-800 border-surface-200 dark:border-surface-700 text-surface-600 dark:text-surface-300 hover:border-primary-400'}"
        onclick={() => toggleType(f.id)}
      >
        {f.label}
      </button>
    {/each}
  </div>

  <div class="flex-1 overflow-hidden mt-2.5 min-h-[140px]">
    {#if clipboardStore.isLoading}
      <p class="text-sm text-surface-500 text-center py-8">{localeStore.t('statusLoading')}</p>
    {:else if clipboardStore.filteredItems.length === 0}
      {#if clipboardStore.items.length === 0}
        <div class="card p-4">
          <EmptyState
            title={localeStore.t('emptyMainTitle')}
            description={localeStore.t('emptyMainDesc')}
          />
          <ol class="text-[13px] text-surface-600 dark:text-surface-300 space-y-1.5 mt-1 mb-3 px-2">
            <li><strong>1.</strong> {localeStore.t('step1a')} <Kbd keys={['Ctrl', 'C']} />{localeStore.t('step1b')}</li>
            <li><strong>2.</strong> {localeStore.t('step2a')} <Kbd keys={['Ctrl', 'Shift', 'V']} /> {localeStore.t('step2b')}</li>
            <li><strong>3.</strong> {localeStore.t('step3')}</li>
          </ol>
        </div>
      {:else}
        <EmptyState
          title={localeStore.t('noResultsTitle')}
          description={localeStore.t('noResultsDesc')}
        />
      {/if}
    {:else}
      <ClipboardList
        items={clipboardStore.filteredItems.slice(0, visibleCount)}
        selectedId={clipboardStore.selectedId}
        onSelect={(id) => clipboardStore.selectItem(id)}
        onPaste={handleCopy}
        onDelete={handleDelete}
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

  {#if selectedItem()}
    <div class="mt-2 max-h-[300px] overflow-y-auto">
      <PreviewPanel
        item={selectedItem()}
        onPaste={handleCopy}
        onDelete={handleDelete}
        {notify}
      />
    </div>
  {/if}

  <footer class="flex items-center gap-2 mt-2 text-[11px] text-surface-400 dark:text-surface-500">
    <span><Kbd keys={['↑', '↓']} /> {localeStore.t('footerNavigate')}</span>
    <span><Kbd keys={['Enter']} /> {localeStore.t('footerCopy')}</span>
    <span><Kbd keys={['Supr']} /> {localeStore.t('footerDelete')}</span>
  </footer>

  {#if toast}
    <div class="fixed bottom-4 left-1/2 -translate-x-1/2 px-3 py-1.5 text-sm bg-surface-900 dark:bg-surface-50 text-white dark:text-surface-900 rounded-full shadow-spotlight">
      {toast}
    </div>
  {/if}
</div>
