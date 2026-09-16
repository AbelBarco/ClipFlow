export type TransformerType =
  | 'uppercase'
  | 'lowercase'
  | 'title_case'
  | 'snake_case'
  | 'kebab_case'
  | 'camel_case'
  | 'pascal_case'
  | 'trim'
  | 'slug'
  | 'json_pretty'
  | 'json_minify'
  | 'url_encode'
  | 'url_decode'
  | 'base64_encode'
  | 'base64_decode';

export interface Transformer {
  id: TransformerType;
  label: string;
  description: string;
  category: 'text' | 'code' | 'encoding' | 'formatting';
  shortcut?: string;
}

export const TRANSFORMERS: Transformer[] = [
  { id: 'uppercase', label: 'UPPERCASE', description: 'Convert to uppercase', category: 'text', shortcut: '⌘⇧U' },
  { id: 'lowercase', label: 'lowercase', description: 'Convert to lowercase', category: 'text', shortcut: '⌘⇧L' },
  { id: 'title_case', label: 'Title Case', description: 'Capitalize each word', category: 'text' },
  { id: 'snake_case', label: 'snake_case', description: 'Convert to snake_case', category: 'code' },
  { id: 'kebab_case', label: 'kebab-case', description: 'Convert to kebab-case', category: 'code' },
  { id: 'camel_case', label: 'camelCase', description: 'Convert to camelCase', category: 'code' },
  { id: 'pascal_case', label: 'PascalCase', description: 'Convert to PascalCase', category: 'code' },
  { id: 'trim', label: 'Trim', description: 'Remove leading/trailing whitespace', category: 'formatting' },
  { id: 'slug', label: 'Slugify', description: 'Convert to URL-friendly slug', category: 'formatting' },
  { id: 'json_pretty', label: 'JSON Pretty', description: 'Format JSON with indentation', category: 'code' },
  { id: 'json_minify', label: 'JSON Minify', description: 'Minify JSON', category: 'code' },
  { id: 'url_encode', label: 'URL Encode', description: 'Encode for URL', category: 'encoding' },
  { id: 'url_decode', label: 'URL Decode', description: 'Decode from URL', category: 'encoding' },
  { id: 'base64_encode', label: 'Base64 Encode', description: 'Encode to Base64', category: 'encoding' },
  { id: 'base64_decode', label: 'Base64 Decode', description: 'Decode from Base64', category: 'encoding' }
];