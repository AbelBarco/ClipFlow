<script lang="ts">
  import ClipboardItem from './ClipboardItem.svelte';
  import type { ClipItem } from '../types/clipboard.types';

  interface Props {
    items: ClipItem[];
    onPaste?: (itemId: string) => Promise<void>;
    compact?: boolean;
  }

  let { items, onPaste, compact = false }: Props = $props();
</script>

<div class="flex flex-col gap-1 overflow-y-auto" role="list" aria-label="Clipboard history">
  {#each items as item (item.id)}
    <ClipboardItem {item} {compact} {onPaste} />
  {/each}
</div>