<script lang="ts">
  import GeneralSettings from '$lib/features/settings/components/GeneralSettings.svelte';
  import ExclusionSettings from '$lib/features/settings/components/ExclusionSettings.svelte';
  import OcrSettings from '$lib/features/settings/components/OcrSettings.svelte';
  import { settingsStore } from '$lib/features/settings/stores/settings.svelte';
</script>

<div class="flex flex-col h-full min-h-[400px] w-[520px] p-4">
  <header class="mb-6">
    <h1 class="text-xl font-semibold text-surface-900 dark:text-surface-50">Settings</h1>
    <p class="text-sm text-surface-500 dark:text-surface-400 mt-1">Configure ClipFlow behavior</p>
  </header>

  <div class="flex-1 overflow-y-auto space-y-6">
    <GeneralSettings />
    <ExclusionSettings />
    <OcrSettings />
  </div>

  <footer class="flex justify-end gap-2 mt-6 pt-4 border-t border-surface-200 dark:border-surface-700">
    <button class="btn-secondary" on:click={() => settingsStore.reset()}>
      Reset to defaults
    </button>
    <button class="btn-primary" on:click={() => settingsStore.save()}>
      Save changes
    </button>
  </footer>
</div>