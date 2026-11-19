//! uGUI export: a flat JSON (parent = index in the list, never null so Unity's
//! JsonUtility can read it) consumed by `EditorGuiImporter.cs`.

use super::{class_name, element_sprite, json_escape_invisible, radius_of, unity_image_ref, unity_names, view, ExportCtx, ImageNames};
use crate::docext::{ElementOverride, Orientation};
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
    let fields = view::field_names(doc);
    let ctx = ExportCtx::new(doc);
    // pictures with the same file name from different folders get different files (only needed for the project copies)
    let image_names = if for_project { Some(ImageNames::new(doc)) } else { None };
    let elements: Vec<_> = doc
        .elements
        .iter()
        .map(|e| {
            let rt = ctx.rt(e);
            let parent = ctx.parent(e).and_then(|p| index.get(&p.id)).map(|i| *i as i64).unwrap_or(-1);
            let source = if e.image.is_empty() {
                String::new()
            } else if let Some((asset, _)) = unity_image_ref(&e.image) {
                // already in the Unity project: referenced, never copied
                asset
            } else if let Some(names) = &image_names {
                names.asset_path(&e.image)
            } else {
                crate::library::resolve_image(&e.image).to_string_lossy().replace('\\', "/")
            };
            let pad = crate::render::text_padding(e);
            let valign = if e.kind == Kind::Text { 0 } else { e.valign.min(2) };
            let mut j = json!({
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
                    "preserveAspect": e.preserve_aspect,
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
                "contentHeight": if e.kind == Kind::ScrollView { r2(ctx.content_height(e)) } else { 0.0 },
                "onClick": e.on_click,
            });
            // Optional data (never null: JsonUtility cannot read it); the Unity side reads what it knows.
            if let Some(o) = j.as_object_mut() {
                if let Some(u) = e.unity_name.as_deref().filter(|u| crate::docext::valid_unity_name(u)) {
                    o.insert("unityName".into(), json!(u));
                }
                if let Some(sl) = e.slice {
                    o.insert("slice".into(), json!(sl.map(r2)));
                }
                if let Some(p) = e.ppu {
                    o.insert("ppu".into(), json!(r2(p)));
                }
                if let Some(sp) = element_sprite(e) {
                    o.insert("sprite".into(), json!(sp));
                }
                if let Some(fl) = fields.get(&e.id) {
                    o.insert("field".into(), json!(fl));
                }
                if e.image_mode.is_some() || e.slice.is_some() {
                    o.insert("imageMode".into(), json!(e.effective_image_mode().as_str()));
                }
            }
            j
        })
        .collect();
    let mut v = json!({
        "format": "editor-gui/ugui/2",
        "canvas": {
            "name": doc.title,
            "className": class_name(&doc.title),
            "referenceResolution": [doc.width, doc.height],
            "match": r2(doc.canvas_match),
            "pixelArt": doc.pixel_art,
        },
        "elements": elements,
    });
    // A fully transparent page has no canvas background at all (the importer then creates no background image).
    if doc.has_background() {
        v["canvas"]["background"] = json!(hex(doc.background));
    }
    doc_extras(doc, &mut v);
    // bidi / zero-width controls in the texts are written as an escape: same values, nothing invisible in the file
    json_escape_invisible(&serde_json::to_string_pretty(&v).unwrap_or_default())
}

fn layout_json(l: &crate::model::LayoutGroup) -> serde_json::Value {
    json!({
        "mode": l.mode.index(),
        "padding": l.padding.iter().map(|x| r2(*x)).collect::<Vec<_>>(),
        "spacing": [r2(l.spacing[0]), r2(l.spacing[1])],
        "cellSize": [r2(l.cell[0]), r2(l.cell[1])],
        "childAlignment": l.align.min(8),
        "expandWidth": l.expand_w,
        "expandHeight": l.expand_h,
    })
}

/// RectTransform of an element in the portrait layout, or `None` when the override does not move / resize it.
/// Computed on the design resolved for portrait (page size, overrides applied), against the parent's portrait box.
fn portrait_rect(pd: &Doc, pctx: &ExportCtx, ov: &ElementOverride) -> Option<serde_json::Value> {
    let geometry = ov.x.is_some() || ov.y.is_some() || ov.w.is_some() || ov.h.is_some() || ov.rot.is_some() || ov.anchor_min.is_some() || ov.anchor_max.is_some() || ov.pivot.is_some();
    if !geometry {
        return None;
    }
    let e = pctx.get(ov.id)?;
    let rt = pctx.rt(e);
    let _ = pd;
    Some(json!({
        "anchorMin": [r2(rt.anchor_min[0]), r2(rt.anchor_min[1])],
        "anchorMax": [r2(rt.anchor_max[0]), r2(rt.anchor_max[1])],
        "pivot": [r2(rt.pivot[0]), r2(rt.pivot[1])],
        "anchoredPosition": [r2(rt.anchored_position[0]), r2(rt.anchored_position[1])],
        "sizeDelta": [r2(rt.size_delta[0]), r2(rt.size_delta[1])],
        "rotationZ": r2(rt.rotation_z),
    }))
}

