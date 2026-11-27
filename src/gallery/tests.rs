use super::*;
use crate::review::{contrast, review, Level};

fn over(up: Rgba, down: Rgba) -> Rgba {
    let a = up[3] as f32 / 255.0;
    let m = |i: usize| (up[i] as f32 * a + down[i] as f32 * (1.0 - a)).round() as u8;
    [m(0), m(1), m(2), 255]
}

#[test]
fn every_type_has_three_templates_in_three_styles() {
    let mut seen = std::collections::HashSet::new();
    for s in SECTIONS {
        for ty in s.types {
            assert!(seen.insert(*ty), "type «{}» is listed twice", ty);
            let idx = of_type(ty);
            assert_eq!(idx.len(), 3, "«{}» has {} templates", ty, idx.len());
            let styles: std::collections::HashSet<_> = idx.iter().map(|&i| entry(i).style).collect();
            assert_eq!(styles.len(), 3, "«{}» repeats a style", ty);
        }
    }
    for i in 0..count() {
        assert!(seen.contains(entry(i).ty), "«{}» is not in SECTIONS", entry(i).ty);
    }
    let mut names = std::collections::HashSet::new();
    for n in crate::model::TEMPLATES.iter() {
        assert!(names.insert(*n), "duplicate template name «{}»", n);
    }
}

#[test]
fn styles_keep_their_contrast_pairs() {
    for st in STYLES {
        let t = st.theme();
        for (what, fg, bg, min) in [
            ("text/panel", t.text, over(t.panel, t.bg), 4.5),
            ("muted/panel", t.muted, over(t.panel, t.bg), 4.5),
            ("text/panel2", t.text, over(t.panel2, t.bg), 4.5),
            ("muted/panel2", t.muted, over(t.panel2, t.bg), 4.5),
            ("on_accent/accent", t.on_accent, t.accent, 4.5),
            ("bg_text/bg", t.bg_text, t.bg, 4.5),
            ("bg_text/bg2", t.bg_text, t.bg2, 4.5),
            ("text/accent-free key cap", t.panel2, t.text, 4.5),
        ] {
            let c = contrast(fg, bg);
            assert!(c >= min, "{}: {} contrast {:.2} < {}", st.name(), what, c, min);
        }
    }
}

/// Every gallery template, in both orientations: no review errors, no text-contrast warnings, every child inside its
/// parent (a layout that overflows its box looks broken in Unity).
#[test]
fn gallery_templates_are_clean() {
    let base = crate::model::BUILTIN_TEMPLATES.len();
    let mut failures = Vec::new();
    for i in 0..count() {
        let d = crate::model::template_with_portrait(base + i, 1920.0, 1080.0);
        for p in review(&d) {
            let bad = p.level == Level::Error || p.template.starts_with("Text contrast");
            if bad && !p.data.iter().any(|a| a.starts_with("pack_")) {
                failures.push(format!("{} · {}: {} {:?}", name(i), p.element, p.template, p.data));
            }
        }
        // the portrait layout stored with the template shows the portrait build of the template
        let shown = d.resolved_for(crate::docext::Orientation::Portrait);
        let built = crate::model::template(base + i, 1080.0, 1920.0);
        for (a, b) in shown.elements.iter().zip(built.elements.iter()) {
            if (a.x - b.x).abs() >= 1.0 || (a.y - b.y).abs() >= 1.0 || (a.w - b.w).abs() >= 1.0 || (a.h - b.h).abs() >= 1.0 {
                failures.push(format!("{} (portrait view): «{}» {:?} vs {:?}", name(i), a.name, [a.x, a.y, a.w, a.h], [b.x, b.y, b.w, b.h]));
            }
        }
        for (w, h) in [(1920.0, 1080.0), (1080.0, 1920.0), (1280.0, 720.0), (800.0, 600.0), (2560.0, 1440.0)] {
            let d = crate::model::template(base + i, w, h);
            for e in &d.elements {
                if e.x < -1.0 || e.y < -1.0 || e.x + e.w > w + 1.0 || e.y + e.h > h + 1.0 {
                    failures.push(format!("{} @{}x{}: «{}» outside the page {:?}", name(i), w, h, e.name, [e.x, e.y, e.w, e.h]));
                }
            }
            assert!(d.elements.len() >= 6, "{}: only {} elements", name(i), d.elements.len());
            for e in &d.elements {
                let Some(p) = e.parent.and_then(|p| d.get(p)) else { continue };
                let tol = 2.0;
                if e.x < p.x - tol || e.y < p.y - tol || e.x + e.w > p.x + p.w + tol || e.y + e.h > p.y + p.h + tol {
                    failures.push(format!("{} @{}x{}: «{}» goes out of «{}» {:?} vs {:?}", name(i), w, h, e.name, p.name, [e.x, e.y, e.w, e.h], [p.x, p.y, p.w, p.h]));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{} problems:\n{}", failures.len(), failures.join("\n"));
}

/// `GALLERY_DUMP=<file.js> cargo test dump_gallery -- --ignored`: writes every gallery template (landscape and
/// portrait) as `window.GALLERY = [...]` for the contact-sheet preview (tools that render the JSON in a browser).
#[test]
#[ignore]
fn dump_gallery() {
    let out = std::env::var("GALLERY_DUMP").expect("GALLERY_DUMP");
    let filter = std::env::var("GALLERY_FILTER").unwrap_or_default().to_lowercase();
    let base = crate::model::BUILTIN_TEMPLATES.len();
    let mut items = Vec::new();
    for i in 0..count() {
        if !filter.is_empty() && !name(i).to_lowercase().contains(&filter) && !SECTIONS.iter().any(|s| s.types.contains(&entry(i).ty) && s.group.to_lowercase().contains(&filter)) {
            continue;
        }
        for (w, h) in [(1920.0, 1080.0), (1080.0, 1920.0)] {
            let d = crate::model::template(base + i, w, h);
            items.push(serde_json::json!({ "index": base + i, "name": name(i), "doc": d }));
        }
    }
    let pack: std::collections::BTreeMap<String, String> = crate::pack::items().iter().map(|i| (crate::pack::library_name(i), i.file.clone())).collect();
    std::fs::write(&out, format!("window.GALLERY = {};
window.PACK = {};", serde_json::to_string(&items).unwrap(), serde_json::to_string(&pack).unwrap())).unwrap();
}

/// Every gallery template, at 1920 × 1080: text at least the review's minimum size and every control the player taps
/// at least the recommended touch size (no "hard to read" / "hard to tap" warnings).
#[test]
fn gallery_templates_meet_text_and_touch_minimums() {
    let base = crate::model::BUILTIN_TEMPLATES.len();
    let mut failures = Vec::new();
    for i in 0..count() {
        let d = crate::model::template(base + i, 1920.0, 1080.0);
        for p in review(&d) {
            if p.template.contains("hard to read") || p.template.contains("hard to tap") {
                failures.push(format!("{} · {}: {:?}", name(i), p.element, p.data));
            }
        }
    }
    assert!(failures.is_empty(), "{} problems:
{}", failures.len(), failures.join("
"));
}
