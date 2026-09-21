<script lang="ts">
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
  <legend class="text-sm font-medium text-surface-900 dark:text-surface-50">Excluded Applications</legend>

  <p class="text-sm text-surface-500 dark:text-surface-400">
    Items copied from these applications won't be saved to history.
    Password managers are excluded automatically when <strong>Respect concealed types</strong> is enabled.
  </p>

  <div class="flex items-center gap-2">
    <input
      type="text"
      bind:value={newApp}
      placeholder="Application name (e.g., 1Password)"
      class="input flex-1"
      onkeydown={handleKeyDown}
    />
    <button class="btn-secondary" onclick={addApp} disabled={!newApp.trim()}>Add</button>
  </div>

  <div class="flex flex-wrap gap-2">
    {#each settingsStore.settings.exclusions.excludedApps as app}
      <span class="inline-flex items-center gap-1 px-2 py-1 text-sm bg-surface-100 dark:bg-surface-800 rounded-lg">
        {app}
        <button
          class="text-surface-500 hover:text-red-500"
          onclick={() => settingsStore.removeExcludedApp(app)}
          aria-label={`Remove ${app}`}
        >
          ×
        </button>
      </span>
    {/each}
  </div>

  <div class="flex items-center justify-between pt-2 border-t border-surface-200 dark:border-surface-700">
    <div>
      <label class="text-sm font-medium text-surface-700 dark:text-surface-300">Respect Concealed Types</label>
      <p class="text-xs text-surface-500 dark:text-surface-400">Automatically exclude password fields and concealed input</p>
    </div>
    <input
      type="checkbox"
      bind:checked={settingsStore.settings.exclusions.respectConcealed}
      class="w-5 h-5 rounded border-surface-300 text-primary-600 focus:ring-primary-500"
      onchange={() => settingsStore.updateExclusions({ respectConcealed: settingsStore.settings.exclusions.respectConcealed })}
    />
  </div>
</fieldset>
