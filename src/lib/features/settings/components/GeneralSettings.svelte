<script lang="ts">
  import Kbd from '$lib/features/ui/components/Kbd.svelte';
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
</script>

<fieldset class="card p-4 space-y-4">
  <legend class="text-sm font-medium text-surface-900 dark:text-surface-50">General</legend>

  <div>
    <label class="block text-sm font-medium text-surface-700 dark:text-surface-300 mb-2">
      Global Shortcut
    </label>
    <div class="flex items-center gap-2">
      <input
        type="text"
        bind:value={shortcutInput}
        class="input font-mono text-center"
        readonly
        onfocus={startRecording}
        onkeydown={handleShortcutChange}
        onblur={stopRecording}
        aria-label="Global shortcut"
      />
      {#if isRecording}
        <span class="text-xs text-primary-600 dark:text-primary-400">Press keys...</span>
      {:else}
        <Kbd keys={shortcutInput.split('+')} />
      {/if}
    </div>
    <p class="text-xs text-surface-500 dark:text-surface-400 mt-1">Press keys to set shortcut</p>
  </div>

  <div>
    <label class="block text-sm font-medium text-surface-700 dark:text-surface-300 mb-2">
      Max History Items
    </label>
    <input
      type="number"
      bind:value={settingsStore.settings.general.maxHistoryItems}
      min="50"
      max="5000"
      step="50"
      class="input w-[100px]"
      oninput={() => settingsStore.updateGeneral({ maxHistoryItems: settingsStore.settings.general.maxHistoryItems })}
    />
    <p class="text-xs text-surface-500 dark:text-surface-400 mt-1">Older items will be removed automatically</p>
  </div>

  <div class="flex items-center justify-between">
    <div>
      <label class="text-sm font-medium text-surface-700 dark:text-surface-300">Launch at Startup</label>
      <p class="text-xs text-surface-500 dark:text-surface-400">Start ClipFlow when you log in</p>
    </div>
    <input
      type="checkbox"
      bind:checked={settingsStore.settings.general.launchAtStartup}
      class="w-5 h-5 rounded border-surface-300 text-primary-600 focus:ring-primary-500"
      onchange={() => settingsStore.updateGeneral({ launchAtStartup: settingsStore.settings.general.launchAtStartup })}
    />
  </div>

  <div class="flex items-center justify-between">
    <div>
      <label class="text-sm font-medium text-surface-700 dark:text-surface-300">Show Notifications</label>
      <p class="text-xs text-surface-500 dark:text-surface-400">Notify when items are copied</p>
    </div>
    <input
      type="checkbox"
      bind:checked={settingsStore.settings.general.showNotifications}
      class="w-5 h-5 rounded border-surface-300 text-primary-600 focus:ring-primary-500"
      onchange={() => settingsStore.updateGeneral({ showNotifications: settingsStore.settings.general.showNotifications })}
    />
  </div>

  <div>
    <label class="block text-sm font-medium text-surface-700 dark:text-surface-300 mb-2">Theme</label>
    <select
      bind:value={settingsStore.settings.general.theme}
      class="input w-[180px]"
      onchange={() => settingsStore.updateGeneral({ theme: settingsStore.settings.general.theme })}
    >
      <option value="system">System</option>
      <option value="light">Light</option>
      <option value="dark">Dark</option>
    </select>
  </div>
</fieldset>
