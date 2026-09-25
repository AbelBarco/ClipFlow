<script lang="ts">
  import { localeStore } from '$lib/features/i18n/stores/locale.svelte';
  import { settingsStore } from '../stores/settings.svelte';

  let newApp: string = $state('');

  function addApp() {
    if (newApp.trim()) {
      settingsStore.addExcludedApp(newApp.trim());
      newApp = '';
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Enter') addApp();
  }
</script>

<fieldset class="card p-4 space-y-4">
  <legend class="text-sm font-medium text-surface-900 dark:text-surface-50">
    {localeStore.t('excTitle')}
  </legend>

  <p class="text-sm text-surface-500 dark:text-surface-400">
    {localeStore.t('excDesc')}
  </p>

  <div class="flex items-center gap-2">
    <input
      type="text"
      bind:value={newApp}
      placeholder={localeStore.t('excPlaceholder')}
      class="input flex-1"
      onkeydown={handleKeyDown}
    />
    <button class="btn-secondary" onclick={addApp} disabled={!newApp.trim()}>
      {localeStore.t('excAdd')}
    </button>
  </div>

  <div class="flex flex-wrap gap-2">
    {#each settingsStore.settings.exclusions.excludedApps as app}
      <span class="inline-flex items-center gap-1 px-2 py-1 text-sm bg-surface-100 dark:bg-surface-800 rounded-lg">
        {app}
        <button
          class="text-surface-500 hover:text-red-500"
          onclick={() => settingsStore.removeExcludedApp(app)}
          aria-label={`${localeStore.t('itemDeleteTitle')}: ${app}`}
        >
          ×
        </button>
      </span>
    {/each}
  </div>

  <div class="flex items-center justify-between pt-2 border-t border-surface-200 dark:border-surface-700">
    <div>
      <label for="respect-concealed" class="text-sm font-medium text-surface-700 dark:text-surface-300">
        {localeStore.t('excRespect')}
      </label>
      <p class="text-xs text-surface-500 dark:text-surface-400">{localeStore.t('excRespectHint')}</p>
    </div>
    <input
      id="respect-concealed"
      type="checkbox"
      bind:checked={settingsStore.settings.exclusions.respectConcealed}
      class="w-5 h-5 rounded border-surface-300 text-primary-600 focus:ring-primary-500"
      onchange={() => settingsStore.updateExclusions({ respectConcealed: settingsStore.settings.exclusions.respectConcealed })}
    />
  </div>

  {#if settingsStore.settings.exclusions.respectConcealed}
    <div>
      <label
        class="block text-sm font-medium text-surface-700 dark:text-surface-300 mb-2"
        for="heuristic-level"
      >
        {localeStore.t('excHeuristic')}
      </label>
      <select
        id="heuristic-level"
        bind:value={settingsStore.settings.exclusions.heuristicLevel}
        class="input w-[240px]"
        onchange={() =>
          settingsStore.updateExclusions({
            heuristicLevel: settingsStore.settings.exclusions.heuristicLevel
          })}
      >
        <option value="conservative">{localeStore.t('excSensConservative')}</option>
        <option value="standard">{localeStore.t('excSensStandard')}</option>
      </select>
      <p class="text-xs text-surface-500 dark:text-surface-400 mt-1">
        {localeStore.t('excHeuristicHint')}
      </p>
    </div>

    <div class="flex items-center justify-between">
      <div>
        <label for="notify-exclude" class="text-sm font-medium text-surface-700 dark:text-surface-300">
          {localeStore.t('excNotify')}
        </label>
        <p class="text-xs text-surface-500 dark:text-surface-400">{localeStore.t('excNotifyHint')}</p>
      </div>
      <input
        id="notify-exclude"
        type="checkbox"
        bind:checked={settingsStore.settings.exclusions.notifyOnExclude}
        class="w-5 h-5 rounded border-surface-300 text-primary-600 focus:ring-primary-500"
        onchange={() =>
          settingsStore.updateExclusions({
            notifyOnExclude: settingsStore.settings.exclusions.notifyOnExclude
          })}
      />
    </div>
  {/if}
</fieldset>
