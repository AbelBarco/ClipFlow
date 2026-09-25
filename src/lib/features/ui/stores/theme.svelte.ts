type Theme = "light" | "dark" | "system";

function isTheme(value: unknown): value is Theme {
  return value === "light" || value === "dark" || value === "system";
}

class ThemeStore {
  theme: Theme = $state("system");
  resolvedTheme: "light" | "dark" = $state("light");
  private initialized = false;

  constructor() {
    // Defer browser access to init() so this module is safe to import anywhere.
  }

  init(): void {
    if (this.initialized || typeof window === "undefined") {
      return;
    }
    this.initialized = true;
    try {
      const stored = localStorage.getItem("theme");
      if (isTheme(stored)) {
        this.theme = stored;
      }
    } catch {
      // Ignore storage errors (private mode, etc.)
    }
    this.updateResolvedTheme();
    this.applyTheme();
    this.watchSystemTheme();
  }

  /**
   * User action in THIS window: apply, remember locally and tell the other
   * windows (main/spotlight/settings each run their own JS context).
   */
  setTheme(theme: Theme): void {
    this.applyLocalTheme(theme);
    void this.broadcast(theme);
  }

  /**
   * Theme arriving from the backend or from another window: apply and
   * remember locally, but do NOT re-broadcast (avoids echo loops).
   */
  applyRemoteTheme(raw: unknown): void {
    if (!isTheme(raw)) {
      return;
    }
    if (raw === this.theme) {
      // Still re-apply: the OS theme may have changed under "system".
      this.updateResolvedTheme();
      this.applyTheme();
      return;
    }
    this.applyLocalTheme(raw);
  }

  private applyLocalTheme(theme: Theme): void {
    this.theme = theme;
    try {
      localStorage.setItem("theme", theme);
    } catch {
      // Ignore storage errors.
    }
    this.updateResolvedTheme();
    this.applyTheme();
  }

  private async broadcast(theme: Theme): Promise<void> {
    try {
      const { emit } = await import("@tauri-apps/api/event");
      await emit("theme-changed", { theme });
    } catch {
      // Not running inside Tauri (browser dev) — nothing to sync.
    }
  }

  private updateResolvedTheme(): void {
    if (typeof window === "undefined") {
      return;
    }
    if (this.theme === "system") {
      this.resolvedTheme = window.matchMedia("(prefers-color-scheme: dark)")
        .matches
        ? "dark"
        : "light";
    } else {
      this.resolvedTheme = this.theme;
    }
  }

  private watchSystemTheme(): void {
    if (typeof window === "undefined") {
      return;
    }
    const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
    mediaQuery.addEventListener("change", () => {
      if (this.theme === "system") {
        this.updateResolvedTheme();
        this.applyTheme();
      }
    });
  }

  private applyTheme(): void {
    if (typeof document === "undefined") {
      return;
    }
    const root = document.documentElement;
    if (this.resolvedTheme === "dark") {
      root.classList.add("dark");
    } else {
      root.classList.remove("dark");
    }
    // Scrollbars, form controls, etc. follow the resolved theme.
    root.style.colorScheme = this.resolvedTheme;
  }
}

export const themeStore = new ThemeStore();
