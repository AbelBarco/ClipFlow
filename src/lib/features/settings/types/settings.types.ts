export interface AppSettings {
  general: GeneralSettings;
  exclusions: ExclusionSettings;
  ocr: OcrSettings;
}

export interface GeneralSettings {
  globalShortcut: string;
  maxHistoryItems: number;
  launchAtStartup: boolean;
  showNotifications: boolean;
  theme: 'light' | 'dark' | 'system';
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

export const DEFAULT_SETTINGS: AppSettings = {
  general: {
    globalShortcut: 'Ctrl+Shift+V',
    maxHistoryItems: 500,
    launchAtStartup: false,
    showNotifications: true,
    theme: 'system'
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
  }
};