<script lang="ts">
  import { transformersStore } from '../stores/transformers.svelte';
  import type { Transformer } from '../types/transformers.types';

  interface Props {
    onSelect: (transformer: Transformer) => void;
  }

  let { onSelect }: Props = $props();

  const categories = ['text', 'code', 'formatting', 'encoding'] as const;

  function getCategoryLabel(category: string): string {
    return category.charAt(0).toUpperCase() + category.slice(1);
  }
</script>

<div class="card p-2 min-w-[200px]" role="menu">
  {#each categories as category}
    <div class="mb-2 last:mb-0">
      <div class="px-2 py-1 text-xs font-semibold text-surface-500 dark:text-surface-400 uppercase tracking-wide">
        {getCategoryLabel(category)}
      </div>
      {#each transformersStore.getTransformersByCategory(category) as transformer}
        <button
          class="w-full flex items-center justify-between px-2 py-1.5 text-sm text-surface-700 dark:text-surface-300 rounded-lg hover:bg-surface-100 dark:hover:bg-surface-800 transition-colors"
          role="menuitem"
          onclick={() => onSelect(transformer)}
        >
          <span>{transformer.label}</span>
          {#if transformer.shortcut}
            <span class="text-xs text-surface-400 dark:text-surface-500 font-mono">{transformer.shortcut}</span>
          {/if}
        </button>
      {/each}
    </div>
  {/each}
</div>
