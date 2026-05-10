pub fn t(lang: &str, key: &'static str) -> &'static str {
    match lang {
        "ru" => match key {
            "tray.quit" => "Выход",
            "tray.show" => "Показать интерфейс",
            "notif.title" => "Профиль изменен",
            "notif.body" => "Активный профиль: {}",
            _ => key,
        },
        _ => match key {
            "tray.quit" => "Quit",
            "tray.show" => "Show UI",
            "notif.title" => "Profile Switched",
            "notif.body" => "Active profile: {}",
            _ => key,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translations_en() {
        assert_eq!(t("en", "tray.quit"), "Quit");
        assert_eq!(t("en", "notif.title"), "Profile Switched");
        assert_eq!(t("en", "unknown"), "unknown");
    }

    #[test]
    fn test_translations_ru() {
        assert_eq!(t("ru", "tray.quit"), "Выход");
        assert_eq!(t("ru", "notif.title"), "Профиль изменен");
        assert_eq!(t("ru", "unknown"), "unknown");
    }
}
