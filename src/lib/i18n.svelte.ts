export class I18nManager {
  currentLang = $state('en');

  private translations: Record<string, Record<string, string>> = {
    en: {
      "nav.profiles": "Profiles",
      "nav.settings": "Settings",
      "profiles.active": "Active",
      "profiles.apply": "Apply",
      "profiles.edit": "Edit",
      "profiles.empty": "No profiles found.",
      "profiles.add": "+ Add New Profile",
      "editor.name": "Name",
      "editor.description": "Description",
      "editor.info": "Basic Info",
      "editor.brightness": "Brightness",
      "editor.contrast": "Contrast",
      "editor.gamma": "Gamma",
      "editor.vibrance": "Digital Vibrance",
      "editor.title": "Edit Profile",
      "editor.new_profile": "New Profile",
      "editor.settings": "Settings",
      "editor.save": "Save",
      "editor.cancel": "Cancel",
      "editor.delete": "Delete",
      "settings.title": "Global Settings",
      "settings.behavior": "Behavior",
      "settings.hotkeys": "Hotkeys",
      "settings.language": "Language",
      "settings.notifications": "SHOW NOTIFICATIONS",
      "settings.minimized": "START MINIMIZED",
      "settings.autostart": "START WITH WINDOWS",
      "settings.save": "Save Changes",
      "hotkey.next": "NEXT PROFILE",
      "hotkey.prev": "PREVIOUS PROFILE",
      "hotkey.reset": "RESET TO DEFAULT",
      "hotkey.recording": "RECORDING... PRESS KEYS",
      "hotkey.none": "NOT CONFIGURED",
      "app.status.loading": "Loading...",
      "app.status.ready": "Ready",
      "app.status.applying": "Applying...",
      "app.status.saving": "Saving...",
      "app.status.saved": "Saved",
      "app.status.deleting": "Deleting...",
      "app.status.deleted": "Deleted",
      "app.confirm.delete": "Are you sure you want to delete this profile?",
      "app.status.error": "Error"
    },
    ru: {
      "nav.profiles": "Профили",
      "nav.settings": "Настройки",
      "profiles.active": "Активен",
      "profiles.apply": "Применить",
      "profiles.edit": "Изменить",
      "profiles.empty": "Профили не найдены.",
      "profiles.add": "+ Добавить профиль",
      "editor.name": "Название",
      "editor.description": "Описание",
      "editor.info": "Основная информация",
      "editor.brightness": "Яркость",
      "editor.contrast": "Контраст",
      "editor.gamma": "Гамма",
      "editor.vibrance": "Насыщенность (NV)",
      "editor.title": "Редактировать профиль",
      "editor.new_profile": "Новый профиль",
      "editor.settings": "Настройки",
      "editor.save": "Сохранить",
      "editor.cancel": "Отмена",
      "editor.delete": "Удалить",
      "settings.title": "Общие настройки",
      "settings.behavior": "Поведение",
      "settings.hotkeys": "Горячие клавиши",
      "settings.language": "Язык",
      "settings.notifications": "ВКЛЮЧИТЬ УВЕДОМЛЕНИЯ",
      "settings.minimized": "ЗАПУСК СВЕРНУТЫМ",
      "settings.autostart": "ЗАПУСК ПРИ СТАРТЕ WINDOWS",
      "settings.save": "Сохранить изменения",
      "hotkey.next": "СЛЕДУЮЩИЙ ПРОФИЛЬ",
      "hotkey.prev": "ПРЕДЫДУЩИЙ ПРОФИЛЬ",
      "hotkey.reset": "СБРОС НАСТРОЕК",
      "hotkey.recording": "ЗАПИСЬ... НАЖМИТЕ КЛАВИШИ",
      "hotkey.none": "НЕ НАЗНАЧЕНО",
      "app.status.loading": "Загрузка...",
      "app.status.ready": "Готов",
      "app.status.applying": "Применение...",
      "app.status.saving": "Сохранение...",
      "app.status.saved": "Сохранено",
      "app.status.deleting": "Удаление...",
      "app.status.deleted": "Удалено",
      "app.confirm.delete": "Вы уверены, что хотите удалить этот профиль?",
      "app.status.error": "Ошибка"
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
