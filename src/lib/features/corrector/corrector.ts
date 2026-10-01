/**
 * Corrector nativo de ClipFlow — 100 % offline, sin APIs externas.
 *
 * Funciona en los 10 idiomas de la app combinando:
 *  - reglas universales (espacios dobles, repeticiones, puntuación,
 *    mayúsculas de frase),
 *  - tablas de erratas frecuentes por idioma (wrong -> right),
 *  - reglas CJK (puntuación de ancho medio -> ancho completo en zh/ja).
 *
 * El subrayado ondulado adicional lo pone el corrector del propio sistema
 * operativo mediante `spellcheck` + `lang` en el editor.
 */

import type { LocaleCode } from "$lib/features/i18n/translations";

export type CorrectorIssueType =
  "doublespace" | "repeat" | "punct" | "spaceAfter" | "caps" | "typo" | "dict";

/**
 * Idiomas con diccionario FST empaquetado (backend). zh/ja/ko usan solo
 * reglas: sin espacios y con morfología rica, las listas son la herramienta
 * equivocada ahí (igual que en Word).
 */
export const DICT_LANGS: ReadonlyArray<LocaleCode> = [
  "es",
  "en",
  "fr",
  "de",
  "pt",
  "it",
  "ru",
];

/**
 * Tipos que la autocorrección aplica sola al escribir: inequívocos y sin
 * cambiar el significado. `repeat` y `caps` ("had had", nombres propios,
 * código) solo se sugieren para aplicar a mano.
 */
export const AUTO_SAFE_TYPES: ReadonlySet<CorrectorIssueType> = new Set([
  "typo",
  "doublespace",
  "spaceAfter",
  "punct",
]);

/**
 * Idiomas sin espacios entre palabras: las tablas se buscan por subcadena
 * porque los boundaries de letra (`(?<![\p{L}])`) bloquearían casi todo
 * ("安되요" contiene "되요" precedida de letra, "事が出来る" igual, etc.).
 */
const SPACED_LANGS: ReadonlySet<LocaleCode> = new Set([
  "es",
  "en",
  "fr",
  "de",
  "pt",
  "it",
  "ru",
]);

export interface CorrectorIssue {
  id: number;
  type: CorrectorIssueType;
  /** Offset en UTF-16 dentro del texto original. */
  index: number;
  length: number;
  original: string;
  suggestion: string;
}

let nextId = 1;

