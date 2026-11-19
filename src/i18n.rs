//! Interface languages. English is the code's source language (the text passed to `t()`);
//! every other language is a file `assets/i18n/<code>.tsv` with lines «English text<TAB>translation».
//! Untranslated texts are shown in English. To add a language: copy es.tsv,
//! translate it and add its entry to `LANGUAGES` (see docs/EXTENDING.md).

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
/// One pass over the template: an argument that itself contains `{}` is never treated as a placeholder for the next argument.
pub fn tf(template: &str, args: &[&str]) -> String {
    let text = t(template);
    let mut out = String::with_capacity(text.len() + args.iter().map(|a| a.len()).sum::<usize>());
    let mut args = args.iter();
    let mut rest = text.as_str();
    while let Some(i) = rest.find("{}") {
        out.push_str(&rest[..i]);
        match args.next() {
            Some(a) => out.push_str(a),
            None => out.push_str("{}"), // more placeholders than arguments: left as they are
        }
        rest = &rest[i + 2..];
    }
    out.push_str(rest);
    out
}

/// Tests that switch the global language (`set`) or compare a text with `t()` take this lock so they do not run at the same time.
#[cfg(test)]
pub fn test_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

/// `t()` in a given language without touching the global one (tests).
#[cfg(test)]
pub fn t_in(code: &str, text: &str) -> String {
    match LANGUAGES.iter().position(|(c, _, _)| *c == code) {
        Some(i) if i > 0 => TABLES[i].get(text).map(|s| s.to_string()).unwrap_or_else(|| text.to_string()),
        _ => text.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translates_and_returns_to_english() {
        let _l = test_lock();
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
            // Each language translates at least 95 % of what Spanish translates (strings added later fall back to English until translated).
            assert!(views.len() * 100 >= base.len() * 95, "{}: only {} of {} texts", code, views.len(), base.len());
        }
    }

    #[test]
    fn keys_with_quotes_match_the_tsv_text() {
        let _l = test_lock();
        // A Rust literal with escaped quotes is the plain text in the TSV (no unescaping is done by the loader).
        set("es");
        assert_eq!(t("Check the details and press \"Send for review\"."), "Revisa los datos y pulsa «Enviar a revisión».");
        set("fr");
        assert_ne!(t("Check the details and press \"Send for review\"."), "Check the details and press \"Send for review\".");
        set("en");
    }

    #[test]
    fn tf_fills_the_gaps_in_order() {
        assert_eq!(tf("a {} b {}", &["1", "2"]), "a 1 b 2");
        assert_eq!(tf("no placeholders", &["x"]), "no placeholders");
        assert_eq!(tf("{}", &["{}"]), "{}");
        // an argument with `{}` does not take the next argument's place
        assert_eq!(tf("{} / {}", &["a{}b", "X"]), "a{}b / X");
        assert_eq!(tf("{} / {}", &["{}", "{}"]), "{} / {}");
        assert_eq!(tf("only {}", &["1", "2"]), "only 1");
        assert_eq!(tf("{} and {}", &["1"]), "1 and {}");
    }
}
