<script lang="ts">
  import { LANGUAGES } from '$lib/features/i18n/translations';
  import { localeStore } from '$lib/features/i18n/stores/locale.svelte';
  import { settingsStore } from '../stores/settings.svelte';

  // El selector de OCR muestra los idiomas con su nombre nativo y usa el
  // código Tesseract que el backend aplica de verdad al reconocer.
  const ocrLanguages = LANGUAGES.map((l) => ({ code: l.ocr, name: l.name, flag: l.flag }));
</script>

<fieldset class="card p-4 space-y-4">
  <legend class="text-sm font-medium text-surface-900 dark:text-surface-50">
    {localeStore.t('ocrTitle')}
  </legend>

  <p class="text-sm text-surface-500 dark:text-surface-400">
    {localeStore.t('ocrDesc')}
  </p>

  <div class="flex items-center justify-between">
    <div>
      <label for="ocr-enabled" class="text-sm font-medium text-surface-700 dark:text-surface-300">
        {localeStore.t('ocrEnable')}
      </label>
      <p class="text-xs text-surface-500 dark:text-surface-400">{localeStore.t('ocrEnableHint')}</p>
    </div>
    <input
      id="ocr-enabled"
      type="checkbox"
      bind:checked={settingsStore.settings.ocr.enabled}
      class="w-5 h-5 rounded border-surface-300 text-primary-600 focus:ring-primary-500"
      onchange={() => settingsStore.updateOcr({ enabled: settingsStore.settings.ocr.enabled })}
    />
  </div>

  {#if settingsStore.settings.ocr.enabled}
    <div>
      <label for="ocr-language" class="block text-sm font-medium text-surface-700 dark:text-surface-300 mb-2">
        {localeStore.t('ocrLanguage')}
      </label>
      <select
        id="ocr-language"
        bind:value={settingsStore.settings.ocr.language}
        class="input w-[200px]"
        onchange={() => settingsStore.updateOcr({ language: settingsStore.settings.ocr.language })}
      >
        {#each ocrLanguages as lang}
          <option value={lang.code}>{lang.flag} {lang.name}</option>
        {/each}
      </select>
    </div>

    <div class="flex items-center justify-between">
      <div>
        <label for="ocr-autorun" class="text-sm font-medium text-surface-700 dark:text-surface-300">
          {localeStore.t('ocrAutoRun')}
        </label>
        <p class="text-xs text-surface-500 dark:text-surface-400">{localeStore.t('ocrAutoRunHint')}</p>
      </div>
      <input
        id="ocr-autorun"
        type="checkbox"
        bind:checked={settingsStore.settings.ocr.autoRun}
        class="w-5 h-5 rounded border-surface-300 text-primary-600 focus:ring-primary-500"
        onchange={() => settingsStore.updateOcr({ autoRun: settingsStore.settings.ocr.autoRun })}
      />
    </div>
  {/if}
</fieldset>
