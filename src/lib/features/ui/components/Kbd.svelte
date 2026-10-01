<script lang="ts">
  interface Props {
    keys: string[];
  }

  let { keys }: Props = $props();
</script>

<div class="inline-flex items-center gap-1">
  {#each keys as key, i}
    <kbd class="px-1.5 py-0.5 text-[10px] font-mono font-medium bg-surface-100 dark:bg-surface-800 text-surface-700 dark:text-surface-300 rounded border border-surface-200 dark:border-surface-700">
      {key}
    </kbd>
    {#if i < keys.length - 1}
      <span class="text-[10px] text-surface-400 dark:text-surface-500">+</span>
    {/if}
  {/each}
</div>