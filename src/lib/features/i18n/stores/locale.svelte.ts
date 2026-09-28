import {
  DEFAULT_LOCALE,
  htmlLangOf,
  isLocale,
  resolveLocale,
  translations,
  type LocaleCode,
  type TranslationKey,
} from "../translations";

const STORAGE_KEY = "clipflow-locale";

class LocaleStore {
  /** Idioma activo de la interfaz. Cambiarlo re-renderiza toda la app. */
  locale: LocaleCode = $state(DEFAULT_LOCALE);
  private initialized = false;

  /**
   * Inicializa el idioma. Prioridad: ajustes persistidos > localStorage >
   * idioma del navegador > español.
   */
  init(persisted?: string | null): void {
    if (this.initialized) return;
    this.initialized = true;

    let next: LocaleCode = DEFAULT_LOCALE;
    if (persisted && isLocale(persisted)) {
      next = persisted;
    } else {
      try {
        const stored = localStorage.getItem(STORAGE_KEY);
        if (stored && isLocale(stored)) {
          next = stored;
        } else if (typeof navigator !== "undefined" && navigator.language) {
          next = resolveLocale(navigator.language, DEFAULT_LOCALE);
        }
      } catch {
        // Almacenamiento no disponible: se queda el valor por defecto.
      }
    }
    this.locale = next;
    this.applyHtmlLang();
  }

  /** Cambia el idioma al instante y lo recuerda para la próxima sesión. */
  setLocale(code: string): LocaleCode {
    const next = this.applyLocalLocale(code);
    void this.broadcast(next);
    return next;
  }

  /**
   * Idioma que llega del backend o de otra ventana: se aplica y recuerda
   * localmente, pero NO se re-emite (evita bucles de eco).
   */
  applyRemoteLocale(raw: unknown): LocaleCode {
    return this.applyLocalLocale(typeof raw === "string" ? raw : "");
  }

  /** ¿Hay una elección explícita guardada de una sesión anterior? */
  hasStoredLocale(): boolean {
    try {
      return isLocale(localStorage.getItem(STORAGE_KEY));
    } catch {
      return false;
    }
  }

  private applyLocalLocale(code: string): LocaleCode {
    const next = isLocale(code) ? code : DEFAULT_LOCALE;
    this.locale = next;
    try {
      localStorage.setItem(STORAGE_KEY, next);
    } catch {
      // Ignorar errores de almacenamiento.
    }
    this.applyHtmlLang();
    return next;
  }

  private async broadcast(locale: LocaleCode): Promise<void> {
    try {
      const { emit } = await import("@tauri-apps/api/event");
      await emit("locale-changed", { locale });
    } catch {
      // Fuera de Tauri (navegador en dev) — no hay nada que sincronizar.
    }
  }

  /** Traduce una clave e interpola `{vars}`. Con reactividad de Svelte 5. */
  t(key: TranslationKey, params?: Record<string, string | number>): string {
    const dict = translations[this.locale] ?? translations[DEFAULT_LOCALE];
    let text: string = dict[key] ?? translations[DEFAULT_LOCALE][key] ?? key;
    if (params) {
      for (const [name, value] of Object.entries(params)) {
        text = text.replaceAll(`{${name}}`, String(value));
      }
    }
    return text;
  }

  private applyHtmlLang(): void {
    if (typeof document === "undefined") return;
    document.documentElement.lang = htmlLangOf(this.locale);
    // Marca universal (sin subtítulo en un idioma concreto).
    document.title = "ClipFlow";
  }
}

export const localeStore = new LocaleStore();
export type { LocaleCode, TranslationKey };
