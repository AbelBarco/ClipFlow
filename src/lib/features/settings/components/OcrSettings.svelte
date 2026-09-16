<script lang="ts">
  import { settingsStore } from '../stores/settings.svelte';

  const languages = [
    { code: 'eng', name: 'English' },
    { code: 'spa', name: 'Spanish' },
    { code: 'fra', name: 'French' },
    { code: 'deu', name: 'German' },
    { code: 'chi_sim', name: 'Chinese (Simplified)' },
    { code: 'jpn', name: 'Japanese' },
    { code: 'kor', name: 'Korean' },
    { code: 'rus', name: 'Russian' },
    { code: 'por', name: 'Portuguese' },
    { code: 'ita', name: 'Italian' }
  ];
</script>

<fieldset class="card p-4 space-y-4">
  <legend class="text-sm font-medium text-surface-900 dark:text-surface-50">OCR (Optical Character Recognition)</legend>

  <p class="text-sm text-surface-500 dark:text-surface-400">
    Extract text from images in your clipboard history. Uses native OS APIs:
    Vision (macOS), WinRT (Windows), Tesseract (Linux).
  </p>

  <div class="flex items-center justify-between">
    <div>
      <label class="text-sm font-medium text-surface-700 dark:text-surface-300">Enable OCR</label>
      <p class="text-xs text-surface-500 dark:text-surface-400">Process images for text extraction</p>
    </div>
    <input
      type="checkbox"
      bind:checked={settingsStore.settings.ocr.enabled}
      class="w-5 h-5 rounded border-surface-300 text-primary-600 focus:ring-primary-500"
      on:change={() => settingsStore.updateOcr({ enabled: settingsStore.settings.ocr.enabled })}
    />
  </div>

  {#if settingsStore.settings.ocr.enabled}
    <div>
      <label class="block text-sm font-medium text-surface-700 dark:text-surface-300 mb-2">Language</label>
      <select
        bind:value={settingsStore.settings.ocr.language}
        class="input w-[200px]"
        on:change={() => settingsStore.updateOcr({ language: settingsStore.settings.ocr.language })}
      >
        {#each languages as lang}
          <option value={lang.code}>{lang.name}</option>
        {/each}
      </select>
    </div>

    <div class="flex items-center justify-between">
      <div>
        <label class="text-sm font-medium text-surface-700 dark:text-surface-300">Auto-run on new images</label>
        <p class="text-xs text-surface-500 dark:text-surface-400">Automatically run OCR when images are copied</p>
      </div>
      <input
        type="checkbox"
        bind:checked={settingsStore.settings.ocr.autoRun}
        class="w-5 h-5 rounded border-surface-300 text-primary-600 focus:ring-primary-500"
        on:change={() => settingsStore.updateOcr({ autoRun: settingsStore.settings.ocr.autoRun })}
      />
    </div>
  {/if}
</fieldset>