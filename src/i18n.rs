//! Idiomas de la interfaz. El español es el idioma base del código; cada idioma extra es un
//! archivo `assets/i18n/<código>.tsv` con líneas «texto en español<TAB>traducción».
//! Los textos sin traducción se muestran en español. Para añadir un idioma: copia en.tsv,
//! tradúcelo y añade su entrada a `IDIOMAS` (ver docs/EXTENDER.md).

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::LazyLock;

/// (código, nombre visible, contenido del .tsv)
pub const IDIOMAS: &[(&str, &str, &str)] = &[("es", "Español", ""), ("en", "English", include_str!("../assets/i18n/en.tsv"))];

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

/// Cambia el idioma por su código («es», «en»…). Un código desconocido deja el español.
pub fn set(codigo: &str) {
    ACTUAL.store(IDIOMAS.iter().position(|(c, _, _)| *c == codigo).unwrap_or(0), Ordering::Relaxed);
}

pub fn codigo() -> &'static str {
    IDIOMAS[ACTUAL.load(Ordering::Relaxed)].0
}

pub fn es_espanol() -> bool {
    ACTUAL.load(Ordering::Relaxed) == 0
}

/// Traduce un texto de la interfaz al idioma actual (o lo deja igual si no hay traducción).
pub fn t(texto: &str) -> String {
    let i = ACTUAL.load(Ordering::Relaxed);
    if i == 0 {
        return texto.to_string();
    }
    TABLAS[i].get(texto).map(|s| s.to_string()).unwrap_or_else(|| texto.to_string())
}

/// Traduce una plantilla con huecos `{}` y los rellena en orden (para mensajes con datos).
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

    #[test]
    fn el_tsv_ingles_esta_bien_formado() {
        for (n, l) in IDIOMAS[1].2.lines().enumerate() {
            if l.trim().is_empty() || l.starts_with('#') {
                continue;
            }
            let (es, en) = l.split_once('\t').unwrap_or_else(|| panic!("línea {} sin TAB: {}", n + 1, l));
            assert!(!es.is_empty() && !en.trim().is_empty(), "línea {} vacía", n + 1);
        }
    }

    #[test]
    fn tf_rellena_los_huecos_en_orden() {
        assert_eq!(tf("a {} b {}", &["1", "2"]), "a 1 b 2");
        assert_eq!(tf("sin huecos", &["x"]), "sin huecos");
        assert_eq!(tf("{}", &["{}"]), "{}");
    }
}
