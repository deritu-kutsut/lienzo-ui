//! «Revisar diseño»: comprobaciones rápidas de legibilidad y usabilidad antes de exportar a Unity.
//! Todo es puro (sin interfaz) para usarlo desde la ventana, la línea de comandos (`--revisar`) y el MCP.

use crate::model::{Doc, Element, Kind, Rgba};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Nivel {
    Aviso,
    Error,
}

#[derive(Clone, Debug)]
pub struct Problema {
    pub nivel: Nivel,
    pub id: u64,
    pub elemento: String,
    /// Texto en español (se traduce al mostrarlo con `i18n::tf`).
    pub plantilla: &'static str,
    pub datos: Vec<String>,
}

/// Tamaño mínimo de letra legible a 1080 p de alto (se escala con la resolución de referencia).
const LETRA_MIN: f32 = 18.0;
/// Lado mínimo de un control pulsable a 1080 p (≈ 9 mm en un móvil de 6").
const TOQUE_MIN: f32 = 64.0;

fn luminancia(c: Rgba) -> f32 {
    let f = |v: u8| {
        let s = v as f32 / 255.0;
        if s <= 0.04045 { s / 12.92 } else { ((s + 0.055) / 1.055).powf(2.4) }
    };
    0.2126 * f(c[0]) + 0.7152 * f(c[1]) + 0.0722 * f(c[2])
}

