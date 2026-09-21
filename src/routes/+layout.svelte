<script lang="ts">
  import { themeStore } from '$lib/features/ui/stores/theme.svelte';
  import { onMount } from 'svelte';

  onMount(() => {
    themeStore.init();
  });
</script>

<slot />
