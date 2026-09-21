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

import type { LocaleCode } from '$lib/features/i18n/translations';

export type CorrectorIssueType =
  | 'doublespace'
  | 'repeat'
  | 'punct'
  | 'spaceAfter'
  | 'caps'
  | 'typo';

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
const TYPO_TABLES: Record<LocaleCode, Array<[string, string]>> = {
  es: [
    ['porfavor', 'por favor'],
    ['osea', 'o sea'],
    ['aver', 'a ver'],
    ['hechar', 'echar'],
    ['herror', 'error'],
    ['haver', 'haber'],
    ['haser', 'hacer'],
    ['yegar', 'llegar'],
    ['ablar', 'hablar'],
    ['exajerado', 'exagerado'],
    ['setiembre', 'septiembre'],
    ['saver', 'saber'],
    ['revez', 'revés'],
    ['atravez', 'a través'],
    ['conciente', 'consciente']
  ],
  en: [
    ['teh', 'the'],
    ['recieve', 'receive'],
    ['adress', 'address'],
    ['seperate', 'separate'],
    ['definately', 'definitely'],
    ['occured', 'occurred'],
    ['neccessary', 'necessary'],
    ['accomodate', 'accommodate'],
    ['embarass', 'embarrass'],
    ['suprise', 'surprise'],
    ['tomorow', 'tomorrow'],
    ['writting', 'writing'],
    ['begining', 'beginning'],
    ['goverment', 'government'],
    ['untill', 'until']
  ],
  fr: [
    ['etre', 'être'],
    ['ca', 'ça'],
    ['tres', 'très'],
    ['meme', 'même'],
    ['connaitre', 'connaître'],
    ['paraitre', 'paraître'],
    ['fenetre', 'fenêtre'],
    ['interet', 'intérêt'],
    ['foret', 'forêt'],
    ['arret', 'arrêt'],
    ['tete', 'tête'],
    ['hotel', 'hôtel'],
    ['grace', 'grâce'],
    ['cout', 'coût'],
    ['maitre', 'maître']
  ],
  de: [
    ['standart', 'standard'],
    ['rythmus', 'rhythmus'],
    ['maschiene', 'maschine'],
    ['interesant', 'interessant'],
    ['orginal', 'original'],
    ['seperat', 'separat'],
    ['paralell', 'parallel'],
    ['agressiv', 'aggressiv'],
    ['terasse', 'terrasse'],
    ['nähmlich', 'nämlich'],
    ['endgülitg', 'endgültig']
  ],
  pt: [
    ['porfavor', 'por favor'],
    ['concerteza', 'com certeza'],
    ['excessão', 'exceção'],
    ['exceçao', 'exceção'],
    ['previlegio', 'privilégio'],
    ['enchergar', 'enxergar'],
    ['muinto', 'muito'],
    ['voce', 'você'],
    ['tambem', 'também'],
    ['nao', 'não'],
    ['opçao', 'opção'],
    ['caza', 'casa']
  ],
  it: [
    ['pero', 'però'],
    ['cioe', 'cioè'],
    ['perche', 'perché'],
    ['poiche', 'poiché'],
    ['gia', 'già'],
    ['puo', 'può'],
    ['citta', 'città'],
    ['sopratutto', 'soprattutto'],
    ['daccordo', "d'accordo"],
    ['qualè', 'qual è'],
    ['percui', 'per cui'],
    ['affianco', 'a fianco']
  ],
  zh: [
    ['帐号', '账号'],
    ['做用', '作用'],
    ['部份', '部分'],
    ['另人', '令人'],
    ['以身作责', '以身作则']
  ],
  ja: [
    ['いう事', 'いうこと'],
    ['出来る', 'できる'],
    ['出来ます', 'できます'],
    ['事が', 'ことが'],
    ['無い', 'ない'],
    ['有る', 'ある'],
    ['下さい', 'ください'],
    ['頂く', 'いただく']
  ],
  ko: [
    ['되요', '돼요'],
    ['됬', '됐'],
    ['됌', '됨'],
    ['않돼', '안돼'],
    ['할께', '할게'],
    ['할꺼야', '할 거야'],
    ['왠만하면', '웬만하면'],
    ['금새', '금세']
  ],
  ru: [
    ['зделать', 'сделать'],
    ['вообщем', 'в общем'],
    ['вобщем', 'в общем'],
    ['как-будто', 'как будто'],
    ['извените', 'извините'],
    ['симпотичный', 'симпатичный'],
    ['через-чур', 'чересчур'],
    ['впринципе', 'в принципе'],
    ['вобщем-то', 'в общем-то'],
    ['до-свидания', 'до свидания']
  ]
};

/** Puntuación latina -> ancho completo (para texto CJK). */
const CJK_PUNCT: Record<string, string> = {
  ',': '，',
  ';': '；',
  ':': '：',
  '!': '！',
  '?': '？'
};

const CJK_RE = /[\u4e00-\u9fff\u3040-\u30ff\u31f0-\u31ff\uff00-\uffef]/;
const USES_CASE: ReadonlySet<LocaleCode> = new Set(['es', 'en', 'fr', 'de', 'pt', 'it', 'ru']);

