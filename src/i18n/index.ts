import { languages, type LanguageCode } from './languages';

// Eagerly load all translation.json files from the locales directory
const modules = import.meta.glob('./locales/*/translation.json', { eager: true });

const translations: Record<string, Record<string, string>> = {};

for (const path in modules) {
  // Path format: ./locales/en/translation.json
  const segments = path.split('/');
  const lang = segments[segments.length - 2]; 
  translations[lang] = (modules[path] as any).default;
}

export class I18nManager {
  currentLang = $state<LanguageCode>('en');

  /**
   * Translates a key based on the current language.
   * Returns the key if translation is missing.
   */
  t(key: string): string {
    return translations[this.currentLang]?.[key] ?? key;
  }

  /**
   * Updates the current language if supported.
   */
  setLanguage(lang: string) {
    if (translations[lang]) {
      this.currentLang = lang as LanguageCode;
    }
  }
}

export const i18n = new I18nManager();
