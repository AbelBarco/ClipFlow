<script lang="ts">
  import Kbd from '$lib/features/ui/components/Kbd.svelte';
  import { localeStore } from '$lib/features/i18n/stores/locale.svelte';
  import { themeStore } from '$lib/features/ui/stores/theme.svelte';
  import { settingsStore } from '../stores/settings.svelte';

  let shortcutInput: string = $state(settingsStore.settings.general.globalShortcut);
  let isRecording = $state(false);

  // Keep the input in sync if settings are (re)loaded from the backend.
  $effect(() => {
    if (!isRecording) {
      shortcutInput = settingsStore.settings.general.globalShortcut;
    }
  });

  function handleShortcutChange(e: KeyboardEvent) {
    // Escape cancela la grabación sin tocar el atajo (si no, "Escape" se
    // guardaría como atajo y habría que resetear ajustes para deshacerlo).
    if (e.key === 'Escape') {
      e.preventDefault();
      isRecording = false;
      (e.currentTarget as HTMLInputElement | null)?.blur();
      return;
    }
    e.preventDefault();
    const parts: string[] = [];
    if (e.ctrlKey || e.metaKey) parts.push(e.metaKey ? '⌘' : 'Ctrl');
    if (e.shiftKey) parts.push('⇧');
    if (e.altKey) parts.push(e.metaKey ? '⌥' : 'Alt');
    if (e.key.length === 1) parts.push(e.key.toUpperCase());
    else if (e.key !== 'Control' && e.key !== 'Shift' && e.key !== 'Alt' && e.key !== 'Meta') {
      parts.push(e.key);
    }
    if (parts.length > 0) {
      shortcutInput = parts.join('+');
      isRecording = false;
      settingsStore.updateGeneral({ globalShortcut: shortcutInput });
    }
  }

  function startRecording() {
    isRecording = true;
  }

  function stopRecording() {
    isRecording = false;
  }

  function handleThemeChange(e: Event) {
    const select = e.currentTarget as HTMLSelectElement;
    const theme = select.value as 'light' | 'dark' | 'system';
    settingsStore.updateGeneral({ theme });
    themeStore.setTheme(theme);
  }

  function handleMaxHistory(e: Event) {
    // bind:value en un input numérico entrega string: sin convertir, el
    // Guardar enviaría "500" y el backend (usize) lo rechazaría.
    const input = e.currentTarget as HTMLInputElement;
    const parsed = Number.parseInt(input.value, 10);
    const clamped = Number.isFinite(parsed)
      ? Math.min(5000, Math.max(50, parsed))
      : settingsStore.settings.general.maxHistoryItems;
    settingsStore.updateGeneral({ maxHistoryItems: clamped });
  }
</script>

<fieldset class="card p-4 space-y-4">
  <legend class="text-sm font-medium text-surface-900 dark:text-surface-50">
    {localeStore.t('generalTitle')}
  </legend>

  <div>
    <label for="global-shortcut" class="block text-sm font-medium text-surface-700 dark:text-surface-300 mb-2">
      {localeStore.t('generalShortcut')}
    </label>
    <div class="flex items-center gap-2">
      <input
        id="global-shortcut"
        type="text"
        bind:value={shortcutInput}
        class="input font-mono text-center"
        readonly
        onfocus={startRecording}
        onkeydown={handleShortcutChange}
        onblur={stopRecording}
        aria-label={localeStore.t('generalShortcut')}
      />
      {#if isRecording}
        <span class="text-xs text-primary-600 dark:text-primary-400">
          {localeStore.t('generalPressKeys')}
        </span>
      {:else}
        <Kbd keys={shortcutInput.split('+')} />
      {/if}
    </div>
    <p class="text-xs text-surface-500 dark:text-surface-400 mt-1">
      {localeStore.t('generalShortcutHint')}
    </p>
  </div>

  <div>
    <label for="max-history" class="block text-sm font-medium text-surface-700 dark:text-surface-300 mb-2">
      {localeStore.t('generalMaxHistory')}
    </label>
    <input
      id="max-history"
      type="number"
      value={settingsStore.settings.general.maxHistoryItems}
      min="50"
      max="5000"
      step="50"
      class="input w-[100px]"
      oninput={handleMaxHistory}
    />
    <p class="text-xs text-surface-500 dark:text-surface-400 mt-1">
      {localeStore.t('generalMaxHistoryHint')}
    </p>
  </div>

  <div class="flex items-center justify-between">
    <div>
      <label for="launch-startup" class="text-sm font-medium text-surface-700 dark:text-surface-300">
        {localeStore.t('generalLaunch')}
      </label>
      <p class="text-xs text-surface-500 dark:text-surface-400">{localeStore.t('generalLaunchHint')}</p>
    </div>
    <input
      id="launch-startup"
      type="checkbox"
      bind:checked={settingsStore.settings.general.launchAtStartup}
      class="w-5 h-5 rounded border-surface-300 text-primary-600 focus:ring-primary-500"
      onchange={() => settingsStore.updateGeneral({ launchAtStartup: settingsStore.settings.general.launchAtStartup })}
    />
  </div>

  <div class="flex items-center justify-between">
    <div>
      <label for="show-notif" class="text-sm font-medium text-surface-700 dark:text-surface-300">
        {localeStore.t('generalNotif')}
      </label>
      <p class="text-xs text-surface-500 dark:text-surface-400">{localeStore.t('generalNotifHint')}</p>
    </div>
    <input
      id="show-notif"
      type="checkbox"
      bind:checked={settingsStore.settings.general.showNotifications}
      class="w-5 h-5 rounded border-surface-300 text-primary-600 focus:ring-primary-500"
      onchange={() => settingsStore.updateGeneral({ showNotifications: settingsStore.settings.general.showNotifications })}
    />
  </div>

  <div>
    <label for="theme-select" class="block text-sm font-medium text-surface-700 dark:text-surface-300 mb-2">
      {localeStore.t('generalTheme')}
    </label>
    <select
      id="theme-select"
      value={settingsStore.settings.general.theme}
      class="input w-[180px]"
      onchange={handleThemeChange}
    >
      <option value="system">{localeStore.t('themeSystem')}</option>
      <option value="light">{localeStore.t('themeLight')}</option>
      <option value="dark">{localeStore.t('themeDark')}</option>
    </select>
  </div>
</fieldset>