/// Canvas / unity / portrait data, written only when the design uses it. The canvas options go inside
/// the existing `canvas` object (`renderMode`, `sortOrder`, `safeArea`, `embed`, `planeDistance`).
fn doc_extras(doc: &Doc, v: &mut serde_json::Value) {
    let c = &doc.canvas;
    if !c.is_default() {
        if let Some(o) = v["canvas"].as_object_mut() {
            o.insert("renderMode".into(), json!(c.render_mode.as_str()));
            o.insert("sortOrder".into(), json!(c.sort_order));
            o.insert("safeArea".into(), json!(c.safe_area));
            o.insert("embed".into(), json!(c.embed));
            o.insert("planeDistance".into(), json!(r2(c.plane_distance)));
        }
    }
    if !doc.unity.is_default() {
        let u = &doc.unity;
        let mut o = serde_json::Map::new();
        if let Some(s) = &u.scripts_dir {
            o.insert("scriptsDir".into(), json!(s));
        }
        if let Some(s) = &u.assembly {
            o.insert("assembly".into(), json!(s));
        }
        o.insert(
            "generate".into(),
            json!({ "prefab": u.generate.prefab, "uitk": u.generate.uitk, "imgui": u.generate.imgui_enabled(), "viewScripts": u.generate.view_scripts }),
        );
        if let Some(s) = &u.default_font {
            o.insert("defaultFont".into(), json!(s));
        }
        if !u.fallback_fonts.is_empty() {
            o.insert("fallbackFonts".into(), json!(u.fallback_fonts));
        }
        v["unity"] = serde_json::Value::Object(o);
    }
    if let Some(p) = &doc.portrait {
        // the portrait design is resolved once (not once per override)
        let pd = doc.resolved_for(Orientation::Portrait);
        let pctx = ExportCtx::new(&pd);
        let overrides: Vec<_> = p
            .overrides
            .iter()
            .map(|ov| {
                let mut o = serde_json::Map::new();
                o.insert("id".into(), json!(ov.id));
                for (k, val) in [("x", ov.x), ("y", ov.y), ("w", ov.w), ("h", ov.h), ("rot", ov.rot), ("fontSize", ov.font_size)] {
                    if let Some(val) = val {
                        o.insert(k.into(), json!(r2(val)));
                    }
                }
                for (k, val) in [("anchorMin", ov.anchor_min), ("anchorMax", ov.anchor_max), ("pivot", ov.pivot)] {
                    if let Some(val) = val {
                        o.insert(k.into(), json!([r2(val[0]), r2(val[1])]));
                    }
                }
                if let Some(vis) = ov.visible {
                    o.insert("visible".into(), json!(vis));
                }
                if let Some(l) = &ov.layout {
                    o.insert("layout".into(), layout_json(l));
                }
                // Real RectTransform values of the portrait state (what Unity needs), next to the raw ones above.
                if let Some(rect) = portrait_rect(&pd, &pctx, ov) {
                    o.insert("rect".into(), rect);
                    o.insert("hasRect".into(), json!(true));
                }
                o.insert("hasVisible".into(), json!(ov.visible.is_some()));
                o.insert("hasFontSize".into(), json!(ov.font_size.is_some()));
                o.insert("hasLayout".into(), json!(ov.layout.is_some()));
                serde_json::Value::Object(o)
            })
            .collect();
        // A ScrollView's content height depends on the boxes of its children: portrait needs its own.
        let heights: Vec<_> = pd
            .elements
            .iter()
            .filter(|e| e.kind == Kind::ScrollView)
            .filter_map(|e| {
                let ph = pctx.content_height(e);
                let lh = doc.get(e.id).map(|l| super::content_height(doc, l)).unwrap_or(ph);
                if (ph - lh).abs() > 0.5 { Some(json!({ "id": e.id, "contentHeight": r2(ph) })) } else { None }
            })
            .collect();
        v["portrait"] = json!({ "width": r2(p.width), "height": r2(p.height), "overrides": overrides });
        if !heights.is_empty() {
            v["portrait"]["contentHeights"] = json!(heights);
        }
    }
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
