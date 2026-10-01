<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import ColorPreview from '$lib/features/color/components/ColorPreview.svelte';
  import TransformerMenu from '$lib/features/transformers/components/TransformerMenu.svelte';
  import type { Transformer } from '$lib/features/transformers/types/transformers.types';
  import type { ColorConversion } from '$lib/features/color/types/color.types';
  import { localeStore } from '$lib/features/i18n/stores/locale.svelte';
  import type { ClipItem } from '../types/clipboard.types';
  import {
    copyToClipboard,
    getImageDataUrlCached,
    runOcr
  } from '../api/clipboard';
  import { clipboardStore } from '../stores/clipboard.svelte';

  interface Props {
    item: ClipItem | null;
    onPaste: (itemId: string) => Promise<void>;
    onDelete: (itemId: string) => Promise<void>;
    notify: (msg: string) => void;
  }

  let { item, onPaste, onDelete, notify }: Props = $props();

  // --- Imagen ---
  let imageUrl: string | null = $state(null);
  let imageLoading = $state(false);
  let ocrRunning = $state(false);

  // --- Color ---
  let colorConv: ColorConversion | null = $state(null);

  // --- Transformadores ---
  let showTransformers = $state(false);
  let transformResult: string | null = $state(null);
  let transformRunning = $state(false);

  // Guardas anti-carrera: navegando rápido con ↑/↓, una promesa vieja no debe
  // pintar datos de otro item.
  let imageReq = 0;
  let colorReq = 0;
  let transformReq = 0;

  // Recarga los datos derivados cada vez que cambia el item seleccionado.
  $effect(() => {
    const current = item;
    imageUrl = null;
    colorConv = null;
    transformResult = null;
    showTransformers = false;
    transformRunning = false;
    // Invalida transformaciones en vuelo del item anterior.
    transformReq++;

    if (!current) return;

    if (current.type === 'image') {
      const req = ++imageReq;
      imageLoading = true;
      getImageDataUrlCached(current.id)
        .then((url) => {
          if (req !== imageReq) return;
          imageUrl = url;
        })
        .catch(() => {
          if (req !== imageReq) return;
          imageUrl = null;
        })
        .finally(() => {
          if (req !== imageReq) return;
          imageLoading = false;
        });
    } else if (current.type === 'color') {
      const req = ++colorReq;
      const content = current.content;
      invoke<ColorConversion>('color_convert', { input: content })
        .then((conv) => {
          if (req !== colorReq) return;
          colorConv = conv;
        })
        .catch(() => {
          if (req !== colorReq) return;
          colorConv = null;
        });
    }
  });

  function dimensions(): string | null {
    const meta = item?.metadata as { width?: number; height?: number } | undefined;
    if (meta && typeof meta.width === 'number' && typeof meta.height === 'number') {
      return `${meta.width} × ${meta.height} px`;
    }
    return null;
  }

  async function copyText(text: string, label: string) {
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
    notify(localeStore.t('toastCopiedLabel', { label }));
  }

  async function copyImage() {
    if (!item) return;
    try {
      await copyToClipboard(item.content, 'image');
      notify(localeStore.t('toastCopiedImage'));
    } catch (e) {
      console.error('Copy image failed:', e);
      notify(localeStore.t('toastImageCopyFailed'));
    }
  }

  async function handleOcr() {
    if (!item || ocrRunning) return;
    ocrRunning = true;
    try {
      const text = await runOcr(item.id);
      clipboardStore.updateItem(item.id, { ocrText: text });
      notify(text ? localeStore.t('toastExtracted') : localeStore.t('toastNoText'));
    } catch (e) {
      console.error('OCR failed:', e);
      notify(localeStore.t('toastOcrUnavailable'));
    } finally {
      ocrRunning = false;
    }
  }

  async function handleOpenUrl() {
    if (!item) return;
    try {
      const { openUrl } = await import('@tauri-apps/plugin-opener');
      await openUrl(item.content);
    } catch {
      window.open(item.content, '_blank', 'noopener');
    }
  }

  async function handleTransform(t: Transformer) {
    if (!item) return;
    const req = ++transformReq;
    const source = item.content;
    transformRunning = true;
    try {
      const out = await invoke<string>('transform_apply', {
        text: source,
        transformer: t.id
      });
      if (req !== transformReq) return;
      transformResult = out;
    } catch (e) {
      console.error('Transform failed:', e);
      notify(localeStore.t('toastTransformFailed'));
    } finally {
      if (req === transformReq) transformRunning = false;
    }
  }
</script>