/// Relación de contraste WCAG (1 a 21).
pub fn contraste(a: Rgba, b: Rgba) -> f32 {
    let (la, lb) = (luminancia(a), luminancia(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

fn mezcla(arriba: Rgba, abajo: Rgba) -> Rgba {
    let a = arriba[3] as f32 / 255.0;
    let m = |i: usize| (arriba[i] as f32 * a + abajo[i] as f32 * (1.0 - a)).round() as u8;
    [m(0), m(1), m(2), 255]
}

fn relleno(e: &Element) -> Rgba {
    // Las imágenes sin archivo son degradados: se usa el color medio.
    if e.kind == Kind::Image && e.image.is_empty() {
        let m = |i: usize| ((e.fill[i] as u16 + e.fill2[i] as u16) / 2) as u8;
        return [m(0), m(1), m(2), m(3)];
    }
    e.fill
}

fn visible(doc: &Doc, e: &Element) -> bool {
    let mut cur = Some(e);
    let mut pasos = 0;
    while let Some(x) = cur {
        if !x.visible || x.opacity <= 0.01 {
            return false;
        }
        pasos += 1;
        if pasos > 256 {
            break;
        }
        cur = x.parent.and_then(|p| doc.get(p));
    }
    true
}

fn tiene_fondo_propio(e: &Element) -> bool {
    matches!(e.kind, Kind::Button | Kind::InputField | Kind::Dropdown | Kind::Panel | Kind::Ellipse) && e.fill[3] > 0
}

/// Color sobre el que se ve el texto de `idx`: su propio relleno si lo tiene, o lo que hay debajo.
fn fondo_de(doc: &Doc, idx: usize) -> Rgba {
    let e = &doc.elements[idx];
    let (cx, cy) = (e.x + e.w / 2.0, e.y + e.h / 2.0);
    let mut color = doc.background;
    color[3] = 255;
    for (i, o) in doc.elements.iter().enumerate() {
        if i >= idx && !(i == idx && tiene_fondo_propio(e)) {
            break;
        }
        if o.kind == Kind::Text || o.kind == Kind::Empty || !o.image.is_empty() || !visible(doc, o) {
            continue;
        }
        if cx >= o.x && cx <= o.x + o.w && cy >= o.y && cy <= o.y + o.h {
            color = mezcla(relleno(o), color);
        }
    }
    color
}

fn es_pulsable(e: &Element) -> bool {
    e.interactable && matches!(e.kind, Kind::Button | Kind::Toggle | Kind::InputField | Kind::Dropdown)
}

/// Revisa el diseño y devuelve los problemas, errores primero.
pub fn revisar(doc: &Doc) -> Vec<Problema> {
    let escala = (doc.height.min(doc.width) / 1080.0).max(0.1);
    let mut out = Vec::new();
    let mut nombres: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for e in &doc.elements {
        *nombres.entry(e.name.as_str()).or_default() += 1;
    }
    for (idx, e) in doc.elements.iter().enumerate() {
        let mut p = |nivel, plantilla, datos: Vec<String>| out.push(Problema { nivel, id: e.id, elemento: e.name.clone(), plantilla, datos });
        if !visible(doc, e) {
            continue;
        }
        let con_texto = !e.text.trim().is_empty() && matches!(e.kind, Kind::Text | Kind::Button | Kind::Panel | Kind::Ellipse | Kind::Toggle);
        if con_texto {
            let min = LETRA_MIN * escala;
            if e.font_size < min {
                p(Nivel::Aviso, "Letra de {} px: en pantallas pequeñas será difícil de leer (mínimo recomendado {} px)", vec![format!("{:.0}", e.font_size), format!("{:.0}", min)]);
            }
            // WCAG no exige contraste a los controles desactivados.
            let desactivado = !e.interactable && matches!(e.kind, Kind::Button | Kind::Toggle);
            let fondo = fondo_de(doc, idx);
            let c = contraste(mezcla(e.text_color, fondo), fondo);
            // «Texto grande» de WCAG: 18 pt (24 px) o 14 pt en negrita (≈18,7 px), a la escala de referencia.
            let grande = e.font_size >= 24.0 * escala || (e.bold && e.font_size >= 18.7 * escala);
            let min_c = if grande { 3.0 } else { 4.5 };
            if c < min_c && !desactivado {
                let nivel = if c < 2.0 { Nivel::Error } else { Nivel::Aviso };
                p(nivel, "Contraste del texto {}:1 (WCAG pide {}:1)", vec![format!("{:.1}", c), format!("{:.1}", min_c)]);
            }
        }
        if es_pulsable(e) {
            let min = TOQUE_MIN * escala;
            if e.w.min(e.h) < min {
                p(Nivel::Aviso, "Control de {} px: difícil de pulsar con el dedo (mínimo recomendado {} px)", vec![format!("{:.0}", e.w.min(e.h)), format!("{:.0}", min)]);
            }
        }
        let fuera = e.x + e.w < 0.0 || e.y + e.h < 0.0 || e.x > doc.width || e.y > doc.height;
        if fuera && e.rot.abs() < 0.01 {
            p(Nivel::Error, "Está completamente fuera de la página: no se verá en el juego", vec![]);
        } else if e.parent.is_none() && (e.x < -1.0 || e.y < -1.0 || e.x + e.w > doc.width + 1.0 || e.y + e.h > doc.height + 1.0) && e.rot.abs() < 0.01 {
            p(Nivel::Aviso, "Se sale en parte de la página", vec![]);
        }
        if es_pulsable(e) && nombres.get(e.name.as_str()).copied().unwrap_or(0) > 1 {
            p(Nivel::Aviso, "Hay varios elementos llamados así: GameObject.Find y los scripts encontrarán solo el primero", vec![]);
        }
        if e.kind == Kind::Image && !e.image.is_empty() && !crate::library::resolve_image(&e.image).exists() {
            p(Nivel::Error, "Falta la imagen «{}»: se exportará vacía", vec![e.image.clone()]);
        }
    }
    out.sort_by(|a, b| b.nivel.cmp(&a.nivel));
    out
}

/// Texto de un problema en el idioma actual.
pub fn describir(p: &Problema) -> String {
    let args: Vec<&str> = p.datos.iter().map(|s| s.as_str()).collect();
    crate::i18n::tf(p.plantilla, &args)
}

/// `--revisar`: imprime los problemas; código 1 si hay errores.
pub fn informe(doc: &Doc) -> (String, bool) {
    let ps = revisar(doc);
    if ps.is_empty() {
        return (crate::i18n::t("Sin problemas: el diseño pasa todas las comprobaciones."), false);
    }
    let errores = ps.iter().any(|p| p.nivel == Nivel::Error);
    let lineas: Vec<String> = ps
        .iter()
        .map(|p| format!("[{}] {}: {}", if p.nivel == Nivel::Error { "ERROR" } else { "AVISO" }, p.elemento, describir(p)))
        .collect();
    (lineas.join("\n"), errores)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Doc, Element, Kind};

    fn doc_con(e: Element) -> Doc {
        let mut d = Doc::empty(1920.0, 1080.0);
        d.background = [255, 255, 255, 255];
        d.add(e);
        d
    }

    #[test]
    fn contraste_conocido() {
        assert!((contraste([0, 0, 0, 255], [255, 255, 255, 255]) - 21.0).abs() < 0.01);
        assert!((contraste([10, 20, 30, 255], [10, 20, 30, 255]) - 1.0).abs() < 0.001);
    }

    #[test]
    fn detecta_texto_ilegible_y_boton_pequeno() {
        let mut t = Element::new(0, Kind::Text);
        t.text = "hola".into();
        t.text_color = [240, 240, 240, 255];
        t.font_size = 12.0;
        t.x = 100.0;
        t.y = 100.0;
        let ps = revisar(&doc_con(t));
        assert!(ps.iter().any(|p| p.plantilla.starts_with("Letra")));
        assert!(ps.iter().any(|p| p.plantilla.starts_with("Contraste") && p.nivel == Nivel::Error));

        let mut b = Element::new(0, Kind::Button);
        b.x = 100.0;
        b.y = 100.0;
        b.w = 40.0;
        b.h = 40.0;
        let ps = revisar(&doc_con(b));
        assert!(ps.iter().any(|p| p.plantilla.starts_with("Control")));
    }

    #[test]
    fn fuera_de_pagina_y_nombres_repetidos() {
        let mut b = Element::new(0, Kind::Button);
        b.x = 5000.0;
        let ps = revisar(&doc_con(b));
        assert!(ps.iter().any(|p| p.nivel == Nivel::Error && p.plantilla.contains("fuera")));
        let mut d = Doc::empty(1920.0, 1080.0);
        for _ in 0..2 {
            let mut b = Element::new(0, Kind::Button);
            b.name = "Jugar".into();
            b.x = 100.0;
            b.y = 100.0;
            d.add(b);
        }
        assert!(revisar(&d).iter().any(|p| p.plantilla.contains("GameObject.Find")));
    }

    #[test]
    fn plantillas_incluidas_sin_errores() {
        for i in 0..crate::model::TEMPLATES.len() {
            let d = crate::model::template(i, 1920.0, 1080.0);
            let errores: Vec<String> = revisar(&d).iter().filter(|p| p.nivel == Nivel::Error).map(|p| format!("{}: {}", p.elemento, describir(p))).collect();
            assert!(errores.is_empty(), "{}: {:?}", crate::model::TEMPLATES[i], errores);
        }
    }
}

impl crate::app::EditorApp {
    /// Ventana «Revisar diseño»: lista de problemas; clic selecciona el elemento.
    pub fn revision_window(&mut self, ctx: &eframe::egui::Context) {
        use crate::i18n::t;
        use eframe::egui::{self, Color32, RichText};
        if !self.revision_abierta {
            return;
        }
        let problemas = revisar(&self.doc);
        let mut abierta = true;
        let mut elegir = None;
        egui::Window::new(t("Revisar diseño")).open(&mut abierta).collapsible(false).default_width(460.0).show(ctx, |ui| {
            let errores = problemas.iter().filter(|p| p.nivel == Nivel::Error).count();
            let avisos = problemas.len() - errores;
            if problemas.is_empty() {
                ui.label(RichText::new(t("Sin problemas: el diseño pasa todas las comprobaciones.")).color(crate::theme::ACCENT()));
            } else {
                ui.label(crate::i18n::tf("{} errores · {} avisos. Clic en uno para seleccionarlo.", &[&errores.to_string(), &avisos.to_string()]));
            }
            ui.add_space(6.0);
            egui::ScrollArea::vertical().max_height(360.0).show(ui, |ui| {
                for p in &problemas {
                    let (etq, col) = match p.nivel {
                        Nivel::Error => (t("Error"), Color32::from_rgb(235, 80, 90)),
                        Nivel::Aviso => (t("Aviso"), Color32::from_rgb(230, 170, 60)),
                    };
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new(etq).strong().color(col));
                        if ui.link(&p.elemento).clicked() {
                            elegir = Some(p.id);
                        }
                        ui.label(RichText::new(describir(p)).color(crate::theme::MUTED()));
                    });
                }
            });
            ui.add_space(6.0);
            ui.label(RichText::new(t("Los mínimos se calculan para la resolución de referencia (18 px de letra y 64 px por control a 1080 p).")).small().color(crate::theme::MUTED()));
        });
        if let Some(id) = elegir {
            self.sel = vec![id];
        }
        self.revision_abierta = abierta;
    }
}
