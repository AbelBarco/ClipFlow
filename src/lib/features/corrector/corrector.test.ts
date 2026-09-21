import { describe, expect, it } from 'vitest';
import {
  analyzeText,
  applyAllIssues,
  applyIssue,
  countChars,
  countWords
} from '$lib/features/corrector/corrector';
import { LANGUAGES, es, translations, type LocaleCode } from '$lib/features/i18n/translations';

describe('corrector engine', () => {
  it('detects double spaces', () => {
    const issues = analyzeText('hola  mundo', 'es');
    expect(issues.some((i) => i.type === 'doublespace' && i.suggestion === ' ')).toBe(true);
  });

  it('detects repeated words', () => {
    const issues = analyzeText('la la casa', 'es');
    expect(issues.some((i) => i.type === 'repeat' && i.suggestion === 'la')).toBe(true);
  });

  it('detects repeated punctuation', () => {
    const issues = analyzeText('genial!!', 'es');
    expect(issues.some((i) => i.type === 'punct' && i.suggestion === '!')).toBe(true);
  });

  it('detects missing space after punctuation', () => {
    const issues = analyzeText('hola,mundo', 'es');
    expect(issues.some((i) => i.type === 'spaceAfter' && i.suggestion === ', ')).toBe(true);
  });

  it('skips decimals and times in spaceAfter rule', () => {
    expect(analyzeText('vale 3,14 euros', 'es').filter((i) => i.type === 'spaceAfter')).toHaveLength(0);
    expect(analyzeText('a las 12:30', 'es').filter((i) => i.type === 'spaceAfter')).toHaveLength(0);
  });

  it('capitalizes sentence starts', () => {
    const issues = analyzeText('hola. mundo', 'es');
    expect(issues.some((i) => i.type === 'caps' && i.suggestion === 'M')).toBe(true);
  });

  it('suggests frequent typos per language', () => {
    expect(
      analyzeText('porfavor ven', 'es').some((i) => i.type === 'typo' && i.suggestion === 'por favor')
    ).toBe(true);
    expect(
      analyzeText('this is teh end', 'en').some((i) => i.type === 'typo' && i.suggestion === 'the')
    ).toBe(true);
    expect(
      analyzeText('je veux ca', 'fr').some((i) => i.type === 'typo' && i.suggestion === 'ça')
    ).toBe(true);
    expect(
      analyzeText('das ist standart', 'de').some(
        (i) => i.type === 'typo' && i.suggestion === 'standard'
      )
    ).toBe(true);
  });

  it('converts half-width punctuation to full-width in CJK text', () => {
    const issues = analyzeText('你好,世界', 'zh');
    expect(issues.some((i) => i.suggestion === '，')).toBe(true);
  });

  it('applyIssue and applyAllIssues fix the text', () => {
    const text = 'hola  mundo,amigo';
    const fixed = applyAllIssues(text, analyzeText(text, 'es'));
    expect(fixed).toBe('Hola mundo, amigo');
    const double = analyzeText('hola  mundo', 'es').find((i) => i.type === 'doublespace')!;
    expect(applyIssue('hola  mundo', double)).toBe('hola mundo');
  });

  it('counts words in CJK languages', () => {
    expect(countWords('你好世界', 'zh')).toBe(4);
    expect(countWords('hola mundo', 'es')).toBe(2);
    expect(countWords('  ', 'es')).toBe(0);
    expect(countChars('a😀')).toBe(2);
  });
});

describe('i18n dictionaries', () => {
  const locales = Object.keys(translations) as LocaleCode[];

  it('covers exactly the supported locales', () => {
    expect(locales.sort()).toEqual(
      ['de', 'en', 'es', 'fr', 'it', 'ja', 'ko', 'pt', 'ru', 'zh'].sort()
    );
  });

  it('every locale has exactly the same keys as Spanish', () => {
    const reference = Object.keys(es).sort();
    expect(reference.length).toBeGreaterThan(50);
    for (const locale of locales) {
      expect(Object.keys(translations[locale]).sort(), `locale ${locale}`).toEqual(reference);
    }
  });

  it('no empty translations', () => {
    for (const locale of locales) {
      for (const [key, value] of Object.entries(translations[locale])) {
        expect(value.trim().length, `${locale}.${key}`).toBeGreaterThan(0);
      }
    }
  });

  it('every language has OCR metadata', () => {
    expect(LANGUAGES).toHaveLength(10);
    for (const lang of LANGUAGES) {
      expect(lang.ocr.length).toBeGreaterThan(0);
      expect(lang.html.length).toBeGreaterThan(0);
      expect(lang.name.length).toBeGreaterThan(0);
    }
  });
});
