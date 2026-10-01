import { describe, expect, it } from "vitest";
import {
  analyzeText,
  applyAllIssues,
  applyIssue,
  applyIssuesWithCaret,
  autoCorrectText,
  countChars,
  countWords,
  DICT_LANGS,
  mergeIssues,
  TYPO_TABLES,
} from "$lib/features/corrector/corrector";
import {
  LANGUAGES,
  es,
  translations,
  type LocaleCode,
} from "$lib/features/i18n/translations";

describe("corrector engine", () => {
  it("detects double spaces", () => {
    const issues = analyzeText("hola  mundo", "es");
    expect(
      issues.some((i) => i.type === "doublespace" && i.suggestion === " "),
    ).toBe(true);
  });

  it("detects repeated words", () => {
    const issues = analyzeText("la la casa", "es");
    expect(
      issues.some((i) => i.type === "repeat" && i.suggestion === "la"),
    ).toBe(true);
  });

  it("detects repeated punctuation", () => {
    const issues = analyzeText("genial!!", "es");
    expect(issues.some((i) => i.type === "punct" && i.suggestion === "!")).toBe(
      true,
    );
  });

  it("detects missing space after punctuation", () => {
    const issues = analyzeText("hola,mundo", "es");
    expect(
      issues.some((i) => i.type === "spaceAfter" && i.suggestion === ", "),
    ).toBe(true);
  });

  it("skips decimals and times in spaceAfter rule", () => {
    expect(
      analyzeText("vale 3,14 euros", "es").filter(
        (i) => i.type === "spaceAfter",
      ),
    ).toHaveLength(0);
    expect(
      analyzeText("a las 12:30", "es").filter((i) => i.type === "spaceAfter"),
    ).toHaveLength(0);
  });

  it("capitalizes sentence starts", () => {
    const issues = analyzeText("hola. mundo", "es");
    expect(issues.some((i) => i.type === "caps" && i.suggestion === "M")).toBe(
      true,
    );
  });

  it("suggests frequent typos per language", () => {
    expect(
      analyzeText("porfavor ven", "es").some(
        (i) => i.type === "typo" && i.suggestion === "por favor",
      ),
    ).toBe(true);
    expect(
      analyzeText("this is teh end", "en").some(
        (i) => i.type === "typo" && i.suggestion === "the",
      ),
    ).toBe(true);
    expect(
      analyzeText("je veux ca", "fr").some(
        (i) => i.type === "typo" && i.suggestion === "ça",
      ),
    ).toBe(true);
    expect(
      analyzeText("das ist standart", "de").some(
        (i) => i.type === "typo" && i.suggestion === "standard",
      ),
    ).toBe(true);
  });

  it("converts half-width punctuation to full-width in CJK text", () => {
    const issues = analyzeText("你好,世界", "zh");
    expect(issues.some((i) => i.suggestion === "，")).toBe(true);
  });

  it("applyIssue and applyAllIssues fix the text", () => {
    const text = "hola  mundo,amigo";
    const fixed = applyAllIssues(text, analyzeText(text, "es"));
    expect(fixed).toBe("Hola mundo, amigo");
    const double = analyzeText("hola  mundo", "es").find(
      (i) => i.type === "doublespace",
    )!;
    expect(applyIssue("hola  mundo", double)).toBe("hola mundo");
  });

  it("never destroys text on overlapping issues", () => {
    // repeat [0..7] contiene dos typos: antes colapsaba a "teh".
    const fixed = applyAllIssues("teh teh", analyzeText("teh teh", "en"));
    expect(fixed).toBe("the the");
    // Y la repetición restante se puede aplicar después, a mano.
    const rep = analyzeText(fixed, "en").find((i) => i.type === "repeat")!;
    expect(applyIssue(fixed, rep)).toBe("the");
  });

  it("keeps the caret stable through replacements", () => {
    // "porfavor| ven" con cursor al final -> "por favor| " desplazado +1.
    const r1 = applyIssuesWithCaret(
      "porfavor ven",
      analyzeText("porfavor ven", "es"),
      11,
    );
    expect(r1.text).toBe("por favor ven");
    expect(r1.caret).toBe(12);
    // Cursor dentro de lo reemplazado -> queda al final del reemplazo.
    const r2 = applyIssuesWithCaret(
      "porfavor ven",
      analyzeText("porfavor ven", "es"),
      4,
    );
    expect(r2.text).toBe("por favor ven");
    expect(r2.caret).toBe("por favor".length);
    // Cursor delante de todo cambio -> no se mueve.
    const r3 = applyIssuesWithCaret(
      "hola  mundo",
      analyzeText("hola  mundo", "es"),
      0,
    );
    expect(r3.text).toBe("Hola mundo");
    expect(r3.caret).toBe(0);
  });

  it("detects typos inside CJK/Korean words (no word boundaries)", () => {
    expect(
      analyzeText("安되요", "ko").some(
        (i) => i.type === "typo" && i.suggestion === "돼요",
      ),
    ).toBe(true);
    expect(
      analyzeText("明天会下雨吗。在见", "zh").some(
        (i) => i.type === "typo" && i.suggestion === "再见",
      ),
    ).toBe(true);
    expect(
      analyzeText("明日もこんにちわ", "ja").some(
        (i) => i.type === "typo" && i.suggestion === "こんにちは",
      ),
    ).toBe(true);
  });

  it("auto-correct only applies safe fixes", () => {
    // Typo al terminar la palabra (espacio final) -> se corrige.
    const r1 = autoCorrectText("esto es una prueva ", "es", 19);
    expect(r1.text).toBe("esto es una prueba ");
    expect(r1.changed).toBe(true);
    // Palabra en curso -> no se toca.
    const r2 = autoCorrectText("esto es una prueva", "es", 18);
    expect(r2.text).toBe("esto es una prueva");
    expect(r2.changed).toBe(false);
    // Ni repeat ni caps entran en automático.
    const r3 = autoCorrectText("hola hola ", "es", 10);
    expect(r3.text).toBe("hola hola ");
    const r4 = autoCorrectText("hola. mundo ", "es", 12);
    expect(r4.text).toBe("hola. mundo ");
    // Pero espacios dobles y puntuación sí.
    const r5 = autoCorrectText("hola  mundo ", "es", 12);
    expect(r5.text).toBe("hola mundo ");
  });

  it("auto-correct respects manual-only entries", () => {
    // "pero" (it) y estilo kanji/kana (ja) solo se sugieren, no se aplican.
    const r1 = autoCorrectText("vorrei un pero ", "it", 14);
    expect(r1.text).toBe("vorrei un pero ");
    const r2 = autoCorrectText("事が出来る。", "ja", 6);
    expect(r2.text).toBe("事が出来る。");
    // …pero siguen apareciendo como sugerencias manuales.
    expect(
      analyzeText("vorrei un pero", "it").some(
        (i) => i.type === "typo" && i.suggestion === "però",
      ),
    ).toBe(true);
  });

  it("mergeIssues prefers rules and keeps dict rows learnable", () => {
    expect(DICT_LANGS).toEqual(["es", "en", "fr", "de", "pt", "it", "ru"]);
    const rule = analyzeText("escreva porfavor aqui", "pt");
    const merged = mergeIssues(rule, [
      { index: 7, length: 8, word: "porfavor", suggestions: ["por favor"] },
      { index: 16, length: 4, word: "aqui", suggestions: [] },
    ]);
    // El typo curado manda sobre el dict solapado; el otro dict sobrevive
    // sin sugerencia (sirve para "aprender") y no borra al aplicar todo.
    expect(merged.filter((i) => i.type === "dict")).toHaveLength(1);
    expect(merged.some((i) => i.type === "typo")).toBe(true);
    const dictRow = merged.find((i) => i.type === "dict")!;
    expect(dictRow.original).toBe("aqui");
    expect(dictRow.suggestion).toBe("");
    expect(applyAllIssues("escreva porfavor aqui", merged)).toBe(
      "Escreva por favor aqui",
    );
  });

  it("typo tables have no identity, empty or duplicate entries", () => {
    for (const [lang, pairs] of Object.entries(TYPO_TABLES)) {
      const wrongs = new Set<string>();
      expect(pairs.length, `${lang} table`).toBeGreaterThan(0);
      for (const [wrong, right] of pairs) {
        expect(wrong.trim().length, `${lang}:${wrong} empty`).toBeGreaterThan(
          0,
        );
        expect(right.trim().length, `${lang}:${wrong} empty`).toBeGreaterThan(
          0,
        );
        expect(wrong, `${lang}:${wrong} identical`).not.toBe(right);
        expect(
          wrongs.has(wrong.toLowerCase()),
          `${lang}:${wrong} duplicated`,
        ).toBe(false);
        wrongs.add(wrong.toLowerCase());
      }
    }
  });

  it("counts words in CJK languages", () => {
    expect(countWords("你好世界", "zh")).toBe(4);
    expect(countWords("hola mundo", "es")).toBe(2);
    expect(countWords("  ", "es")).toBe(0);
    expect(countChars("a😀")).toBe(2);
  });
});

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
