import { describe, expect, it } from "vitest";
import {
  LANGUAGES,
  es,
  translations,
  type LocaleCode,
} from "$lib/features/i18n/translations";

describe("i18n dictionaries", () => {
  const locales = Object.keys(translations) as LocaleCode[];

  it("covers exactly the supported locales", () => {
    expect(locales.sort()).toEqual(
      ["de", "en", "es", "fr", "it", "ja", "ko", "pt", "ru", "zh"].sort(),
    );
  });

  it("every locale has exactly the same keys as Spanish", () => {
    const reference = Object.keys(es).sort();
    expect(reference.length).toBeGreaterThan(50);
    for (const locale of locales) {
      expect(
        Object.keys(translations[locale]).sort(),
        `locale ${locale}`,
      ).toEqual(reference);
    }
  });

  it("no empty translations", () => {
    for (const locale of locales) {
      for (const [key, value] of Object.entries(translations[locale])) {
        expect(value.trim().length, `${locale}.${key}`).toBeGreaterThan(0);
      }
    }
  });

  it("every language has OCR metadata", () => {
    expect(LANGUAGES).toHaveLength(10);
    for (const lang of LANGUAGES) {
      expect(lang.ocr.length).toBeGreaterThan(0);
      expect(lang.html.length).toBeGreaterThan(0);
      expect(lang.name.length).toBeGreaterThan(0);
    }
  });
});
