import { invoke } from "@tauri-apps/api/core";
import type {
  AppSettings,
  GeneralSettings,
  ExclusionSettings,
  OcrSettings,
  CorrectorSettings,
  SecuritySettings,
} from "../types/settings.types";
import { DEFAULT_SETTINGS } from "../types/settings.types";

class SettingsStore {
  settings: AppSettings = $state({ ...DEFAULT_SETTINGS });
  isLoading: boolean = $state(false);
  hasChanges: boolean = $state(false);

  async load(): Promise<void> {
    this.isLoading = true;
    try {
      const loaded = await invoke<AppSettings>("settings_get");
      // Merge section by section so configs saved by older versions
      // (without `language` or `corrector`) keep working.
      this.settings = {
        ...DEFAULT_SETTINGS,
        ...loaded,
        general: { ...DEFAULT_SETTINGS.general, ...(loaded.general ?? {}) },
        exclusions: {
          ...DEFAULT_SETTINGS.exclusions,
          ...(loaded.exclusions ?? {}),
        },
        ocr: { ...DEFAULT_SETTINGS.ocr, ...(loaded.ocr ?? {}) },
        corrector: {
          ...DEFAULT_SETTINGS.corrector,
          ...(loaded.corrector ?? {}),
        },
        security: { ...DEFAULT_SETTINGS.security, ...(loaded.security ?? {}) },
      };
      this.hasChanges = false;
    } catch (error) {
      console.error("Failed to load settings:", error);
      this.settings = { ...DEFAULT_SETTINGS };
    } finally {
      this.isLoading = false;
    }
  }

  async save(): Promise<void> {
    try {
      await invoke("settings_set", { config: this.settings });
      this.hasChanges = false;
    } catch (error) {
      console.error("Failed to save settings:", error);
      throw error;
    }
  }

  async resetToDefaults(): Promise<void> {
    try {
      const defaults = await invoke<AppSettings>("settings_reset");
      this.settings = defaults;
    } catch {
      this.settings = { ...DEFAULT_SETTINGS };
    }
    this.hasChanges = true;
  }

  reset(): void {
    this.settings = { ...DEFAULT_SETTINGS };
    this.hasChanges = true;
  }

  updateGeneral(general: Partial<GeneralSettings>): void {
    this.settings.general = { ...this.settings.general, ...general };
    this.hasChanges = true;
  }

  updateExclusions(exclusions: Partial<ExclusionSettings>): void {
    this.settings.exclusions = { ...this.settings.exclusions, ...exclusions };
    this.hasChanges = true;
  }

  updateOcr(ocr: Partial<OcrSettings>): void {
    this.settings.ocr = { ...this.settings.ocr, ...ocr };
    this.hasChanges = true;
  }

  updateCorrector(corrector: Partial<CorrectorSettings>): void {
    this.settings.corrector = { ...this.settings.corrector, ...corrector };
    this.hasChanges = true;
  }

  updateSecurity(security: Partial<SecuritySettings>): void {
    this.settings.security = { ...this.settings.security, ...security };
    this.hasChanges = true;
  }

  addExcludedApp(app: string): void {
    if (!this.settings.exclusions.excludedApps.includes(app)) {
      this.settings.exclusions.excludedApps = [
        ...this.settings.exclusions.excludedApps,
        app,
      ];
      this.hasChanges = true;
    }
  }

  removeExcludedApp(app: string): void {
    this.settings.exclusions.excludedApps =
      this.settings.exclusions.excludedApps.filter((a) => a !== app);
    this.hasChanges = true;
  }
}

export const settingsStore = new SettingsStore();