/** Erratas frecuentes por idioma: [forma incorrecta, corrección]. */
export const TYPO_TABLES: Record<LocaleCode, Array<[string, string]>> = {
  es: [
    ["porfavor", "por favor"],
    ["osea", "o sea"],
    ["aver", "a ver"],
    ["hechar", "echar"],
    ["herror", "error"],
    ["haver", "haber"],
    ["haser", "hacer"],
    ["yegar", "llegar"],
    ["ablar", "hablar"],
    ["exajerado", "exagerado"],
    ["setiembre", "septiembre"],
    ["saver", "saber"],
    ["revez", "revés"],
    ["atravez", "a través"],
    ["conciente", "consciente"],
    ["dijistes", "dijiste"],
    ["hicistes", "hiciste"],
    ["fuistes", "fuiste"],
    ["vinistes", "viniste"],
    ["dijieron", "dijeron"],
    ["haiga", "haya"],
    ["deacuerdo", "de acuerdo"],
    ["postdata", "posdata"],
    ["fué", "fue"],
    ["dió", "dio"],
    ["vió", "vio"],
    ["exámen", "examen"],
    ["imágen", "imagen"],
    ["jóven", "joven"],
    ["prueva", "prueba"],
  ],
  en: [
    ["teh", "the"],
    ["recieve", "receive"],
    ["adress", "address"],
    ["seperate", "separate"],
    ["definately", "definitely"],
    ["occured", "occurred"],
    ["neccessary", "necessary"],
    ["accomodate", "accommodate"],
    ["embarass", "embarrass"],
    ["suprise", "surprise"],
    ["tomorow", "tomorrow"],
    ["writting", "writing"],
    ["begining", "beginning"],
    ["goverment", "government"],
    ["untill", "until"],
    ["wich", "which"],
    ["beleive", "believe"],
    ["freind", "friend"],
    ["happend", "happened"],
    ["truely", "truly"],
    ["arguement", "argument"],
    ["comming", "coming"],
    ["runing", "running"],
    ["geting", "getting"],
    ["useing", "using"],
    ["writen", "written"],
    ["choosen", "chosen"],
    ["thier", "their"],
    ["peice", "piece"],
    ["libary", "library"],
    ["febuary", "february"],
    ["wensday", "wednesday"],
  ],
  fr: [
    ["etre", "être"],
    ["ca", "ça"],
    ["tres", "très"],
    ["meme", "même"],
    ["connaitre", "connaître"],
    ["paraitre", "paraître"],
    ["fenetre", "fenêtre"],
    ["interet", "intérêt"],
    ["foret", "forêt"],
    ["arret", "arrêt"],
    ["tete", "tête"],
    ["hotel", "hôtel"],
    ["grace", "grâce"],
    ["cout", "coût"],
    ["maitre", "maître"],
    ["ete", "été"],
    ["etait", "était"],
    ["etaient", "étaient"],
    ["apres", "après"],
    ["voila", "voilà"],
    ["deja", "déjà"],
    ["premiere", "première"],
    ["pere", "père"],
    ["mere", "mère"],
    ["frere", "frère"],
    ["fete", "fête"],
    ["bete", "bête"],
    ["gouter", "goûter"],
  ],
  de: [
    ["standart", "standard"],
    ["rythmus", "rhythmus"],
    ["maschiene", "maschine"],
    ["interesant", "interessant"],
    ["orginal", "original"],
    ["seperat", "separat"],
    ["paralell", "parallel"],
    ["agressiv", "aggressiv"],
    ["terasse", "terrasse"],
    ["nähmlich", "nämlich"],
    ["endgülitg", "endgültig"],
    ["tolleranz", "toleranz"],
    ["potential", "potenzial"],
    ["kokusnuss", "kokosnuss"],
    ["zuchini", "zucchini"],
    ["spagetti", "spaghetti"],
    ["capuchino", "cappuccino"],
    ["expresso", "espresso"],
  ],
  pt: [
    ["porfavor", "por favor"],
    ["concerteza", "com certeza"],
    ["excessão", "exceção"],
    ["exceçao", "exceção"],
    ["previlegio", "privilégio"],
    ["enchergar", "enxergar"],
    ["muinto", "muito"],
    ["voce", "você"],
    ["tambem", "também"],
    ["nao", "não"],
    ["opçao", "opção"],
    ["idéia", "ideia"],
    ["vôo", "voo"],
    ["vôos", "voos"],
    ["derepente", "de repente"],
    ["sussesso", "sucesso"],
    ["excurçao", "excursão"],
    ["pretençao", "pretensão"],
    ["estória", "história"],
  ],
  it: [
    ["pero", "però"],
    ["cioe", "cioè"],
    ["perche", "perché"],
    ["poiche", "poiché"],
    ["gia", "già"],
    ["puo", "può"],
    ["citta", "città"],
    ["sopratutto", "soprattutto"],
    ["daccordo", "d'accordo"],
    ["qualè", "qual è"],
    ["percui", "per cui"],
    ["affianco", "a fianco"],
    ["pultroppo", "purtroppo"],
    ["atimo", "attimo"],
    ["arivato", "arrivato"],
    ["raporti", "rapporti"],
    ["tapeto", "tappeto"],
    ["sestesso", "se stesso"],
  ],
  zh: [
    ["帐号", "账号"],
    ["做用", "作用"],
    ["部份", "部分"],
    ["另人", "令人"],
    ["以身作责", "以身作则"],
    ["在见", "再见"],
    ["做为", "作为"],
    ["即然", "既然"],
    ["按排", "安排"],
    ["幅射", "辐射"],
  ],
  ja: [
    ["いう事", "いうこと"],
    ["出来る", "できる"],
    ["出来ます", "できます"],
    ["事が", "ことが"],
    ["無い", "ない"],
    ["有る", "ある"],
    ["下さい", "ください"],
    ["頂く", "いただく"],
    ["こんにちわ", "こんにちは"],
    ["こんばんわ", "こんばんは"],
    ["すいません", "すみません"],
  ],
  ko: [
    ["되요", "돼요"],
    ["됬", "됐"],
    ["됌", "됨"],
    ["않돼", "안돼"],
    ["할께", "할게"],
    ["할꺼야", "할 거야"],
    ["왠만하면", "웬만하면"],
    ["금새", "금세"],
    ["어떻해", "어떡해"],
    ["이쁘다", "예쁘다"],
    ["왠지", "웬지"],
    ["가르키다", "가리키다"],
    ["붇다", "붓다"],
    ["맞추다", "맞히다"],
  ],
  ru: [
    ["зделать", "сделать"],
    ["вообщем", "в общем"],
    ["вобщем", "в общем"],
    ["как-будто", "как будто"],
    ["извените", "извините"],
    ["симпотичный", "симпатичный"],
    ["через-чур", "чересчур"],
    ["впринципе", "в принципе"],
    ["вобщем-то", "в общем-то"],
    ["до-свидания", "до свидания"],
    ["ето", "это"],
    ["што", "что"],
    ["конешно", "конечно"],
    ["девченка", "девчонка"],
    ["подскользнуться", "поскользнуться"],
  ],
};

