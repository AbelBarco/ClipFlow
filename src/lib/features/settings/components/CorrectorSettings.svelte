<script lang="ts">
  import { LANGUAGES, htmlLangOf, resolveLocale } from '$lib/features/i18n/translations';
  import { localeStore } from '$lib/features/i18n/stores/locale.svelte';
  import { settingsStore } from '../stores/settings.svelte';
  import { copyToClipboard } from '$lib/features/clipboard/api/clipboard';
  import {
    analyzeText,
    applyAllIssues,
    applyIssue,
    countChars,
    countWords,
    type CorrectorIssue
  } from '$lib/features/corrector/corrector';

  let text = $state('');
  let copiedFlash = $state(false);
  let copyTimer: ReturnType<typeof setTimeout> | undefined = $state(undefined);

  // Idioma efectivo del corrector: el elegido o el de la interfaz ('auto').
  const effectiveLang = $derived(
    settingsStore.settings.corrector.language === 'auto'
      ? localeStore.locale
      : resolveLocale(settingsStore.settings.corrector.language, localeStore.locale)
  );

  const issues: CorrectorIssue[] = $derived(
    settingsStore.settings.corrector.enabled ? analyzeText(text, effectiveLang) : []
  );

  const words = $derived(countWords(text, effectiveLang));
  const chars = $derived(countChars(text));

  function issueLabel(type: CorrectorIssue['type']): string {
    switch (type) {
      case 'doublespace':
        return localeStore.t('issueDoublespace');
      case 'repeat':
        return localeStore.t('issueRepeat');
      case 'punct':
        return localeStore.t('issuePunct');
      case 'spaceAfter':
        return localeStore.t('issueSpaceAfter');
      case 'caps':
        return localeStore.t('issueCaps');
      case 'typo':
        return localeStore.t('issueTypo');
    }
  }

  function handleInput(e: Event) {
    const el = e.currentTarget as HTMLTextAreaElement;
    let next = el.value;
    // Autocorrección: aplica las correcciones seguras al escribir.
    if (settingsStore.settings.corrector.enabled && settingsStore.settings.corrector.autoCorrect) {
      const found = analyzeText(next, effectiveLang);
      if (found.length > 0) {
        next = applyAllIssues(next, found);
        el.value = next;
      }
    }
    text = next;
  }

  function fixIssue(issue: CorrectorIssue) {
    text = applyIssue(text, issue);
  }

  function fixAll() {
    text = applyAllIssues(text, issues);
  }

  async function copyResult() {
    if (!text) return;
    try {
      await copyToClipboard(text, 'text');
    } catch {
      try {
        await navigator.clipboard.writeText(text);
      } catch (e) {
        console.error('Copy failed:', e);
        return;
      }
    }
    copiedFlash = true;
    if (copyTimer) clearTimeout(copyTimer);
    copyTimer = setTimeout(() => {
      copiedFlash = false;
    }, 2000);
  }
</script>

