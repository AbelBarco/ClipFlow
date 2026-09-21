<script lang="ts">
  import { localeStore } from '$lib/features/i18n/stores/locale.svelte';
  import { transformersStore } from '../stores/transformers.svelte';
  import type { Transformer } from '../types/transformers.types';

  interface Props {
    onSelect: (transformer: Transformer) => void;
  }

  let { onSelect }: Props = $props();

  const categories = [
    { id: 'text', labelKey: 'catText' },
    { id: 'code', labelKey: 'catCode' },
    { id: 'formatting', labelKey: 'catFormatting' },
    { id: 'encoding', labelKey: 'catEncoding' }
  ] as const;
</script>

<div class="card p-2 min-w-[200px]" role="menu">
  {#each categories as category}
    <div class="mb-2 last:mb-0">
      <div class="px-2 py-1 text-xs font-semibold text-surface-500 dark:text-surface-400 uppercase tracking-wide">
        {localeStore.t(category.labelKey)}
      </div>
      {#each transformersStore.getTransformersByCategory(category.id) as transformer}
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
