<script lang="ts">
  import { onMount } from 'svelte';
  import GeneralSettings from '$lib/features/settings/components/GeneralSettings.svelte';
  import LanguageSettings from '$lib/features/settings/components/LanguageSettings.svelte';
  import ExclusionSettings from '$lib/features/settings/components/ExclusionSettings.svelte';
  import PrivacySettings from '$lib/features/settings/components/PrivacySettings.svelte';
  import OcrSettings from '$lib/features/settings/components/OcrSettings.svelte';
  import { settingsStore } from '$lib/features/settings/stores/settings.svelte';
  import { localeStore } from '$lib/features/i18n/stores/locale.svelte';
  import { themeStore } from '$lib/features/ui/stores/theme.svelte';
  import { DEFAULT_LOCALE, isLocale } from '$lib/features/i18n/translations';

  let savedFlash = $state(false);
  let saveError: boolean | null = $state(null);
  let flashTimer: ReturnType<typeof setTimeout> | undefined = $state(undefined);

  onMount(() => {
    const hadStoredLocale = localeStore.hasStoredLocale();
    void settingsStore.load().then(() => {
      // Silent apply: App.svelte already synced + listens for changes.
      // Broadcasting here would just echo back to every window.
      themeStore.applyRemoteTheme(settingsStore.settings.general.theme);
      // Misma regla que en App.svelte: el backend manda salvo que sea el
      // valor de fábrica sin elección previa (se respeta el navegador).
      const saved = settingsStore.settings.general.language;
      if (isLocale(saved) && (hadStoredLocale || saved !== DEFAULT_LOCALE)) {
        localeStore.applyRemoteLocale(saved);
      }
    });
  });

  async function handleSave() {
    saveError = null;
    try {
      await settingsStore.save();
      savedFlash = true;
      if (flashTimer) clearTimeout(flashTimer);
      flashTimer = setTimeout(() => {
        savedFlash = false;
      }, 2200);
    } catch (e) {
      console.error('Save failed:', e);
      saveError = true;
    }
  }

  async function handleReset() {
    saveError = null;
    try {
      await settingsStore.resetToDefaults();
      themeStore.setTheme(settingsStore.settings.general.theme);
      localeStore.setLocale(settingsStore.settings.general.language);
    } catch (e) {
      console.error('Reset failed:', e);
      saveError = true;
    }
  }
</script>

<div class="flex flex-col h-full min-h-[400px] w-full max-w-[520px] mx-auto p-4">
  <header class="mb-6">
    <h1 class="text-xl font-semibold text-surface-900 dark:text-surface-50">
      {localeStore.t('settingsTitle')}
    </h1>
    <p class="text-sm text-surface-500 dark:text-surface-400 mt-1">
      {localeStore.t('settingsSubtitle')}
    </p>
  </header>

  <div class="flex-1 overflow-y-auto space-y-6">
    <GeneralSettings />
    <LanguageSettings />
    <ExclusionSettings />
    <PrivacySettings />
    <OcrSettings />
  </div>

  <footer class="flex items-center justify-end gap-2 mt-6 pt-4 border-t border-surface-200 dark:border-surface-700">
    {#if savedFlash}
      <span class="text-xs font-medium text-green-600 dark:text-green-400 mr-auto">
        {localeStore.t('settingsSaved')}
      </span>
    {:else if saveError}
      <span class="text-xs font-medium text-red-500 mr-auto">
        {localeStore.t('settingsSaveFailed')}
      </span>
    {/if}
    <button class="btn-secondary" onclick={() => void handleReset()}>
      {localeStore.t('settingsReset')}
    </button>
    <button
      class="btn-primary"
      onclick={() => void handleSave()}
      disabled={!settingsStore.hasChanges}
    >
      {localeStore.t('settingsSave')}
    </button>
  </footer>
</div>
