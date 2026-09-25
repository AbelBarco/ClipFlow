type Theme = "light" | "dark" | "system";

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
      const stored = localStorage.getItem("theme") as Theme | null;
      if (stored === "light" || stored === "dark" || stored === "system") {
        this.theme = stored;
      }
    } catch {
      // Ignore storage errors (private mode, etc.)
    }
    this.updateResolvedTheme();
    this.applyTheme();
    this.watchSystemTheme();
  }

  setTheme(theme: Theme): void {
    this.theme = theme;
    try {
      localStorage.setItem("theme", theme);
    } catch {
      // Ignore storage errors.
    }
    this.updateResolvedTheme();
    this.applyTheme();
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
  }
}

export const themeStore = new ThemeStore();