/** Puntuación latina -> ancho completo (para texto CJK). */
const CJK_PUNCT: Record<string, string> = {
  ",": "，",
  ";": "；",
  ":": "：",
  "!": "！",
  "?": "？",
};

const CJK_RE = /[\u4e00-\u9fff\u3040-\u30ff\u31f0-\u31ff\uff00-\uffef]/;
const USES_CASE: ReadonlySet<LocaleCode> = new Set([
  "es",
  "en",
  "fr",
  "de",
  "pt",
  "it",
  "ru",
]);

/**
 * Erratas que solo se sugieren a mano, nunca en automático: preferencias de
 * estilo (kanji vs kana en japonés) o formas ambiguas (`pero` = peral en
 * italiano). Clave `idioma:forma-en-minúsculas`.
 */
const MANUAL_ONLY_TYPOS: ReadonlySet<string> = new Set([
  "it:pero",
  "ja:いう事",
  "ja:出来る",
  "ja:出来ます",
  "ja:事が",
  "ja:無い",
  "ja:有る",
  "ja:下さい",
  "ja:頂く",
]);

function escapeRegExp(s: string): string {
  return s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function matchCase(sample: string, word: string): string {
  if (!sample) return word;
  if (
    sample[0] === sample[0].toUpperCase() &&
    sample.slice(1) === sample.slice(1).toLowerCase()
  ) {
    return word.charAt(0).toUpperCase() + word.slice(1);
  }
  if (sample === sample.toUpperCase() && sample !== sample.toLowerCase()) {
    return word.toUpperCase();
  }
  return word;
}

function push(
  issues: CorrectorIssue[],
  type: CorrectorIssueType,
  index: number,
  original: string,
  suggestion: string,
): void {
  if (!original || original === suggestion) return;
  issues.push({
    id: nextId++,
    type,
    index,
    length: original.length,
    original,
    suggestion,
  });
}

/**
 * Analiza `text` y devuelve la lista de incidencias ordenada por posición.
 * Es pura y síncrona: no hay red ni E/S, apta para ejecutarse en cada
 * pulsación (`autoCorrect` la reutiliza para corregir al escribir).
 */
export function analyzeText(text: string, lang: LocaleCode): CorrectorIssue[] {
  const issues: CorrectorIssue[] = [];
  if (!text) return issues;

  // 1. Espacios dobles (o más). Se ignoran los de inicio de línea (código).
  for (const m of text.matchAll(/[^ \t\n][ \t]{2,}(?=[^ \t\n])/g)) {
    const matched = m[0];
    const start = (m.index ?? 0) + 1;
    push(issues, "doublespace", start, matched.slice(1), " ");
  }

  // 2. Palabras repetidas: "la la", "que que".
  for (const m of text.matchAll(/\b(\p{L}+)(\s+)(\1)\b/giu)) {
    const full = m[0];
    const word = m[1];
    // "had had" / "que que" a veces es válido; aun así se sugiere revisar.
    push(issues, "repeat", m.index ?? 0, full, word);
  }

  // 3. Puntuación repetida: "!!", "??", "...", ",,".
  for (const m of text.matchAll(/([!?])\1+|(\.\.\.+)|([,;:])\3+/g)) {
    const full = m[0];
    let suggestion = full[0];
    if (/^\.+$/.test(full))
      suggestion = lang === "zh" || lang === "ja" ? "……" : "…";
    push(issues, "punct", m.index ?? 0, full, suggestion);
  }

  // 4. Falta espacio tras puntuación: "hola,mundo" -> "hola, mundo".
  //    Se excluyen decimales/horas (3,14 / 12:30) y URLs.
  for (const m of text.matchAll(/([,;:!?])(?=\S)/g)) {
    const idx = m.index ?? 0;
    const before = text[idx - 1] ?? "";
    const after = text[idx + 1] ?? "";
    if (/\d/.test(before) && /\d/.test(after)) continue;
    if (before === ":" && /[/:]/.test(after)) continue; // http://, 12:30:45
    if (/[/\w]/.test(before) && after === "/") continue;
    // Emoticonos habituales :)
    if ((m[1] === ":" || m[1] === ";") && /[)D(P]/.test(after)) continue;
    const window = text.slice(Math.max(0, idx - 6), idx);
    if (/https?$/.test(window) || /www\.?$/.test(window)) continue;
    push(issues, "spaceAfter", idx, m[1], `${m[1]} `);
  }

  // 5. Mayúscula inicial de frase (solo idiomas con mayúsculas).
  if (USES_CASE.has(lang)) {
    const startMatch = text.match(/^(\s*)(\p{Ll})/u);
    if (startMatch && startMatch[2]) {
      const idx = startMatch[1]?.length ?? 0;
      push(issues, "caps", idx, startMatch[2], startMatch[2].toUpperCase());
    }
    for (const m of text.matchAll(/([.!?…]\s+)([\p{Ll}])/gu)) {
      const prefixLen = m[1].length;
      const idx = (m.index ?? 0) + prefixLen;
      push(issues, "caps", idx, m[2], m[2].toUpperCase());
    }
    // Pronombre inglés "i" aislado -> "I".
    if (lang === "en") {
      for (const m of text.matchAll(/(?<![\p{L}'])i(?![\p{L}'])/gu)) {
        push(issues, "caps", m.index ?? 0, "i", "I");
      }
    }
  }

  // 6. Puntuación latina dentro de texto CJK -> ancho completo.
  if (lang === "zh" || lang === "ja") {
    for (const m of text.matchAll(/([,;:!?])/g)) {
      const idx = m.index ?? 0;
      const before = text[idx - 1] ?? "";
      const after = text[idx + 1] ?? "";
      if (CJK_RE.test(before) || CJK_RE.test(after)) {
        const full = CJK_PUNCT[m[1]];
        if (full) push(issues, "punct", idx, m[1], full);
      }
    }
  }

  // 7. Erratas frecuentes del idioma activo.
  const table = TYPO_TABLES[lang] ?? [];
  const useBoundaries = SPACED_LANGS.has(lang);
  for (const [wrong, right] of table) {
    if (!wrong || wrong === right) continue;
    let re: RegExp;
    try {
      re = useBoundaries
        ? new RegExp(`(?<![\\p{L}])${escapeRegExp(wrong)}(?![\\p{L}])`, "giu")
        : new RegExp(escapeRegExp(wrong), "gu");
    } catch {
      continue;
    }
    for (const m of text.matchAll(re)) {
      const found = m[0];
      push(issues, "typo", m.index ?? 0, found, matchCase(found, right));
    }
  }

  issues.sort((a, b) => a.index - b.index || a.length - b.length);
  return issues;
}

/** Aplica una incidencia a un texto y devuelve el texto resultante. */
export function applyIssue(text: string, issue: CorrectorIssue): string {
  // Sin sugerencia no hay nada que aplicar (nunca borrar por accidente:
  // las incidencias `dict` sin candidatas solo sirven para "aprender").
  if (!issue.suggestion) return text;
  return (
    text.slice(0, issue.index) +
    issue.suggestion +
    text.slice(issue.index + issue.length)
  );
}

/** Aplica todas las incidencias sin solapamientos (de atrás hacia delante para no mover offsets). */
export function applyAllIssues(text: string, issues: CorrectorIssue[]): string {
  return applyIssuesWithCaret(text, issues, text.length).text;
}

export interface CaretApplyResult {
  text: string;
  /** Nueva posición del cursor (offsets UTF-16, como selectionStart). */
  caret: number;
  applied: number;
}

/**
 * Filtra incidencias solapadas (p. ej. un `repeat` que contiene dos `typo`):
 * se aplican de atrás adelante y se descarta la que pise una ya aplicada.
 * Sin esto, "teh teh" colapsaba a "teh" (pérdida de datos).
 */
function nonOverlapping(issues: CorrectorIssue[]): CorrectorIssue[] {
  // De atrás adelante; a igual índice gana el match más largo (p. ej. el
  // typo "teh"[0..3] prevalece sobre el caps "t"[0..1]). Sin sugerencia no
  // se puede aplicar: se descarta del lote (pero sigue listada para
  // "aprender" la palabra).
  const ordered = [...issues].sort(
    (a, b) => b.index - a.index || b.length - a.length,
  );
  const kept: CorrectorIssue[] = [];
  let minStart = Infinity;
  for (const issue of ordered) {
    if (!issue.suggestion) continue;
    if (issue.index + issue.length > minStart) continue; // solapa: descartar
    kept.push(issue);
    minStart = issue.index;
  }
  return kept;
}

/**
 * Como `applyAllIssues` pero recalcula la posición del cursor a través de
 * cada reemplazo: si el cursor va detrás, se desplaza por el delta; si cae
 * dentro de lo reemplazado, queda al final del reemplazo.
 */
export function applyIssuesWithCaret(
  text: string,
  issues: CorrectorIssue[],
  caret: number,
): CaretApplyResult {
  let out = text;
  let pos = Math.max(0, Math.min(caret, text.length));
  let applied = 0;
  for (const issue of nonOverlapping(issues)) {
    out = applyIssue(out, issue);
    const end = issue.index + issue.length;
    if (pos > end) {
      pos += issue.suggestion.length - issue.length;
    } else if (pos > issue.index) {
      pos = issue.index + issue.suggestion.length;
    }
    applied++;
  }
  return { text: out, caret: Math.max(0, pos), applied };
}

export interface AutoCorrectResult extends CaretApplyResult {
  /** true si se corrigió algo. */
  changed: boolean;
}

/** Coincidencia del diccionario backend (palabra desconocida + sugerencias). */
export interface DictMatch {
  index: number;
  length: number;
  word: string;
  suggestions: string[];
}

/**
 * Une incidencias de reglas con las del diccionario. Ante solape manda la
 * regla curada (p. ej. el typo "porfavor" prevalece sobre el dict que también
 * lo marcaría). Las de diccionario sin sugerencias se conservan igual: sirven
 * para "aprender" la palabra.
 */
export function mergeIssues(
  rule: CorrectorIssue[],
  dict: DictMatch[],
): CorrectorIssue[] {
  const merged: CorrectorIssue[] = [...rule];
  for (const d of dict) {
    const overlaps = merged.some(
      (r) => d.index < r.index + r.length && d.index + d.length > r.index,
    );
    if (overlaps) continue;
    merged.push({
      id: nextId++,
      type: "dict",
      index: d.index,
      length: d.length,
      original: d.word,
      suggestion: d.suggestions[0] ?? "",
    });
  }
  merged.sort((a, b) => a.index - b.index || a.length - b.length);
  return merged;
}

/** Palabra en curso al final del texto (letras/dígitos/apóstrofes/guiones). */
const TRAILING_WORD_RE = /[\p{L}\p{N}'’_-]+$/u;

/**
 * Autocorrección real al escribir: aplica solo tipos seguros, nunca toca la
 * palabra que el usuario aún está escribiendo (hasta que la termina con un
 * espacio, salto o signo) y respeta las entradas solo-manuales.
 */
export function autoCorrectText(
  text: string,
  lang: LocaleCode,
  caret: number,
): AutoCorrectResult {
  if (!text) return { text, caret, applied: 0, changed: false };
  const trailing = text.match(TRAILING_WORD_RE);
  const protectedStart =
    trailing && trailing.index !== undefined ? trailing.index : -1;

  const candidates = analyzeText(text, lang).filter((issue) => {
    if (!AUTO_SAFE_TYPES.has(issue.type)) return false;
    if (
      issue.type === "typo" &&
      MANUAL_ONLY_TYPOS.has(`${lang}:${issue.original.toLowerCase()}`)
    ) {
      return false;
    }
    // No tocar la palabra en curso (ni nada que la solape).
    if (protectedStart >= 0 && issue.index + issue.length > protectedStart) {
      return false;
    }
    return true;
  });

  const result = applyIssuesWithCaret(text, candidates, caret);
  return { ...result, changed: result.text !== text };
}

/** Cuenta palabras de forma razonable en los 10 idiomas. */
export function countWords(text: string, lang: LocaleCode): number {
  const trimmed = text.trim();
  if (!trimmed) return 0;
  if (lang === "zh" || lang === "ja") {
    const cjk =
      trimmed.match(/[\u4e00-\u9fff\u3040-\u30ff\u31f0-\u31ff]/g)?.length ?? 0;
    const latin =
      trimmed.match(/[A-Za-zÀ-ÿ0-9]+(?:['’][A-Za-zÀ-ÿ0-9]+)*/g)?.length ?? 0;
    return cjk + latin;
  }
  return trimmed.match(/\p{L}[\p{L}\p{N}'’-]*/gu)?.length ?? 0;
}

/** Longitud en caracteres (puntos de código, no UTF-16). */
export function countChars(text: string): number {
  return [...text].length;
}
