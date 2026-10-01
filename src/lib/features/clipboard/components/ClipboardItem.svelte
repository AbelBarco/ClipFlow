<script lang="ts">
  import type { ClipItem } from '../types/clipboard.types';
  import { localeStore } from '$lib/features/i18n/stores/locale.svelte';
  import { htmlLangOf } from '$lib/features/i18n/translations';
  import { getImageDataUrlCached } from '../api/clipboard';

  interface Props {
    item: ClipItem;
    compact?: boolean;
    selected?: boolean;
    onSelect?: (itemId: string) => void;
    onPaste?: (itemId: string) => Promise<void>;
    onDelete?: (itemId: string) => Promise<void>;
  }

  let { item, compact = false, selected = false, onSelect, onPaste, onDelete }: Props = $props();

  let thumb: string | null = $state(null);

  // Carga perezosa de la miniatura solo para imágenes.
  $effect(() => {
    if (item.type === 'image') {
      const id = item.id;
      getImageDataUrlCached(id)
        .then((url) => {
          thumb = url;
        })
        .catch(() => {
          thumb = null;
        });
    } else {
      thumb = null;
    }
  });

  function getTypeIcon(type: string): string {
    const icons: Record<string, string> = {
      text: '📝',
      url: '🔗',
      code: '💻',
      color: '🎨',
      image: '🖼️'
    };
    return icons[type] || '📝';
  }

  function formatTimestamp(timestamp: number): string {
    const diff = Date.now() - timestamp;
    const minutes = Math.floor(diff / 60000);
    const hours = Math.floor(diff / 3600000);
    const days = Math.floor(diff / 86400000);

    if (minutes < 1) return localeStore.t('itemNow');
    if (minutes < 60) return localeStore.t('itemMinAgo', { n: minutes });
    if (hours < 24) return localeStore.t('itemHAgo', { n: hours });
    if (days < 7) return localeStore.t('itemDAgo', { n: days });
    return new Date(timestamp).toLocaleDateString(htmlLangOf(localeStore.locale));
  }

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  function typeLabel(type: string): string {
    if (type === 'color') return localeStore.t('itemColor');
    if (type === 'image') return localeStore.t('itemImage');
    return type;
  }

  function handleClick() {
    onSelect?.(item.id);
    // En spotlight un clic pega directamente.
    if (onPaste && compact) {
      void onPaste(item.id);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && onPaste) {
      e.preventDefault();
      void onPaste(item.id);
    }
  }

  function handlePaste(e: MouseEvent) {
    e.stopPropagation();
    if (onPaste) void onPaste(item.id);
  }

  function handleDelete(e: MouseEvent) {
    e.stopPropagation();
    if (onDelete) void onDelete(item.id);
  }
</script>

<div
  data-item-id={item.id}
  class="card flex items-start gap-2.5 p-2.5 w-full text-left item-hover group cursor-pointer
    {selected ? '!border-primary-500 ring-1 ring-primary-500' : ''} {compact ? '!p-2' : ''}"
  onclick={handleClick}
  onkeydown={handleKeydown}
  role="option"
  aria-selected={selected}
  tabindex="0"
>
  {#if item.type === 'image'}
    <span class="w-11 h-11 flex-shrink-0 rounded-lg overflow-hidden bg-surface-100 dark:bg-surface-800 flex items-center justify-center">
      {#if thumb}
        <img src={thumb} alt="" class="w-full h-full object-cover" loading="lazy" />
      {:else}
        <span class="text-lg">🖼️</span>
      {/if}
    </span>
  {:else if item.type === 'color'}
    <span
      class="w-11 h-11 flex-shrink-0 rounded-lg border border-surface-300 dark:border-surface-600"
      style="background-color: {item.content}"
      title={item.content}
    ></span>
  {:else}
    <span class="text-lg flex-shrink-0 mt-0.5 w-7 text-center">{getTypeIcon(item.type)}</span>
  {/if}

  <div class="flex-1 min-w-0">
    <div class="flex items-center gap-2">
      <span class="text-[11px] font-semibold uppercase tracking-wide text-surface-500 dark:text-surface-400">
        {typeLabel(item.type)}
      </span>
      <span class="text-[11px] text-surface-400 dark:text-surface-500 ml-auto flex-shrink-0">
        {formatTimestamp(item.timestamp)}
      </span>
    </div>

    {#if item.type === 'color'}
      <p class="font-mono text-sm text-surface-900 dark:text-surface-100 truncate mt-0.5">{item.content}</p>
    {:else if item.type !== 'image'}
      <pre class="text-[13px] text-surface-900 dark:text-surface-100 whitespace-pre-wrap break-words font-mono mt-0.5 {compact ? 'line-clamp-1' : 'line-clamp-2'}">{item.preview}</pre>
    {:else}
      <p class="text-[13px] text-surface-700 dark:text-surface-300 truncate mt-0.5">{item.preview}</p>
    {/if}

    <div class="flex items-center gap-2 mt-1 text-[11px] text-surface-400 dark:text-surface-500">
      {#if item.type !== 'color'}<span>{formatSize(item.size)}</span>{/if}
      {#if item.ocrText}<span class="text-primary-600 dark:text-primary-400">🔍 OCR</span>{/if}
    </div>
  </div>

  <div class="flex flex-col gap-0.5 opacity-0 group-hover:opacity-100 focus-within:opacity-100 transition-opacity flex-shrink-0">
    {#if onPaste && !compact}
      <button class="btn-ghost p-1 text-xs" onclick={handlePaste} title={localeStore.t('itemPasteTitle')} aria-label={localeStore.t('itemPasteTitle')}>
        ⤵️
      </button>
    {/if}
    {#if onDelete}
      <button class="btn-ghost p-1 text-xs" onclick={handleDelete} title={localeStore.t('itemDeleteTitle')} aria-label={localeStore.t('itemDeleteTitle')}>
        🗑️
      </button>
    {/if}
  </div>
</div>