<fieldset class="card p-4 space-y-4">
  <legend class="text-sm font-medium text-surface-900 dark:text-surface-50">
    ✍️ {localeStore.t('corrTitle')}
  </legend>

  <p class="text-sm text-surface-500 dark:text-surface-400">
    {localeStore.t('corrDesc')}
  </p>

  <div class="flex items-center justify-between">
    <div>
      <label class="text-sm font-medium text-surface-700 dark:text-surface-300">
        {localeStore.t('corrEnable')}
      </label>
      <p class="text-xs text-surface-500 dark:text-surface-400">{localeStore.t('corrEnableHint')}</p>
    </div>
    <input
      type="checkbox"
      bind:checked={settingsStore.settings.corrector.enabled}
      class="w-5 h-5 rounded border-surface-300 text-primary-600 focus:ring-primary-500"
      onchange={() =>
        settingsStore.updateCorrector({ enabled: settingsStore.settings.corrector.enabled })}
    />
  </div>

  <div>
    <label
      class="block text-sm font-medium text-surface-700 dark:text-surface-300 mb-2"
      for="corrector-language"
    >
      {localeStore.t('corrLanguage')}
    </label>
    <select
      id="corrector-language"
      bind:value={settingsStore.settings.corrector.language}
      class="input w-[240px]"
      onchange={() =>
        settingsStore.updateCorrector({ language: settingsStore.settings.corrector.language })}
    >
      <option value="auto">✨ {localeStore.t('corrFollowUi')}</option>
      {#each LANGUAGES as lang}
        <option value={lang.code}>{lang.flag} {lang.name}</option>
      {/each}
    </select>
    <p class="text-xs text-surface-500 dark:text-surface-400 mt-1">
      {localeStore.t('corrLanguageHint')}
    </p>
  </div>

  <div class="flex items-center justify-between">
    <div>
      <label class="text-sm font-medium text-surface-700 dark:text-surface-300">
        {localeStore.t('corrAuto')}
      </label>
      <p class="text-xs text-surface-500 dark:text-surface-400">{localeStore.t('corrAutoHint')}</p>
    </div>
    <input
      type="checkbox"
      bind:checked={settingsStore.settings.corrector.autoCorrect}
      disabled={!settingsStore.settings.corrector.enabled}
      class="w-5 h-5 rounded border-surface-300 text-primary-600 focus:ring-primary-500 disabled:opacity-40"
      onchange={() =>
        settingsStore.updateCorrector({
          autoCorrect: settingsStore.settings.corrector.autoCorrect
        })}
    />
  </div>

  {#if settingsStore.settings.corrector.enabled}
    <div class="pt-2 border-t border-surface-200 dark:border-surface-700 space-y-2">
      <textarea
        value={text}
        oninput={handleInput}
        rows="5"
        spellcheck="true"
        lang={htmlLangOf(effectiveLang)}
        placeholder={localeStore.t('corrPlaceholder')}
        aria-label={localeStore.t('corrTitle')}
        class="input font-sans resize-y min-h-[120px] leading-relaxed"
      ></textarea>
      <p class="text-[11px] text-surface-400 dark:text-surface-500">
        💡 {localeStore.t('corrNativeHint')}
      </p>

      <div class="flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-surface-500 dark:text-surface-400">
        <span>📝 {words} {localeStore.t('corrWords')}</span>
        <span>🔤 {chars} {localeStore.t('corrChars')}</span>
        {#if issues.length > 0}
          <span class="font-semibold text-amber-600 dark:text-amber-400">
            ⚠️ {localeStore.t('corrErrors', { n: issues.length })}
          </span>
        {:else if text.trim()}
          <span class="font-medium text-green-600 dark:text-green-400">
            {localeStore.t('corrNoErrors')}
          </span>
        {/if}
      </div>

      {#if issues.length > 0}
        <ul class="space-y-1.5 max-h-[180px] overflow-y-auto pr-0.5">
          {#each issues as issue (issue.id)}
            <li
              class="flex items-center gap-2 text-xs bg-surface-100 dark:bg-surface-800 rounded-lg px-2 py-1.5"
            >
              <span class="font-semibold text-surface-700 dark:text-surface-200 flex-shrink-0">
                {issueLabel(issue.type)}
              </span>
              <span class="font-mono truncate flex-1 text-surface-500 dark:text-surface-400">
                “{issue.original}” → “{issue.suggestion}”
              </span>
              <button
                class="btn-secondary !px-2 !py-0.5 !text-[11px] flex-shrink-0"
                onclick={() => fixIssue(issue)}
              >
                {localeStore.t('corrApply')}
              </button>
            </li>
          {/each}
        </ul>
      {/if}

      <div class="flex flex-wrap gap-1.5">
        <button
          class="btn-secondary text-xs"
          onclick={fixAll}
          disabled={issues.length === 0}
        >
          ✨ {localeStore.t('corrApplyAll')}
        </button>
        <button class="btn-secondary text-xs" onclick={() => void copyResult()} disabled={!text}>
          📋 {copiedFlash ? localeStore.t('corrCopied') : localeStore.t('corrCopy')}
        </button>
        <button class="btn-ghost text-xs text-red-500" onclick={() => (text = '')} disabled={!text}>
          🗑️ {localeStore.t('corrClear')}
        </button>
      </div>
    </div>
  {/if}
</fieldset>
