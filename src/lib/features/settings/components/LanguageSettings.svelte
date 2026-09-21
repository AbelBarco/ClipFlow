<script lang="ts">
  import { LANGUAGES } from '$lib/features/i18n/translations';
  import { localeStore } from '$lib/features/i18n/stores/locale.svelte';
  import { settingsStore } from '../stores/settings.svelte';

  let savedFlash = $state(false);
  let flashTimer: ReturnType<typeof setTimeout> | undefined = $state(undefined);

  function handleLanguageChange(e: Event) {
    const select = e.currentTarget as HTMLSelectElement;
    // Aplica al instante a toda la app y lo deja pendiente de "Guardar"
    // para persistirlo también en el backend.
    localeStore.setLocale(select.value);
    settingsStore.updateGeneral({ language: localeStore.locale });
    savedFlash = true;
    if (flashTimer) clearTimeout(flashTimer);
    flashTimer = setTimeout(() => {
      savedFlash = false;
    }, 2200);
  }
</script>

<fieldset class="card p-4 space-y-4">
  <legend class="text-sm font-medium text-surface-900 dark:text-surface-50">
    🌐 {localeStore.t('langTitle')}
  </legend>

  <p class="text-sm text-surface-500 dark:text-surface-400">
    {localeStore.t('langDesc')}
  </p>

  <div>
    <label
      class="block text-sm font-medium text-surface-700 dark:text-surface-300 mb-2"
      for="ui-language"
    >
      {localeStore.t('langLabel')}
    </label>
    <select
      id="ui-language"
      class="input w-[240px]"
      value={settingsStore.settings.general.language}
      onchange={handleLanguageChange}
    >
      {#each LANGUAGES as lang}
        <option value={lang.code}>
          {lang.flag} {lang.name}
        </option>
      {/each}
    </select>
    <p class="text-xs text-surface-500 dark:text-surface-400 mt-1">
      {localeStore.t('langHint')}
      {#if savedFlash}
        <span class="text-primary-600 dark:text-primary-400 font-medium">
          {localeStore.t('langApplied')}
        </span>
      {/if}
    </p>
  </div>
</fieldset>
