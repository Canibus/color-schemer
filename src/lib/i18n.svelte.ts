export class I18nManager {
  currentLang = $state('en');

  private translations: Record<string, Record<string, string>> = {
    en: {
      "nav.profiles": "Profiles",
      "nav.settings": "Settings",
      "profiles.active": "Active",
      "profiles.apply": "Apply",
      "profiles.edit": "Edit",
      "editor.name": "Name",
      "editor.description": "Description",
      "editor.brightness": "Brightness",
      "editor.contrast": "Contrast",
      "editor.gamma": "Gamma",
      "editor.vibrance": "Digital Vibrance",
      "editor.save": "Save",
      "editor.cancel": "Cancel",
      "settings.hotkeys": "Hotkeys",
      "settings.language": "Language",
      "settings.notifications": "Show Notifications",
      "settings.minimized": "Start Minimized",
      "settings.save": "Save Changes",
      "hotkey.next": "Next Profile",
      "hotkey.prev": "Previous Profile",
      "hotkey.reset": "Reset to Default",
      "hotkey.recording": "Recording... Press keys"
    },
    ru: {
      "nav.profiles": "Профили",
      "nav.settings": "Настройки",
      "profiles.active": "Активен",
      "profiles.apply": "Применить",
      "profiles.edit": "Изменить",
      "editor.name": "Название",
      "editor.description": "Описание",
      "editor.brightness": "Яркость",
      "editor.contrast": "Контраст",
      "editor.gamma": "Гамма",
      "editor.vibrance": "Насыщенность (NV)",
      "editor.save": "Сохранить",
      "editor.cancel": "Отмена",
      "settings.hotkeys": "Горячие клавиши",
      "settings.language": "Язык",
      "settings.notifications": "Уведомления",
      "settings.minimized": "Запуск свернутым",
      "settings.save": "Сохранить изменения",
      "hotkey.next": "Следующий профиль",
      "hotkey.prev": "Предыдущий профиль",
      "hotkey.reset": "Сброс",
      "hotkey.recording": "Запись... Нажмите клавиши"
    }
  };

  t(key: string): string {
    return this.translations[this.currentLang]?.[key] ?? key;
  }

  setLanguage(lang: string) {
    if (this.translations[lang]) {
      this.currentLang = lang;
    }
  }
}

export const i18n = new I18nManager();
