/// Extract the base ISO 639-1 language code from a regional variant.
/// e.g. "pt-BR" → "pt", "zh-CN" → "zh", "en" → "en"
pub fn lang_base(lang: &str) -> &str {
    lang.split('-').next().unwrap_or(lang)
}

/// Extract the ISO 3166-1 region from a regional language code.
/// e.g. "pt-BR" → Some("BR"), "en" → None
pub fn lang_region(lang: &str) -> Option<&str> {
    lang.split_once('-').map(|(_, r)| r)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lang_base_and_region() {
        assert_eq!(lang_base("pt-BR"), "pt");
        assert_eq!(lang_base("zh-CN"), "zh");
        assert_eq!(lang_base("en"), "en");
        assert_eq!(lang_base(""), "");

        assert_eq!(lang_region("pt-BR"), Some("BR"));
        assert_eq!(lang_region("zh-CN"), Some("CN"));
        assert_eq!(lang_region("en"), None);
        assert_eq!(lang_region(""), None);
    }
}
