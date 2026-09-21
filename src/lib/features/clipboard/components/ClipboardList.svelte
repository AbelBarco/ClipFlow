<script lang="ts">
  import ClipboardItem from './ClipboardItem.svelte';
  import { localeStore } from '$lib/features/i18n/stores/locale.svelte';
  import type { ClipItem } from '../types/clipboard.types';

  interface Props {
    items: ClipItem[];
    selectedId?: string | null;
    onSelect?: (itemId: string) => void;
    onPaste?: (itemId: string) => Promise<void>;
    onDelete?: (itemId: string) => Promise<void>;
    compact?: boolean;
  }

  let { items, selectedId = null, onSelect, onPaste, onDelete, compact = false }: Props = $props();
</script>

<div class="flex flex-col gap-1.5 overflow-y-auto pr-0.5" role="listbox" aria-label={localeStore.t('itemHistory')}>
  {#each items as item (item.id)}
    <ClipboardItem
      {item}
      {compact}
      selected={selectedId === item.id}
      {onSelect}
      {onPaste}
      {onDelete}
    />
  {/each}
</div>
