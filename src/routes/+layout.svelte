<script lang="ts">
  import { themeStore } from '$lib/features/ui/stores/theme.svelte';
  import { localeStore } from '$lib/features/i18n/stores/locale.svelte';
  import { onMount } from 'svelte';

  onMount(() => {
    themeStore.init();
    localeStore.init();
  });
</script>

<slot />
