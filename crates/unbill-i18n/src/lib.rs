//! Rust-native internationalization primitives for Unbill.

/// Supported locales for built-in translations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Locale {
    #[default]
    En,
    ZhCn,
}

impl Locale {
    pub fn parse(tag: &str) -> Self {
        match tag {
            "zh-CN" | "zh" | "cn" => Self::ZhCn,
            _ => Self::En,
        }
    }
}

/// Stable translation keys used across Unbill surfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MessageKey {
    AppTitle,
    ActionCreateLedger,
    ActionAddBill,
    StatusNoLedgers,
}

impl MessageKey {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AppTitle => "app.title",
            Self::ActionCreateLedger => "action.create_ledger",
            Self::ActionAddBill => "action.add_bill",
            Self::StatusNoLedgers => "status.no_ledgers",
        }
    }
}

#[derive(Debug, Default)]
pub struct I18n;

impl I18n {
    pub fn new() -> Self {
        Self
    }

    pub fn t(&self, locale: Locale, key: MessageKey) -> &'static str {
        translate(locale, key)
            .or_else(|| translate(Locale::En, key))
            .unwrap_or(key.as_str())
    }
}

fn translate(locale: Locale, key: MessageKey) -> Option<&'static str> {
    match (locale, key) {
        (Locale::En, MessageKey::AppTitle) => Some("Unbill"),
        (Locale::En, MessageKey::ActionCreateLedger) => Some("Create ledger"),
        (Locale::En, MessageKey::ActionAddBill) => Some("Add bill"),
        (Locale::En, MessageKey::StatusNoLedgers) => Some("No ledgers yet"),

        (Locale::ZhCn, MessageKey::AppTitle) => Some("Unbill"),
        (Locale::ZhCn, MessageKey::ActionCreateLedger) => Some("创建账本"),
        (Locale::ZhCn, MessageKey::ActionAddBill) => Some("添加账单"),
        // Intentionally omit StatusNoLedgers to exercise fallback behavior.
        (Locale::ZhCn, MessageKey::StatusNoLedgers) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{I18n, Locale, MessageKey};

    #[test]
    fn translates_english_messages() {
        let i18n = I18n::new();

        assert_eq!(i18n.t(Locale::En, MessageKey::ActionCreateLedger), "Create ledger");
    }

    #[test]
    fn translates_chinese_messages() {
        let i18n = I18n::new();

        assert_eq!(i18n.t(Locale::ZhCn, MessageKey::ActionAddBill), "添加账单");
    }

    #[test]
    fn parses_unknown_locale_to_english() {
        assert_eq!(Locale::parse("xx"), Locale::En);
    }

    #[test]
    fn falls_back_to_english_when_locale_key_is_missing() {
        let i18n = I18n::new();

        assert_eq!(i18n.t(Locale::ZhCn, MessageKey::StatusNoLedgers), "No ledgers yet");
    }
}
