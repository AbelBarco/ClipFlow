export interface AppSettings {
  general: GeneralSettings;
  exclusions: ExclusionSettings;
  ocr: OcrSettings;
  corrector: CorrectorSettings;
  security: SecuritySettings;
}

export interface GeneralSettings {
  globalShortcut: string;
  maxHistoryItems: number;
  launchAtStartup: boolean;
  showNotifications: boolean;
  theme: "light" | "dark" | "system";
  /** UI locale: en | es | fr | de | pt | it | zh | ja | ko | ru */
  language: string;
}

export interface ExclusionSettings {
  excludedApps: string[];
  excludedTypes: string[];
  respectConcealed: boolean;
  /** Secret-heuristic sensitivity: 'conservative' (default) or 'standard'. */
  heuristicLevel: string;
  /** OS notification when an item is dropped as a possible secret. */
  notifyOnExclude: boolean;
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
  /** User-taught words, always treated as known. */
  customWords: string[];
}

export interface SecuritySettings {
  encryptionEnabled: boolean;
}

export const DEFAULT_SETTINGS: AppSettings = {
  general: {
    globalShortcut: "Ctrl+Shift+V",
    maxHistoryItems: 500,
    launchAtStartup: false,
    showNotifications: true,
    theme: "system",
    language: "es",
  },
  exclusions: {
    excludedApps: ["1Password", "Bitwarden", "LastPass", "KeePass"],
    excludedTypes: ["password", "concealed"],
    respectConcealed: true,
    heuristicLevel: "conservative",
    notifyOnExclude: true,
  },
  ocr: {
    enabled: true,
    language: "eng",
    autoRun: false,
  },
  corrector: {
    enabled: true,
    language: "auto",
    autoCorrect: false,
    customWords: [],
  },
  security: {
    encryptionEnabled: false,
  },
};
