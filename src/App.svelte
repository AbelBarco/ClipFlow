<script lang="ts">
  import { onMount } from 'svelte';
  import MainPage from './routes/main/+page.svelte';
  import SpotlightPage from './routes/spotlight/+page.svelte';
  import SettingsPage from './routes/settings/+page.svelte';
  import { themeStore } from '$lib/features/ui/stores/theme.svelte';

  let windowLabel: string = $state('main');
  let ready: boolean = $state(false);

  onMount(async () => {
    // Apply persisted theme as early as possible.
    themeStore.init();

    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      windowLabel = getCurrentWindow().label;
    } catch {
      // Not running inside Tauri (e.g. `vite dev` in browser) — fall back
      // to the `?window=` query param so each view can still be previewed.
      const params = new URLSearchParams(window.location.search);
      windowLabel = params.get('window') || 'main';
    }
    ready = true;
  });
</script>

{#if ready}
  {#if windowLabel === 'spotlight'}
    <SpotlightPage />
  {:else if windowLabel === 'settings'}
    <SettingsPage />
  {:else}
    <MainPage />
  {/if}
{:else}
  <div class="flex items-center justify-center min-h-screen">
    <p class="text-sm text-surface-500">Loading ClipFlow…</p>
  </div>
{/if}

<style>
  :global(html, body, #app) {
    height: 100%;
  }
</style>
