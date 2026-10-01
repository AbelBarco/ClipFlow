<script lang="ts">
  import Kbd from '$lib/features/ui/components/Kbd.svelte';

  interface Props {
    value: string;
    placeholder?: string;
    autoFocus?: boolean;
  }

  let { value = $bindable(''), placeholder = 'Search...', autoFocus = false }: Props = $props();

  let inputRef: HTMLInputElement | undefined = $state(undefined);

  /** Enfoca (y selecciona) el buscador: lo usa el atajo "/" y spotlight. */
  export function focus() {
    inputRef?.focus();
    inputRef?.select();
  }

  $effect(() => {
    if (inputRef && autoFocus) {
      inputRef.focus();
    }
  });
</script>

<div class="relative">
  <svg class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-surface-400 dark:text-surface-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
  </svg>
  <input
    bind:this={inputRef}
    bind:value
    type="text"
    {placeholder}
    class="input pl-10"
    aria-label={placeholder}
  />
  <div class="absolute right-3 top-1/2 -translate-y-1/2 flex items-center gap-1 text-xs text-surface-400 dark:text-surface-500">
    <Kbd keys={['/']} />
  </div>
</div>
