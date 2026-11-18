//! uGUI export: a flat JSON (parent = index in the list, never null so Unity's
//! JsonUtility can read it) consumed by `EditorGuiImporter.cs`.

use super::{class_name, content_height, export_rt, image_asset_path, radius_of, unity_names};
use crate::model::{Doc, Kind, hex};
use serde_json::json;
use std::collections::HashMap;

fn r2(v: f32) -> f64 {
    ((v as f64) * 100.0).round() / 100.0
}

/// `for_project`: image sources point inside the Unity project (Assets/EditorGUI/Sprites)
/// instead of the files on this computer.
pub fn json(doc: &Doc, for_project: bool) -> String {
    let names = unity_names(doc);
    let index: HashMap<u64, usize> = doc.elements.iter().enumerate().map(|(i, e)| (e.id, i)).collect();
    let elements: Vec<_> = doc
        .elements
        .iter()
        .map(|e| {
            let rt = export_rt(doc, e);
            let parent = doc.parent_of(e.id).and_then(|p| index.get(&p)).map(|i| *i as i64).unwrap_or(-1);
            let source = if e.image.is_empty() {
                String::new()
            } else if for_project {
                image_asset_path(&e.image)
            } else {
                crate::library::resolve_image(&e.image).to_string_lossy().replace('\\', "/")
            };
            let pad = crate::render::text_padding(e);
            let valign = if e.kind == Kind::Text { 0 } else { e.valign.min(2) };
            json!({
                "id": e.id,
                "parent": parent,
                "name": names.get(&e.id).cloned().unwrap_or_default(),
                "type": e.kind.unity_type(),
                "shape": if e.kind == Kind::Ellipse { "ellipse" } else { "rect" },
                "active": e.visible,
                "interactable": e.interactable,
                "raycastTarget": e.raycast,
                "mask": e.mask,
                "size": [r2(e.w), r2(e.h)],
                "rect": {
                    "anchorMin": [r2(rt.anchor_min[0]), r2(rt.anchor_min[1])],
                    "anchorMax": [r2(rt.anchor_max[0]), r2(rt.anchor_max[1])],
                    "pivot": [r2(rt.pivot[0]), r2(rt.pivot[1])],
                    "anchoredPosition": [r2(rt.anchored_position[0]), r2(rt.anchored_position[1])],
                    "sizeDelta": [r2(rt.size_delta[0]), r2(rt.size_delta[1])],
                    "rotationZ": r2(rt.rotation_z),
                },
                "image": {
                    "color": hex(e.fill),
                    "gradientTo": hex(e.fill2),
                    "useGradient": e.kind == Kind::Image && e.image.is_empty(),
                    "source": source,
                    "radius": r2(radius_of(e)),
                    "alpha": r2(e.opacity),
                    "preserveAspect": false,
                },
                "outline": { "color": hex(e.stroke), "width": r2(if e.kind.has_fill() { e.stroke_w } else { 0.0 }) },
                "shadow": { "enabled": e.shadow.on, "color": hex(e.shadow.color), "offset": [r2(e.shadow.offset[0]), r2(e.shadow.offset[1])] },
                "text": {
                    "hasText": e.kind.has_text() && !e.caption().is_empty(),
                    "value": e.text,
                    "fontSize": r2(e.font_size),
                    "color": hex(e.text_color),
                    "bold": e.bold,
                    "italic": e.italic,
                    "underline": e.underline,
                    "alignH": e.align.min(2),
                    "alignV": valign,
                    "font": e.font,
                    "padding": [r2(pad.0), r2(pad.1), r2(pad.2)],
                },
                "layout": {
                    "mode": e.layout.mode.index(),
                    "padding": e.layout.padding.iter().map(|v| r2(*v)).collect::<Vec<_>>(),
                    "spacing": [r2(e.layout.spacing[0]), r2(e.layout.spacing[1])],
                    "cellSize": [r2(e.layout.cell[0]), r2(e.layout.cell[1])],
                    "childAlignment": e.layout.align.min(8),
                    "expandWidth": e.layout.expand_w,
                    "expandHeight": e.layout.expand_h,
                },
                "value": r2(e.value),
                "options": e.options,
                "contentHeight": if e.kind == Kind::ScrollView { r2(content_height(doc, e)) } else { 0.0 },
                "onClick": e.on_click,
            })
        })
        .collect();
    let v = json!({
        "format": "editor-gui/ugui/2",
        "canvas": {
            "name": doc.title,
            "className": class_name(&doc.title),
            "referenceResolution": [doc.width, doc.height],
            "match": r2(doc.canvas_match),
            "pixelArt": doc.pixel_art,
            "background": hex(doc.background),
        },
        "elements": elements,
    });
    serde_json::to_string_pretty(&v).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model;

    #[test]
    fn flat_list_with_parent_indices_and_no_nulls() {
        let d = model::template(2, 1920.0, 1080.0);
        let s = json(&d, true);
        assert!(!s.contains("null"), "JsonUtility can't read nulls");
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        let els = v["elements"].as_array().unwrap();
        let win = els.iter().position(|e| e["name"] == "Window").unwrap();
        let title = els.iter().find(|e| e["name"] == "Window title").unwrap();
        assert_eq!(title["parent"].as_i64().unwrap(), win as i64);
        // parents always come before their children
        for (i, e) in els.iter().enumerate() {
            assert!(e["parent"].as_i64().unwrap() < i as i64);
        }
        assert_eq!(v["canvas"]["className"], "DialogWindow");
    }
}
