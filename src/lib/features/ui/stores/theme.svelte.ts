type Theme = 'light' | 'dark' | 'system';

class ThemeStore {
  theme: Theme = $state('system');
  resolvedTheme: 'light' | 'dark' = $state('light');

  constructor() {
    if (typeof window !== 'undefined') {
      const stored = localStorage.getItem('theme') as Theme | null;
      if (stored) {
        this.theme = stored;
      }
      this.updateResolvedTheme();
      this.watchSystemTheme();
    }
  }

  setTheme(theme: Theme): void {
    this.theme = theme;
    localStorage.setItem('theme', theme);
    this.updateResolvedTheme();
    this.applyTheme();
  }

  private updateResolvedTheme(): void {
    if (this.theme === 'system') {
      this.resolvedTheme = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
    } else {
      this.resolvedTheme = this.theme;
    }
  }

  private watchSystemTheme(): void {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    mediaQuery.addEventListener('change', () => {
      if (this.theme === 'system') {
        this.updateResolvedTheme();
        this.applyTheme();
      }
    });
  }

  private applyTheme(): void {
    const root = document.documentElement;
    if (this.resolvedTheme === 'dark') {
      root.classList.add('dark');
    } else {
      root.classList.remove('dark');
    }
  }
}

export const themeStore = new ThemeStore();