<div class="card p-3">
  {#if !item}
    <p class="text-sm text-surface-500 dark:text-surface-400 text-center py-4">
      {localeStore.t('previewSelect')}
    </p>
  {:else}
    <div class="flex items-center gap-2 mb-2">
      <span class="text-xs font-semibold uppercase tracking-wide text-surface-500 dark:text-surface-400">
        {localeStore.t('previewTitle', { type: item.type })}
      </span>
      {#if dimensions()}
        <span class="text-xs text-surface-400 dark:text-surface-500 ml-auto">{dimensions()}</span>
      {/if}
    </div>

    {#if item.type === 'image'}
      <div class="rounded-lg overflow-hidden bg-surface-100 dark:bg-surface-800 flex items-center justify-center min-h-[120px] max-h-[260px]">
        {#if imageLoading}
          <p class="text-xs text-surface-500 p-6">{localeStore.t('previewLoadingImage')}</p>
        {:else if imageUrl}
          <img src={imageUrl} alt={localeStore.t('previewTitle', { type: item.type })} class="max-h-[260px] w-auto max-w-full object-contain" />
        {:else}
          <p class="text-xs text-surface-500 p-6">{localeStore.t('previewImageFailed')}</p>
        {/if}
      </div>
      {#if item.ocrText}
        <p class="mt-2 text-xs text-surface-600 dark:text-surface-300 bg-surface-100 dark:bg-surface-800 rounded-lg p-2 max-h-20 overflow-y-auto">
          🔍 {item.ocrText}
        </p>
      {/if}
      <div class="flex flex-wrap gap-1.5 mt-2">
        <button class="btn-secondary text-xs" onclick={copyImage} disabled={!imageUrl}>🖼️ {localeStore.t('previewCopyImage')}</button>
        <button class="btn-secondary text-xs" onclick={() => onPaste(item.id)}>⤵️ {localeStore.t('previewPaste')}</button>
        <button class="btn-secondary text-xs" onclick={handleOcr} disabled={ocrRunning}>
          {ocrRunning ? localeStore.t('previewReading') : `🔍 ${localeStore.t('previewOcr')}`}
        </button>
        <button class="btn-ghost text-xs text-red-500" onclick={() => onDelete(item.id)} aria-label={localeStore.t('itemDeleteTitle')}>🗑️</button>
      </div>
    {:else if item.type === 'color'}
      <div class="flex items-center gap-3">
        <ColorPreview color={item.content} size="lg" />
        <div class="flex-1 min-w-0">
          <p class="font-mono text-sm text-surface-900 dark:text-surface-50 truncate">{item.content}</p>
          <p class="text-xs text-surface-500">{localeStore.t('previewClickFormat')}</p>
        </div>
      </div>
      {#if colorConv}
        <div class="grid grid-cols-1 gap-1 mt-2">
          {#each [['HEX', colorConv.hex], ['RGB', colorConv.rgb], ['HSL', colorConv.hsl], ['CSS', colorConv.css]] as [label, value]}
            <button
              class="flex items-center gap-2 text-left px-2 py-1.5 rounded-lg hover:bg-surface-100 dark:hover:bg-surface-800 transition-colors"
              onclick={() => copyText(value, label)}
              title={localeStore.t('previewClickFormat')}
            >
              <span class="text-[10px] font-bold text-surface-500 w-8">{label}</span>
              <span class="font-mono text-xs text-surface-900 dark:text-surface-100 truncate flex-1">{value}</span>
              <span class="text-xs">📋</span>
            </button>
          {/each}
        </div>
      {/if}
      <div class="flex flex-wrap gap-1.5 mt-2">
        <button class="btn-secondary text-xs" onclick={() => onPaste(item.id)}>📋 {localeStore.t('previewPaste')}</button>
        <button class="btn-ghost text-xs text-red-500" onclick={() => onDelete(item.id)} aria-label={localeStore.t('itemDeleteTitle')}>🗑️</button>
      </div>
    {:else}
      {#if item.type === 'url'}
        <button
          class="text-sm text-primary-600 dark:text-primary-400 hover:underline break-all text-left"
          onclick={handleOpenUrl}
          title={localeStore.t('previewOpen')}
        >
          🔗 {item.content}
        </button>
      {:else}
        <pre class="text-xs text-surface-900 dark:text-surface-100 whitespace-pre-wrap break-words font-mono bg-surface-100 dark:bg-surface-800 rounded-lg p-2 max-h-[160px] overflow-y-auto">{item.content}</pre>
      {/if}
      <p class="text-[11px] text-surface-400 mt-1">{localeStore.t('previewChars', { count: [...item.content].length })}</p>
      {#if transformResult !== null}
        <pre class="text-xs whitespace-pre-wrap break-words font-mono bg-primary-50 dark:bg-primary-900/20 border border-primary-200 dark:border-primary-800 rounded-lg p-2 max-h-[120px] overflow-y-auto mt-2">{transformResult}</pre>
      {/if}
      <div class="flex flex-wrap gap-1.5 mt-2">
        <button class="btn-secondary text-xs" onclick={() => copyText(item.content, localeStore.t('previewCopy'))}>📋 {localeStore.t('previewCopy')}</button>
        <button class="btn-secondary text-xs" onclick={() => onPaste(item.id)}>⤵️ {localeStore.t('previewPaste')}</button>
        {#if transformResult !== null}
          <button class="btn-secondary text-xs" onclick={() => copyText(transformResult ?? '', localeStore.t('previewCopyResult'))}>📋 {localeStore.t('previewCopyResult')}</button>
        {/if}
        {#if item.type === 'text' || item.type === 'code'}
          <button class="btn-ghost text-xs" onclick={() => (showTransformers = !showTransformers)}>
            ✨ {localeStore.t('previewTransform')} {transformRunning ? '…' : showTransformers ? '▲' : '▼'}
          </button>
        {/if}
        {#if item.type === 'url'}
          <button class="btn-ghost text-xs" onclick={handleOpenUrl}>🌐 {localeStore.t('previewOpen')}</button>
        {/if}
        <button class="btn-ghost text-xs text-red-500" onclick={() => onDelete(item.id)} aria-label={localeStore.t('itemDeleteTitle')}>🗑️</button>
      </div>
      {#if showTransformers}
        <div class="mt-2">
          <TransformerMenu onSelect={(t) => void handleTransform(t)} />
        </div>
      {/if}
    {/if}
  {/if}
</div>
