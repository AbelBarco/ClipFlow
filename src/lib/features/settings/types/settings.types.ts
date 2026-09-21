export interface AppSettings {
  general: GeneralSettings;
  exclusions: ExclusionSettings;
  ocr: OcrSettings;
  corrector: CorrectorSettings;
}

export interface GeneralSettings {
  globalShortcut: string;
  maxHistoryItems: number;
  launchAtStartup: boolean;
  showNotifications: boolean;
  theme: 'light' | 'dark' | 'system';
  /** UI locale: en | es | fr | de | pt | it | zh | ja | ko | ru */
  language: string;
}

export interface ExclusionSettings {
  excludedApps: string[];
  excludedTypes: string[];
  respectConcealed: boolean;
}

export interface OcrSettings {
  enabled: boolean;
  language: string;
  autoRun: boolean;
}

export interface CorrectorSettings {
  enabled: boolean;
  /** UI locale code or 'auto' (follow interface language). */
  language: string;
  autoCorrect: boolean;
}

export const DEFAULT_SETTINGS: AppSettings = {
  general: {
    globalShortcut: 'Ctrl+Shift+V',
    maxHistoryItems: 500,
    launchAtStartup: false,
    showNotifications: true,
    theme: 'system',
    language: 'es'
  },
  exclusions: {
    excludedApps: ['1Password', 'Bitwarden', 'LastPass', 'KeePass'],
    excludedTypes: ['password', 'concealed'],
    respectConcealed: true
  },
  ocr: {
    enabled: true,
    language: 'eng',
    autoRun: false
  },
  corrector: {
    enabled: true,
    language: 'auto',
    autoCorrect: false
  }
};