function escapeRegExp(s: string): string {
  return s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function matchCase(sample: string, word: string): string {
  if (!sample) return word;
  if (sample[0] === sample[0].toUpperCase() && sample.slice(1) === sample.slice(1).toLowerCase()) {
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
  suggestion: string
): void {
  if (!original || original === suggestion) return;
  issues.push({ id: nextId++, type, index, length: original.length, original, suggestion });
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
    push(issues, 'doublespace', start, matched.slice(1), ' ');
  }

  // 2. Palabras repetidas: "la la", "que que".
  for (const m of text.matchAll(/\b(\p{L}+)(\s+)(\1)\b/giu)) {
    const full = m[0];
    const word = m[1];
    // "had had" / "que que" a veces es válido; aun así se sugiere revisar.
    push(issues, 'repeat', m.index ?? 0, full, word);
  }

  // 3. Puntuación repetida: "!!", "??", "...", ",,".
  for (const m of text.matchAll(/([!?])\1+|(\.\.\.+)|([,;:])\3+/g)) {
    const full = m[0];
    let suggestion = full[0];
    if (/^\.+$/.test(full)) suggestion = lang === 'zh' || lang === 'ja' ? '……' : '…';
    push(issues, 'punct', m.index ?? 0, full, suggestion);
  }

  // 4. Falta espacio tras puntuación: "hola,mundo" -> "hola, mundo".
  //    Se excluyen decimales/horas (3,14 / 12:30) y URLs.
  for (const m of text.matchAll(/([,;:!?])(?=\S)/g)) {
    const idx = m.index ?? 0;
    const before = text[idx - 1] ?? '';
    const after = text[idx + 1] ?? '';
    if (/\d/.test(before) && /\d/.test(after)) continue;
    if (before === ':' && /[/:]/.test(after)) continue; // http://, 12:30:45
    if (/[/\w]/.test(before) && after === '/' ) continue;
    // Emoticonos habituales :)
    if ((m[1] === ':' || m[1] === ';') && /[)D(P]/.test(after)) continue;
    const window = text.slice(Math.max(0, idx - 6), idx);
    if (/https?$/.test(window) || /www\.?$/.test(window)) continue;
    push(issues, 'spaceAfter', idx, m[1], `${m[1]} `);
  }

  // 5. Mayúscula inicial de frase (solo idiomas con mayúsculas).
  if (USES_CASE.has(lang)) {
    const startMatch = text.match(/^(\s*)(\p{Ll})/u);
    if (startMatch && startMatch[2]) {
      const idx = (startMatch[1]?.length ?? 0);
      push(issues, 'caps', idx, startMatch[2], startMatch[2].toUpperCase());
    }
    for (const m of text.matchAll(/([.!?…]\s+)([\p{Ll}])/gu)) {
      const prefixLen = m[1].length;
      const idx = (m.index ?? 0) + prefixLen;
      push(issues, 'caps', idx, m[2], m[2].toUpperCase());
    }
    // Pronombre inglés "i" aislado -> "I".
    if (lang === 'en') {
      for (const m of text.matchAll(/(?<![\p{L}'])i(?![\p{L}'])/gu)) {
        push(issues, 'caps', m.index ?? 0, 'i', 'I');
      }
    }
  }

  // 6. Puntuación latina dentro de texto CJK -> ancho completo.
  if (lang === 'zh' || lang === 'ja') {
    for (const m of text.matchAll(/([,;:!?])/g)) {
      const idx = m.index ?? 0;
      const before = text[idx - 1] ?? '';
      const after = text[idx + 1] ?? '';
      if (CJK_RE.test(before) || CJK_RE.test(after)) {
        const full = CJK_PUNCT[m[1]];
        if (full) push(issues, 'punct', idx, m[1], full);
      }
    }
  }

  // 7. Erratas frecuentes del idioma activo.
  const table = TYPO_TABLES[lang] ?? [];
  for (const [wrong, right] of table) {
    if (!wrong || wrong === right) continue;
    let re: RegExp;
    try {
      re = new RegExp(`(?<![\\p{L}])${escapeRegExp(wrong)}(?![\\p{L}])`, 'giu');
    } catch {
      continue;
    }
    for (const m of text.matchAll(re)) {
      const found = m[0];
      push(issues, 'typo', m.index ?? 0, found, matchCase(found, right));
    }
  }

  issues.sort((a, b) => a.index - b.index || a.length - b.length);
  return issues;
}

/** Aplica una incidencia a un texto y devuelve el texto resultante. */
export function applyIssue(text: string, issue: CorrectorIssue): string {
  return text.slice(0, issue.index) + issue.suggestion + text.slice(issue.index + issue.length);
}

/** Aplica todas las incidencias (de atrás hacia delante para no mover offsets). */
export function applyAllIssues(text: string, issues: CorrectorIssue[]): string {
  const ordered = [...issues].sort((a, b) => b.index - a.index);
  let out = text;
  for (const issue of ordered) {
    out = applyIssue(out, issue);
  }
  return out;
}

/** Cuenta palabras de forma razonable en los 10 idiomas. */
export function countWords(text: string, lang: LocaleCode): number {
  const trimmed = text.trim();
  if (!trimmed) return 0;
  if (lang === 'zh' || lang === 'ja') {
    const cjk = trimmed.match(/[\u4e00-\u9fff\u3040-\u30ff\u31f0-\u31ff]/g)?.length ?? 0;
    const latin = trimmed.match(/[A-Za-zÀ-ÿ0-9]+(?:['’][A-Za-zÀ-ÿ0-9]+)*/g)?.length ?? 0;
    return cjk + latin;
  }
  return trimmed.match(/\p{L}[\p{L}\p{N}'’-]*/gu)?.length ?? 0;
}

/** Longitud en caracteres (puntos de código, no UTF-16). */
export function countChars(text: string): number {
  return [...text].length;
}
