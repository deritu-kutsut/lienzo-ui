//! Interface languages. English is the code's source language (the text passed to `t()`);
//! every other language is a file `assets/i18n/<code>.tsv` with lines «English text<TAB>translation».
//! Untranslated texts are shown in English. To add a language: copy es.tsv,
//! translate it and add its entry to `LANGUAGES` (see docs/EXTENDER.md).

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::LazyLock;

/// (code, display name, contents of the .tsv)
pub const LANGUAGES: &[(&str, &str, &str)] = &[
    ("en", "English", ""),
    ("es", "Español", include_str!("../assets/i18n/es.tsv")),
    ("pt", "Português", include_str!("../assets/i18n/pt.tsv")),
    ("fr", "Français", include_str!("../assets/i18n/fr.tsv")),
    ("de", "Deutsch", include_str!("../assets/i18n/de.tsv")),
    ("it", "Italiano", include_str!("../assets/i18n/it.tsv")),
    ("ru", "Русский", include_str!("../assets/i18n/ru.tsv")),
    ("zh", "简体中文", include_str!("../assets/i18n/zh.tsv")),
    ("ja", "日本語", include_str!("../assets/i18n/ja.tsv")),
    ("ko", "한국어", include_str!("../assets/i18n/ko.tsv")),
];

static CURRENT: AtomicUsize = AtomicUsize::new(0);

static TABLES: LazyLock<Vec<HashMap<&'static str, &'static str>>> = LazyLock::new(|| {
    LANGUAGES
        .iter()
        .map(|(_, _, tsv)| {
            tsv.lines()
                .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
                .filter_map(|l| l.split_once('\t'))
                .map(|(es, tr)| (es.trim_end_matches('\r'), tr.trim_end_matches('\r')))
                .collect()
        })
        .collect()
});

/// Switches the language by its code («en», «es»…). An unknown code leaves English.
pub fn set(code: &str) {
    CURRENT.store(LANGUAGES.iter().position(|(c, _, _)| *c == code).unwrap_or(0), Ordering::Relaxed);
}

pub fn code() -> &'static str {
    LANGUAGES[CURRENT.load(Ordering::Relaxed)].0
}

/// Translates an interface text (English) to the current language (or leaves it as is if there is no translation).
pub fn t(text: &str) -> String {
    let i = CURRENT.load(Ordering::Relaxed);
    if i == 0 {
        return text.to_string();
    }
    TABLES[i].get(text).map(|s| s.to_string()).unwrap_or_else(|| text.to_string())
}

/// Translates a template with `{}` placeholders and fills them in order (for messages with data).
pub fn tf(template: &str, args: &[&str]) -> String {
    let mut out = t(template);
    for a in args {
        if let Some(i) = out.find("{}") {
            out.replace_range(i..i + 2, a);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translates_and_returns_to_english() {
        set("es");
        assert_eq!(t("Save"), "Guardar");
        assert_eq!(t("untranslated text xyz"), "untranslated text xyz");
        set("en");
        assert_eq!(t("Save"), "Save");
        set("zz");
        assert_eq!(code(), "en");
    }

    fn rows(tsv: &str) -> Vec<(&str, &str)> {
        tsv.lines()
            .map(|l| l.trim_end_matches('\r'))
            .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
            .map(|l| l.split_once('\t').unwrap_or_else(|| panic!("line without TAB: {}", l)))
            .collect()
    }

    #[test]
    fn tsv_files_are_well_formed() {
        let base: std::collections::HashSet<&str> = rows(LANGUAGES[1].2).into_iter().map(|(es, _)| es).collect();
        for (code, _, tsv) in &LANGUAGES[1..] {
            let mut views = std::collections::HashSet::new();
            for (es, tr) in rows(tsv) {
                assert!(!es.is_empty() && !tr.trim().is_empty(), "{}: “{}” is empty", code, es);
                assert!(views.insert(es), "{}: duplicate key «{}»", code, es);
                assert!(base.contains(es), "{}: key that does not exist in es.tsv «{}»", code, es);
                assert_eq!(es.matches("{}").count(), tr.matches("{}").count(), "{}: different {{}} placeholders in «{}»", code, es);
                assert_eq!(es.matches("\\n").count(), tr.matches("\\n").count(), "{}: different \\n line breaks in «{}»", code, es);
            }
            // Each language translates at least 60 % of what Spanish translates (strings added later fall back to English until translated).
            assert!(views.len() * 100 >= base.len() * 60, "{}: only {} of {} texts", code, views.len(), base.len());
        }
    }

    #[test]
    fn tf_fills_the_gaps_in_order() {
        assert_eq!(tf("a {} b {}", &["1", "2"]), "a 1 b 2");
        assert_eq!(tf("no placeholders", &["x"]), "no placeholders");
        assert_eq!(tf("{}", &["{}"]), "{}");
    }
}
