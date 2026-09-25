<script lang="ts">
  import { localeStore } from '$lib/features/i18n/stores/locale.svelte';
  import { settingsStore } from '../stores/settings.svelte';
  import { setEncryption } from '../api/settings';

  let working = $state(false);
  let feedback: string | null = $state(null);
  let feedbackOk = $state(true);
  let feedbackTimer: ReturnType<typeof setTimeout> | undefined = $state(undefined);

  function flash(msg: string, ok: boolean) {
    feedback = msg;
    feedbackOk = ok;
    if (feedbackTimer) clearTimeout(feedbackTimer);
    feedbackTimer = setTimeout(() => {
      feedback = null;
    }, 4000);
  }

  async function handleToggle(e: Event) {
    const input = e.currentTarget as HTMLInputElement;
    const enabled = input.checked;
    // Revierte visualmente hasta que el backend confirme: la migración
    // puede fallar (llavero no disponible) y el ajuste nunca debe mentir.
    input.checked = settingsStore.settings.security.encryptionEnabled;
    working = true;
    feedback = null;
    try {
      const converted = await setEncryption(enabled);
      settingsStore.updateSecurity({ encryptionEnabled: enabled });
      flash(localeStore.t('secDone', { n: converted }), true);
    } catch (err) {
      console.error('Encryption toggle failed:', err);
      flash(localeStore.t('secError'), false);
    } finally {
      working = false;
    }
  }
</script>

<fieldset class="card p-4 space-y-4">
  <legend class="text-sm font-medium text-surface-900 dark:text-surface-50">
    🔒 {localeStore.t('secTitle')}
  </legend>

  <p class="text-sm text-surface-500 dark:text-surface-400">
    {localeStore.t('secDesc')}
  </p>

  <div class="flex items-center justify-between">
    <div>
      <label for="sec-encrypt" class="text-sm font-medium text-surface-700 dark:text-surface-300">
        {localeStore.t('secEncrypt')}
      </label>
      <p class="text-xs text-surface-500 dark:text-surface-400">{localeStore.t('secEncryptHint')}</p>
      {#if working}
        <p class="text-xs text-primary-600 dark:text-primary-400 mt-1">
          {localeStore.t('secWorking')}
        </p>
      {:else if feedback}
        <p
          class="text-xs mt-1 font-medium {feedbackOk
            ? 'text-green-600 dark:text-green-400'
            : 'text-red-500'}"
        >
          {feedback}
        </p>
      {/if}
    </div>
    <input
      id="sec-encrypt"
      type="checkbox"
      checked={settingsStore.settings.security.encryptionEnabled}
      disabled={working}
      class="w-5 h-5 rounded border-surface-300 text-primary-600 focus:ring-primary-500 disabled:opacity-40"
      onchange={handleToggle}
    />
  </div>
</fieldset>
