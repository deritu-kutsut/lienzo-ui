//! Interface languages. Spanish is the code's base language; each extra language is a
//! file `assets/i18n/<code>.tsv` with lines «Spanish text<TAB>translation».
//! Untranslated texts are shown in Spanish. To add a language: copy en.tsv,
//! translate it and add its entry to `IDIOMAS` (see docs/EXTENDER.md).

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::LazyLock;

/// (code, display name, contents of the .tsv)
pub const IDIOMAS: &[(&str, &str, &str)] = &[
    ("es", "Español", ""),
    ("en", "English", include_str!("../assets/i18n/en.tsv")),
    ("pt", "Português", include_str!("../assets/i18n/pt.tsv")),
    ("fr", "Français", include_str!("../assets/i18n/fr.tsv")),
    ("de", "Deutsch", include_str!("../assets/i18n/de.tsv")),
    ("it", "Italiano", include_str!("../assets/i18n/it.tsv")),
    ("ru", "Русский", include_str!("../assets/i18n/ru.tsv")),
    ("zh", "简体中文", include_str!("../assets/i18n/zh.tsv")),
    ("ja", "日本語", include_str!("../assets/i18n/ja.tsv")),
    ("ko", "한국어", include_str!("../assets/i18n/ko.tsv")),
];

static ACTUAL: AtomicUsize = AtomicUsize::new(0);

static TABLAS: LazyLock<Vec<HashMap<&'static str, &'static str>>> = LazyLock::new(|| {
    IDIOMAS
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

/// Switches the language by its code («es», «en»…). An unknown code leaves Spanish.
pub fn set(codigo: &str) {
    ACTUAL.store(IDIOMAS.iter().position(|(c, _, _)| *c == codigo).unwrap_or(0), Ordering::Relaxed);
}

pub fn codigo() -> &'static str {
    IDIOMAS[ACTUAL.load(Ordering::Relaxed)].0
}

/// Translates an interface text to the current language (or leaves it as is if there is no translation).
pub fn t(texto: &str) -> String {
    let i = ACTUAL.load(Ordering::Relaxed);
    if i == 0 {
        return texto.to_string();
    }
    TABLAS[i].get(texto).map(|s| s.to_string()).unwrap_or_else(|| texto.to_string())
}

/// Translates a template with `{}` placeholders and fills them in order (for messages with data).
pub fn tf(plantilla: &str, args: &[&str]) -> String {
    let mut out = t(plantilla);
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
    fn traduce_y_vuelve_al_espanol() {
        set("en");
        assert_eq!(t("Guardar"), "Save");
        assert_eq!(t("texto sin traducción xyz"), "texto sin traducción xyz");
        set("es");
        assert_eq!(t("Guardar"), "Guardar");
        set("zz");
        assert_eq!(codigo(), "es");
    }

    fn filas(tsv: &str) -> Vec<(&str, &str)> {
        tsv.lines()
            .map(|l| l.trim_end_matches('\r'))
            .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
            .map(|l| l.split_once('\t').unwrap_or_else(|| panic!("línea sin TAB: {}", l)))
            .collect()
    }

    #[test]
    fn los_tsv_estan_bien_formados() {
        let base: std::collections::HashSet<&str> = filas(IDIOMAS[1].2).into_iter().map(|(es, _)| es).collect();
        for (codigo, _, tsv) in &IDIOMAS[1..] {
            let mut vistas = std::collections::HashSet::new();
            for (es, tr) in filas(tsv) {
                assert!(!es.is_empty() && !tr.trim().is_empty(), "{}: «{}» vacía", codigo, es);
                assert!(vistas.insert(es), "{}: clave repetida «{}»", codigo, es);
                assert!(base.contains(es), "{}: clave que no existe en en.tsv «{}»", codigo, es);
                assert_eq!(es.matches("{}").count(), tr.matches("{}").count(), "{}: huecos {{}} distintos en «{}»", codigo, es);
                assert_eq!(es.matches("\\n").count(), tr.matches("\\n").count(), "{}: saltos \\n distintos en «{}»", codigo, es);
            }
            // Each language translates at least 95 % of what English translates.
            assert!(vistas.len() * 100 >= base.len() * 95, "{}: solo {} de {} textos", codigo, vistas.len(), base.len());
        }
    }

    #[test]
    fn tf_rellena_los_huecos_en_orden() {
        assert_eq!(tf("a {} b {}", &["1", "2"]), "a 1 b 2");
        assert_eq!(tf("sin huecos", &["x"]), "sin huecos");
        assert_eq!(tf("{}", &["{}"]), "{}");
    }
}
