pub fn t(lang: &str, key: &'static str) -> &'static str {
    match lang {
        "ru" => match key {
            "tray.quit" => "Выход",
            "tray.show" => "Показать интерфейс",
            "unknown" => "unknown",
            _ => key,
        },
        "en" => match key {
            "tray.quit" => "Quit",
            "tray.show" => "Show GUI",
            "unknown" => "unknown",
            _ => key,
        },
        _ => key,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translations_en() {
        assert_eq!(t("en", "tray.quit"), "Quit");
        assert_eq!(t("en", "unknown"), "unknown");
    }

    #[test]
    fn test_translations_ru() {
        assert_eq!(t("ru", "tray.quit"), "Выход");
        assert_eq!(t("ru", "unknown"), "unknown");
    }
}
