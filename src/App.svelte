<script lang="ts">
  import { onMount } from "svelte";
  import MainPage from "./routes/main/+page.svelte";
  import SpotlightPage from "./routes/spotlight/+page.svelte";
  import SettingsPage from "./routes/settings/+page.svelte";
  import { themeStore } from "$lib/features/ui/stores/theme.svelte";
  import { localeStore } from "$lib/features/i18n/stores/locale.svelte";
  import { settingsStore } from "$lib/features/settings/stores/settings.svelte";
  import { DEFAULT_LOCALE, isLocale } from "$lib/features/i18n/translations";

  let windowLabel: string = $state("main");
  let ready: boolean = $state(false);

  onMount(async () => {
    // Apply persisted theme/language as early as possible (fast local path).
    // Se registra si había elección explícita ANTES de init(), porque init()
    // también la consume como fallback válido.
    const hadStoredLocale = localeStore.hasStoredLocale();
    themeStore.init();
    localeStore.init();

    // The backend is the source of truth for the theme: another window may
    // have saved a different value, and localStorage can be stale/empty.
    // Runs in every window (each has its own JS context).
    try {
      await settingsStore.load();
      themeStore.applyRemoteTheme(settingsStore.settings.general.theme);
      // ...y para el idioma, con un matiz: si no hay elección guardada en
      // local y el backend trae el valor por defecto de fábrica, se respeta
      // la detección del navegador en vez de imponer español.
      const savedLanguage = settingsStore.settings.general.language;
      if (isLocale(savedLanguage)) {
        if (hadStoredLocale || savedLanguage !== DEFAULT_LOCALE) {
          localeStore.applyRemoteLocale(savedLanguage);
        }
      }
    } catch {
      // Backend unreachable (browser dev) — keep the local value.
    }

    // Live sync: when the user changes the theme in the settings window,
    // the already-open main/spotlight windows follow instantly.
    try {
      const { listen } = await import("@tauri-apps/api/event");
      await listen<{ theme: string }>("theme-changed", (event) => {
        themeStore.applyRemoteTheme(event.payload?.theme);
      });
      // Lo mismo para el idioma: cambiarlo en ajustes re-renderiza
      // principal y spotlight al instante, sin reiniciar.
      await listen<{ locale: string }>("locale-changed", (event) => {
        localeStore.applyRemoteLocale(event.payload?.locale);
      });
    } catch {
      // Not running inside Tauri — nothing to sync.
    }

    try {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      windowLabel = getCurrentWindow().label;
    } catch {
      // Not running inside Tauri (e.g. `vite dev` in browser) — fall back
      // to the `?window=` query param so each view can still be previewed.
      const params = new URLSearchParams(window.location.search);
      windowLabel = params.get("window") || "main";
    }
    ready = true;
  });
</script>

{#if ready}
  {#if windowLabel === "spotlight"}
    <SpotlightPage />
  {:else if windowLabel === "settings"}
    <SettingsPage />
  {:else}
    <MainPage />
  {/if}
{:else}
  <div class="flex items-center justify-center min-h-screen">
    <p class="text-sm text-surface-500">{localeStore.t("appLoading")}</p>
  </div>
{/if}

<style>
  :global(html, body, #app) {
    height: 100%;
  }
</style>
