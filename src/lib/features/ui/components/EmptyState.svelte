<script lang="ts">
  interface Props {
    title: string;
    description: string;
    compact?: boolean;
  }

  let { title, description, compact = false }: Props = $props();
</script>

<div class="flex flex-col items-center justify-center text-center py-8 {compact ? 'py-4' : ''} px-4">
  <div class="w-16 h-16 mx-auto mb-3 rounded-full bg-surface-100 dark:bg-surface-800 flex items-center justify-center {compact ? 'w-10 h-10' : ''}">
    <svg class="w-8 h-8 text-surface-400 dark:text-surface-500 {compact ? 'w-5 h-5' : ''}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 13h6m-3-3v6m-9 1V7a2 2 0 012-2h6l2 2h6a2 2 0 012 2v8a2 2 0 01-2 2H5a2 2 0 01-2-2z" />
    </svg>
  </div>
  <h3 class="text-lg font-medium text-surface-900 dark:text-surface-50 {compact ? 'text-base' : ''}">{title}</h3>
  <p class="text-sm text-surface-500 dark:text-surface-400 mt-1 {compact ? 'text-xs' : ''}">{description}</p>
</div>