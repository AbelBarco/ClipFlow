<script lang="ts">
  import ColorPreview from '$lib/features/color/components/ColorPreview.svelte';
  import Kbd from '$lib/features/ui/components/Kbd.svelte';
  import type { ClipItem } from '../types/clipboard.types';

  interface Props {
    item: ClipItem;
    compact?: boolean;
    onPaste?: (itemId: string) => Promise<void>;
  }

  let { item, compact = false, onPaste }: Props = $props();

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

  function getTypeLabel(type: string): string {
    return type.charAt(0).toUpperCase() + type.slice(1);
  }

  function formatTimestamp(timestamp: number): string {
    const date = new Date(timestamp);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const minutes = Math.floor(diff / 60000);
    const hours = Math.floor(diff / 3600000);
    const days = Math.floor(diff / 86400000);

    if (minutes < 1) return 'Just now';
    if (minutes < 60) return `${minutes}m ago`;
    if (hours < 24) return `${hours}h ago`;
    if (days < 7) return `${days}d ago`;
    return date.toLocaleDateString();
  }

  async function handleClick() {
    if (onPaste) {
      await onPaste(item.id);
    }
  }

  async function handleCopy() {
    await navigator.clipboard.writeText(item.content);
  }
</script>

<button
  class="card flex items-start gap-3 p-3 w-full text-left item-hover group {compact ? 'p-2' : ''}"
  on:click={handleClick}
  role="listitem"
  tabindex="0"
  on:keydown={(e) => e.key === 'Enter' && handleClick()}
>
  <span class="text-lg flex-shrink-0 mt-0.5">{getTypeIcon(item.type)}</span>

  <div class="flex-1 min-w-0">
    <div class="flex items-center gap-2 mb-1">
      <span class="text-xs font-medium text-surface-600 dark:text-surface-400 capitalize">
        {getTypeLabel(item.type)}
      </span>
      <span class="text-xs text-surface-400 dark:text-surface-500">
        {formatTimestamp(item.timestamp)}
      </span>
    </div>

    {#if item.type === 'color'}
      <ColorPreview color={item.content} />
    {:else if item.type === 'image'}
      <div class="aspect-video bg-surface-100 dark:bg-surface-800 rounded-lg overflow-hidden relative">
        <img src={item.content} alt="Clipboard image" class="w-full h-full object-cover" />
        {#if item.ocrText}
          <div class="absolute bottom-0 left-0 right-0 bg-black/70 text-white text-xs p-2">
            OCR: {item.ocrText.slice(0, 50)}{item.ocrText.length > 50 ? '...' : ''}
          </div>
        {/if}
      </div>
    {:else}
      <pre class="text-sm text-surface-900 dark:text-surface-100 whitespace-pre-wrap break-words font-mono {compact ? 'line-clamp-2' : 'line-clamp-4'}">
        {item.preview}
      </pre>
    {/if}

    {#if item.size > 0 && item.type !== 'color'}
      <div class="flex items-center gap-2 mt-2 text-xs text-surface-400 dark:text-surface-500">
        <span>{formatSize(item.size)}</span>
        {#if item.ocrText}
          <span class="flex items-center gap-1 text-primary-600 dark:text-primary-400">
            <span>🔍</span> OCR available
          </span>
        {/if}
      </div>
    {/if}
  </div>

  <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
    <button
      class="btn-ghost p-1.5"
      on:click={(e) => { e.stopPropagation(); handleCopy(); }}
      aria-label="Copy to clipboard"
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 5H6a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2v-1M8 5a2 2 0 002 2h2a2 2 0 002-2M8 5a2 2 0 012-2h2a2 2 0 012 2m0 0h2a2 2 0 012 2v3m2 4H10m0 0l3-3m-3 3l3 3" />
      </svg>
    </button>
  </div>
</button>

<script lang="ts">
  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }
</script>