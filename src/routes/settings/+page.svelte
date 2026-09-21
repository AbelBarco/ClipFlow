<script lang="ts">
  import { onMount } from 'svelte';
  import GeneralSettings from '$lib/features/settings/components/GeneralSettings.svelte';
  import LanguageSettings from '$lib/features/settings/components/LanguageSettings.svelte';
  import ExclusionSettings from '$lib/features/settings/components/ExclusionSettings.svelte';
  import OcrSettings from '$lib/features/settings/components/OcrSettings.svelte';
  import CorrectorSettings from '$lib/features/settings/components/CorrectorSettings.svelte';
  import { settingsStore } from '$lib/features/settings/stores/settings.svelte';
  import { localeStore } from '$lib/features/i18n/stores/locale.svelte';
  import { themeStore } from '$lib/features/ui/stores/theme.svelte';
  import { DEFAULT_LOCALE } from '$lib/features/i18n/translations';

  onMount(() => {
    void settingsStore.load().then(() => {
      // El tema guardado se aplica de inmediato.
      themeStore.setTheme(settingsStore.settings.general.theme);
      // El idioma guardado en el backend manda; si es el valor por defecto
      // de una instalación nueva, se respeta la detección local
      // (localStorage > idioma del navegador).
      const saved = settingsStore.settings.general.language;
      if (saved && saved !== DEFAULT_LOCALE) {
        localeStore.setLocale(saved);
      } else {
        localeStore.init();
      }
    });
  });

  async function handleSave() {
    try {
      await settingsStore.save();
    } catch (e) {
      console.error('Save failed:', e);
    }
  }

  async function handleReset() {
    try {
      await settingsStore.resetToDefaults();
      themeStore.setTheme(settingsStore.settings.general.theme);
      localeStore.setLocale(settingsStore.settings.general.language);
    } catch (e) {
      console.error('Reset failed:', e);
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
    <OcrSettings />
    <CorrectorSettings />
  </div>

  <footer class="flex justify-end gap-2 mt-6 pt-4 border-t border-surface-200 dark:border-surface-700">
    <button class="btn-secondary" onclick={() => void handleReset()}>
      {localeStore.t('settingsReset')}
    </button>
    <button class="btn-primary" onclick={() => void handleSave()}>
      {localeStore.t('settingsSave')}
    </button>
  </footer>
</div>
