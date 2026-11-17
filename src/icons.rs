//! Minimal line icons (SVG, white strokes tinted at draw time). The set lives in
//! `assets/icons/icons.json` as {"name": "<svg …>"}; a missing icon falls back to
//! a short text label so the editor always works.

use eframe::egui::{self, Color32, Rect, Response, Sense, Ui, Vec2};
use std::collections::HashMap;
use std::sync::OnceLock;

const SET: &str = include_str!("../assets/icons/icons.json");

fn set() -> &'static HashMap<String, &'static [u8]> {
    static S: OnceLock<HashMap<String, &'static [u8]>> = OnceLock::new();
    S.get_or_init(|| {
        let map: HashMap<String, String> = serde_json::from_str(SET).unwrap_or_default();
        map.into_iter()
            .map(|(k, v)| {
                // strokes/fills are drawn white so `tint` gives them any colour
                let v = v.replace("currentColor", "#FFFFFF");
                let b: &'static [u8] = Box::leak(v.into_bytes().into_boxed_slice());
                (k, b)
            })
            .collect()
    })
}

/// Icon as an egui image of `size` points, tinted.
pub fn image(name: &str, size: f32, tint: Color32) -> Option<egui::Image<'static>> {
    let bytes = *set().get(name)?;
    Some(egui::Image::from_bytes(format!("bytes://icon/{}.svg", name), bytes).fit_to_exact_size(Vec2::splat(size)).tint(tint))
}

/// Paint an icon centred in `rect`. Returns false when the icon is not in the set.
pub fn paint(ui: &Ui, name: &str, rect: Rect, tint: Color32) -> bool {
    match image(name, rect.width().min(rect.height()), tint) {
        Some(img) => {
            img.paint_at(ui, rect);
            true
        }
        None => false,
    }
}

/// Paint an icon, or `fallback` text when it is missing.
pub fn paint_or(ui: &Ui, name: &str, rect: Rect, tint: Color32, fallback: &str) {
    if !paint(ui, name, rect, tint) {
        ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, fallback, egui::FontId::proportional(rect.height() * 0.8), tint);
    }
}

/// Small square icon button (frameless until hovered).
pub fn button(ui: &mut Ui, name: &str, fallback: &str, tip: &str) -> Response {
    let size = ui.spacing().interact_size.y;
    let (r, resp) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());
    let enabled = ui.is_enabled();
    if resp.hovered() && enabled {
        ui.painter().rect_filled(r, 5.0, crate::theme::HOVER());
    }
    let c = if !enabled { crate::theme::MUTED().gamma_multiply(0.5) } else if resp.hovered() { Color32::WHITE } else { crate::theme::TEXT() };
    paint_or(ui, name, r.shrink(4.0), c, fallback);
    resp.on_hover_text(tip)
}

/// Same as [`button`] but drawn as "selected" when `on`.
pub fn toggle(ui: &mut Ui, on: bool, name: &str, fallback: &str, tip: &str) -> Response {
    let size = ui.spacing().interact_size.y;
    let (r, resp) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());
    if on {
        ui.painter().rect_filled(r, 5.0, crate::theme::ACCENT_SOFT());
    } else if resp.hovered() {
        ui.painter().rect_filled(r, 5.0, crate::theme::HOVER());
    }
    let c = if on || resp.hovered() { Color32::WHITE } else { crate::theme::TEXT() };
    paint_or(ui, name, r.shrink(4.0), c, fallback);
    resp.on_hover_text(tip)
}

/// Button with a leading icon (falls back to text only).
pub fn btn<'a>(name: &str, text: impl Into<egui::WidgetText>) -> egui::Button<'a> {
    btn_tint(name, text, crate::theme::TEXT())
}

pub fn btn_tint<'a>(name: &str, text: impl Into<egui::WidgetText>, tint: Color32) -> egui::Button<'a> {
    match image(name, 15.0, tint) {
        Some(img) => egui::Button::image_and_text(img, text),
        None => egui::Button::new(text),
    }
}

/// Icono (sin texto) como botón redondo sobre una imagen, p. ej. las cámaras del editor de perfil.
pub fn redondo<'a>(name: &str, lado: f32, tint: Color32) -> egui::Button<'a> {
    let b = match image(name, lado * 0.42, tint) {
        Some(img) => egui::Button::image(img),
        None => egui::Button::new(""),
    };
    b.fill(Color32::from_black_alpha(150)).corner_radius(lado / 2.0).min_size(Vec2::splat(lado))
}

/// Icono pequeño seguido de un texto (contadores): imagen + texto en la misma línea.
pub fn con_texto(ui: &mut Ui, name: &str, texto: impl Into<egui::RichText>, tint: Color32) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 3.0;
        if let Some(img) = image(name, 12.0, tint) {
            ui.add(img);
        }
        ui.label(texto.into());
    });
}

/// Icon for a rail tab.
pub fn tab_name(t: crate::app::Tab) -> &'static str {
    use crate::app::Tab::*;
    match t {
        Layers => "tab_hierarchy",
        Elements => "tab_elements",
        Templates => "tab_templates",
        Text => "tab_text",
        Images => "tab_images",
        Plugins => "tab_plugins",
        Comunidad => "tab_comunidad",
    }
}

/// Icon for an element kind.
pub fn kind_name(k: crate::model::Kind) -> &'static str {
    use crate::model::Kind::*;
    match k {
        Panel => "panel",
        Ellipse => "ellipse",
        Image => "image",
        Text => "text",
        Button => "button",
        Toggle => "toggle",
        Slider => "slider",
        InputField => "input",
        Dropdown => "dropdown",
        ScrollView => "scrollview",
        Empty => "empty",
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn icon_set_is_valid_svg() {
        for (name, bytes) in super::set().iter() {
            let s = std::str::from_utf8(bytes).unwrap();
            assert!(s.trim_start().starts_with("<svg") && s.contains("viewBox"), "{}", name);
            assert!(usvg_ok(bytes), "{} no se puede leer", name);
        }
    }

    fn usvg_ok(b: &[u8]) -> bool {
        egui_extras::image::load_svg_bytes(b, &Default::default()).is_ok()
    }
}
