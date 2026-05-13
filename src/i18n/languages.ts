export const languages = [
  { code: 'en', label: 'English' },
  { code: 'ru', label: 'Русский' },
] as const;

export type LanguageCode = typeof languages[number]['code'];
