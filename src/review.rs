//! «Review design» (design review): quick readability and usability checks before exporting to Unity.
//! Everything is pure (no UI) so it can be used from the window, the command line (`--review`) and the MCP.

use crate::model::{Doc, Element, Kind, Rgba};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Warning,
    Error,
}

#[derive(Clone, Debug)]
pub struct Problem {
    pub level: Level,
    pub id: u64,
    pub element: String,
    /// Text in Spanish (translated when displayed with `i18n::tf`).
    pub template: &'static str,
    pub data: Vec<String>,
}

/// Minimum legible font size at 1080 p height (scaled with the reference resolution).
const MIN_FONT: f32 = 18.0;
/// Minimum side of a tappable control at 1080 p (≈ 9 mm on a 6" phone).
const MIN_TOUCH: f32 = 64.0;

fn luminance(c: Rgba) -> f32 {
    let f = |v: u8| {
        let s = v as f32 / 255.0;
        if s <= 0.04045 { s / 12.92 } else { ((s + 0.055) / 1.055).powf(2.4) }
    };
    0.2126 * f(c[0]) + 0.7152 * f(c[1]) + 0.0722 * f(c[2])
}

/// WCAG contrast ratio (1 to 21).
pub fn contrast(a: Rgba, b: Rgba) -> f32 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

/// `v` written with as few decimals as it takes not to look equal to the threshold `min` it is below
/// (17.6 px against 18 px reads «17.6», never a rounded «18»).
fn below(v: f32, min: f32) -> String {
    for d in 0..=3usize {
        let (s, m) = (format!("{:.*}", d, v), format!("{:.*}", d, min));
        if s != m {
            return s;
        }
    }
    format!("{}", v)
}

/// Contrast ratio written rounded down to one decimal, so that a failing 4.46 is not shown as «4.5».
fn floor1(v: f32) -> String {
    format!("{:.1}", (v * 10.0).floor() / 10.0)
}

/// Colour of an element's fill as it is seen: its alpha times the element's own opacity.
fn seen(c: Rgba, opacity: f32) -> Rgba {
    let mut c = c;
    c[3] = (c[3] as f32 * opacity.clamp(0.0, 1.0)).round() as u8;
    c
}

fn blend(up: Rgba, down: Rgba) -> Rgba {
    let a = up[3] as f32 / 255.0;
    let m = |i: usize| (up[i] as f32 * a + down[i] as f32 * (1.0 - a)).round() as u8;
    [m(0), m(1), m(2), 255]
}

fn fill(e: &Element) -> Rgba {
    // Images without a file are gradients: the mid color is used.
    if e.kind == Kind::Image && e.image.is_empty() {
        let m = |i: usize| ((e.fill[i] as u16 + e.fill2[i] as u16) / 2) as u8;
        return [m(0), m(1), m(2), m(3)];
    }
    e.fill
}

/// For every element (by position): visible itself and with every ancestor visible and not fully transparent.
/// Iterative, O(n) (a parent chain of any depth is fine).
fn visible_all(doc: &Doc) -> Vec<bool> {
    let n = doc.elements.len();
    let idx = doc.index_map();
    let par = doc.parent_indices(&idx);
    let mut state = vec![0u8; n]; // 0 new, 1 on the current chain, 2 resolved
    let mut vis = vec![true; n];
    let mut chain: Vec<usize> = Vec::new();
    for s in 0..n {
        if state[s] == 2 {
            continue;
        }
        chain.clear();
        let mut cur = Some(s);
        let mut base = true;
        while let Some(c) = cur {
            if state[c] == 2 {
                base = vis[c];
                break;
            }
            if state[c] == 1 {
                break; // cycle in a malformed file
            }
            state[c] = 1;
            chain.push(c);
            cur = par[c];
        }
        for &c in chain.iter().rev() {
            let e = &doc.elements[c];
            base = base && e.visible && e.opacity > 0.01;
            vis[c] = base;
            state[c] = 2;
        }
    }
    vis
}

fn has_own_background(e: &Element) -> bool {
    matches!(e.kind, Kind::Button | Kind::InputField | Kind::Dropdown | Kind::Panel | Kind::Ellipse) && seen(e.fill, e.opacity)[3] > 0
}

/// Color the text of `idx` is seen against: its own fill if it has one, or whatever is underneath.
fn background_of(doc: &Doc, vis: &[bool], idx: usize) -> Rgba {
    let e = &doc.elements[idx];
    let (cx, cy) = (e.x + e.w / 2.0, e.y + e.h / 2.0);
    let mut color = doc.background;
    color[3] = 255;
    for (i, o) in doc.elements.iter().enumerate() {
        if i >= idx && !(i == idx && has_own_background(e)) {
            break;
        }
        if o.kind == Kind::Text || o.kind == Kind::Empty || !o.image.is_empty() || !vis[i] {
            continue;
        }
        if cx >= o.x && cx <= o.x + o.w && cy >= o.y && cy <= o.y + o.h {
            color = blend(seen(fill(o), o.opacity), color);
        }
    }
    color
}

fn is_pressable(e: &Element) -> bool {
    e.interactable && matches!(e.kind, Kind::Button | Kind::Toggle | Kind::InputField | Kind::Dropdown)
}

/// Reviews the design and returns the problems, errors first. A design with a portrait layout is checked in both
/// orientations (the portrait ones are marked with "(portrait)" after the element name).
pub fn review(doc: &Doc) -> Vec<Problem> {
    let mut out = review_layout(doc, false);
    if doc.portrait.is_some() {
        let resolved = doc.resolved_for(crate::docext::Orientation::Portrait);
        out.extend(review_layout(&resolved, true));
    }
    out.sort_by(|a, b| b.level.cmp(&a.level));
    out
}

/// The checks of one layout. `portrait`: the resolved portrait design (checks that do not depend on the layout - names,
/// missing files, fonts - are only run on the landscape one).
fn review_layout(doc: &Doc, portrait: bool) -> Vec<Problem> {
    let scale = (doc.height.min(doc.width) / 1080.0).max(0.1);
    let mut out = Vec::new();
    let mut names: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for e in &doc.elements {
        *names.entry(e.name.as_str()).or_default() += 1;
    }
    let vis = visible_all(doc);
    let mut image_ok: std::collections::HashMap<&str, bool> = std::collections::HashMap::new();
    for (idx, e) in doc.elements.iter().enumerate() {
        let mut p = |level, template, data: Vec<String>| out.push(Problem { level, id: e.id, element: e.name.clone(), template, data });
        if !vis[idx] {
            continue;
        }
        let with_text = !e.text.trim().is_empty() && matches!(e.kind, Kind::Text | Kind::Button | Kind::Panel | Kind::Ellipse | Kind::Toggle);
        if with_text {
            let min = MIN_FONT * scale;
            if e.font_size < min {
                p(Level::Warning, "{} px text: hard to read on small screens (recommended minimum {} px)", vec![below(e.font_size, min), format!("{:.0}", min)]);
            }
            // WCAG does not require contrast for disabled controls.
            let disabled = !e.interactable && matches!(e.kind, Kind::Button | Kind::Toggle);
            let background = background_of(doc, &vis, idx);
            let c = contrast(blend(seen(e.text_color, e.opacity), background), background);
            // WCAG "large text": 18 pt (24 px) or 14 pt bold (≈18.7 px), at the reference scale.
            let big = e.font_size >= 24.0 * scale || (e.bold && e.font_size >= 18.7 * scale);
            let min_c = if big { 3.0 } else { 4.5 };
            if c < min_c && !disabled {
                let level = if c < 2.0 { Level::Error } else { Level::Warning };
                p(level, "Text contrast {}:1 (WCAG asks for {}:1)", vec![floor1(c), format!("{:.1}", min_c)]);
            }
        }
        if is_pressable(e) {
            let min = MIN_TOUCH * scale;
            if e.w.min(e.h) < min {
                p(Level::Warning, "{} px control: hard to tap with a finger (recommended minimum {} px)", vec![below(e.w.min(e.h), min), format!("{:.0}", min)]);
            }
        }
        let outside = e.x + e.w < 0.0 || e.y + e.h < 0.0 || e.x > doc.width || e.y > doc.height;
        if outside && e.rot.abs() < 0.01 {
            p(Level::Error, "Completely outside the page: it will not be visible in the game", vec![]);
        } else if e.parent.is_none() && (e.x < -1.0 || e.y < -1.0 || e.x + e.w > doc.width + 1.0 || e.y + e.h > doc.height + 1.0) && e.rot.abs() < 0.01 {
            p(Level::Warning, "Partly outside the page", vec![]);
        }
        // Checks that do not depend on the layout (names, files) are made once, on the landscape design.
        if !portrait && is_pressable(e) && names.get(e.name.as_str()).copied().unwrap_or(0) > 1 {
            p(Level::Warning, "Several elements share this name: GameObject.Find and scripts will only find the first one", vec![]);
        }
        if !portrait && e.kind == Kind::Image && !e.image.is_empty() && !*image_ok.entry(e.image.as_str()).or_insert_with(|| crate::library::resolve_image(&e.image).exists()) {
            p(Level::Error, "Missing image “{}”: it will be exported empty", vec![e.image.clone()]);
        }
    }
    if portrait {
        for p in out.iter_mut() {
            p.element = crate::i18n::tf("{} (portrait)", &[&p.element]);
        }
    } else {
        unity_name_problems(doc, &mut out);
        font_problems(doc, &vis, &mut out, &font_cmap_by_name);
    }
    out
}

// ---------------------------------------------------------------- fonts: characters the font lacks

/// The `cmap` table of a font, enough to ask "does this font draw this character?".
pub struct Cmap {
    data: Vec<u8>,
    /// Offsets (inside `data`) of the Unicode subtables of format 4 and 12.
    subs: Vec<usize>,
}

fn be16(d: &[u8], i: usize) -> Option<u16> {
    Some(u16::from_be_bytes(d.get(i..i.checked_add(2)?)?.try_into().ok()?))
}

fn be32(d: &[u8], i: usize) -> Option<u32> {
    Some(u32::from_be_bytes(d.get(i..i.checked_add(4)?)?.try_into().ok()?))
}

/// Largest cmap table accepted (real fonts have well under 1 MB).
const MAX_CMAP: usize = 8 << 20;

/// Extracts the raw `cmap` table from a TrueType / OpenType file (the first font of a collection).
pub fn extract_cmap_table(font: &[u8]) -> Option<Vec<u8>> {
    let mut base = 0usize;
    if font.get(0..4)? == b"ttcf" {
        base = be32(font, 12)? as usize;
    }
    let tag = font.get(base..base.checked_add(4)?)?;
    if !(tag == [0, 1, 0, 0] || tag == b"true" || tag == b"OTTO") {
        return None;
    }
    let n = be16(font, base + 4)? as usize;
    for i in 0..n.min(256) {
        let rec = base + 12 + i * 16;
        if font.get(rec..rec + 4)? == b"cmap" {
            let off = be32(font, rec + 8)? as usize;
            let len = be32(font, rec + 12)? as usize;
            if len > MAX_CMAP {
                return None;
            }
            return font.get(off..off.checked_add(len)?).map(|s| s.to_vec());
        }
    }
    None
}

impl Cmap {
    /// None when the table has no Unicode subtable this reader understands (formats 4 and 12).
    pub fn parse(table: Vec<u8>) -> Option<Cmap> {
        let n = be16(&table, 2)? as usize;
        let mut subs = Vec::new();
        for i in 0..n.min(64) {
            let rec = 4 + i * 8;
            let (platform, encoding) = (be16(&table, rec)?, be16(&table, rec + 2)?);
            let off = be32(&table, rec + 4)? as usize;
            let unicode = platform == 0 || (platform == 3 && (encoding == 1 || encoding == 10));
            if unicode && matches!(be16(&table, off), Some(4) | Some(12)) {
                subs.push(off);
            }
        }
        if subs.is_empty() { None } else { Some(Cmap { data: table, subs }) }
    }

    pub fn from_font(font: &[u8]) -> Option<Cmap> {
        Cmap::parse(extract_cmap_table(font)?)
    }

    /// True when the font has a glyph for `c`.
    pub fn has(&self, c: char) -> bool {
        self.subs.iter().any(|&s| self.lookup(s, c as u32).unwrap_or(false))
    }

    fn lookup(&self, s: usize, c: u32) -> Option<bool> {
        let d = &self.data;
        match be16(d, s)? {
            4 => {
                if c > 0xFFFF {
                    return Some(false);
                }
                let seg_x2 = be16(d, s + 6)? as usize;
                let (ends, starts, deltas, ranges) = (s + 14, s + 16 + seg_x2, s + 16 + 2 * seg_x2, s + 16 + 3 * seg_x2);
                for k in 0..seg_x2 / 2 {
                    let end = be16(d, ends + 2 * k)? as u32;
                    if end < c {
                        continue;
                    }
                    let start = be16(d, starts + 2 * k)? as u32;
                    if start > c {
                        return Some(false);
                    }
                    let delta = be16(d, deltas + 2 * k)?;
                    let ro = be16(d, ranges + 2 * k)? as usize;
                    let gid = if ro == 0 {
                        (c as u16).wrapping_add(delta)
                    } else {
                        let g = be16(d, ranges + 2 * k + ro + 2 * (c - start) as usize)?;
                        if g == 0 { 0 } else { g.wrapping_add(delta) }
                    };
                    return Some(gid != 0);
                }
                Some(false)
            }
            12 => {
                let groups = (be32(d, s + 12)? as usize).min(1 << 20);
                let (mut lo, mut hi) = (0usize, groups);
                while lo < hi {
                    let mid = (lo + hi) / 2;
                    let g = s + 16 + mid * 12;
                    let (start, end) = (be32(d, g)?, be32(d, g + 4)?);
                    if c < start {
                        hi = mid;
                    } else if c > end {
                        lo = mid + 1;
                    } else {
                        return Some(be32(d, g + 8)?.wrapping_add(c - start) != 0);
                    }
                }
                Some(false)
            }
            _ => None,
        }
    }
}

/// Characters of `text` the font has no glyph for (printable ones only, first occurrence, in order).
pub fn missing_chars(cmap: &Cmap, text: &str) -> Vec<char> {
    let mut out: Vec<char> = Vec::new();
    for c in text.chars() {
        if c.is_control() || c.is_whitespace() || out.contains(&c) {
            continue;
        }
        if !cmap.has(c) {
            out.push(c);
        }
        if out.len() >= 64 {
            break;
        }
    }
    out
}

/// Cmap of a font known to the editor (system or library), cached per file. None when the font is unknown or unreadable.
fn font_cmap_by_name(name: &str) -> Option<std::sync::Arc<Cmap>> {
    static CACHE: std::sync::Mutex<Option<std::collections::HashMap<std::path::PathBuf, Option<std::sync::Arc<Cmap>>>>> = std::sync::Mutex::new(None);
    let path = crate::render::font_path(name)?;
    let mut g = CACHE.lock().ok()?;
    let map = g.get_or_insert_with(Default::default);
    if let Some(c) = map.get(&path) {
        return c.clone();
    }
    let parsed = std::fs::metadata(&path)
        .ok()
        .filter(|m| m.len() <= 128 << 20)
        .and_then(|_| std::fs::read(&path).ok())
        .and_then(|b| Cmap::from_font(&b))
        .map(std::sync::Arc::new);
    map.insert(path, parsed.clone());
    parsed
}

/// Text that has a font of its own and uses characters that font lacks (« » — … often). Unknown fonts and the default are skipped.
fn font_problems(doc: &Doc, vis: &[bool], out: &mut Vec<Problem>, cmap_of: &dyn Fn(&str) -> Option<std::sync::Arc<Cmap>>) {
    for (i, e) in doc.elements.iter().enumerate() {
        if e.font.is_empty() || !e.kind.has_text() || !vis[i] {
            continue;
        }
        let mut text = e.text.clone();
        for o in &e.options {
            text.push('\n');
            text.push_str(o);
        }
        if text.trim().is_empty() {
            continue;
        }
        let Some(cmap) = cmap_of(&e.font) else { continue };
        let missing = missing_chars(&cmap, &text);
        if !missing.is_empty() {
            let list: Vec<String> = missing.iter().take(12).map(|c| c.to_string()).collect();
            out.push(Problem { level: Level::Warning, id: e.id, element: e.name.clone(), template: "Font {} lacks characters: {}", data: vec![e.font.clone(), list.join(" ")] });
        }
    }
}


/// Invalid explicit Unity names, and siblings that resolve to the same GameObject name.
fn unity_name_problems(doc: &Doc, out: &mut Vec<Problem>) {
    let mut seen: std::collections::HashMap<(Option<usize>, String), u64> = std::collections::HashMap::new();
    let par = doc.parent_indices(&doc.index_map());
    for (i, e) in doc.elements.iter().enumerate() {
        let mut p = |template, data: Vec<String>| out.push(Problem { level: Level::Warning, id: e.id, element: e.name.clone(), template, data });
        if let Some(u) = e.unity_name.as_deref() {
            if !crate::docext::valid_unity_name(u) {
                p("Invalid Unity name “{}”: it must not be empty, contain “/” or control characters, or exceed 128 characters (the display name is used instead)", vec![u.to_string()]);
            }
        }
        let name = crate::export::base_unity_name(e);
        if let Some(first) = seen.insert((par[i], name.clone()), e.id) {
            let _ = first;
            p("Two siblings resolve to the same Unity name “{}”: the exporter will rename this one", vec![name]);
        }
    }
}

/// Text of a problem in the current language.
pub fn describe(p: &Problem) -> String {
    let args: Vec<&str> = p.data.iter().map(|s| s.as_str()).collect();
    crate::i18n::tf(p.template, &args)
}

/// `--review`: prints the problems; exit code 1 if there are errors.
pub fn report(doc: &Doc) -> (String, bool) {
    let ps = review(doc);
    if ps.is_empty() {
        return (crate::i18n::t("No issues: the design passes every check."), false);
    }
    let errors = ps.iter().any(|p| p.level == Level::Error);
    let lines: Vec<String> = ps
        .iter()
        .map(|p| format!("[{}] {}: {}", if p.level == Level::Error { "ERROR" } else { "WARNING" }, p.element, describe(p)))
        .collect();
    (lines.join("\n"), errors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Doc, Element, Kind};

    fn doc_with(e: Element) -> Doc {
        let mut d = Doc::empty(1920.0, 1080.0);
        d.background = [255, 255, 255, 255];
        d.add(e);
        d
    }

    #[test]
    fn known_contrast() {
        assert!((contrast([0, 0, 0, 255], [255, 255, 255, 255]) - 21.0).abs() < 0.01);
        assert!((contrast([10, 20, 30, 255], [10, 20, 30, 255]) - 1.0).abs() < 0.001);
    }

    #[test]
    fn detects_illegible_text_and_small_button() {
        let mut t = Element::new(0, Kind::Text);
        t.text = "hola".into();
        t.text_color = [240, 240, 240, 255];
        t.font_size = 12.0;
        t.x = 100.0;
        t.y = 100.0;
        let ps = review(&doc_with(t));
        assert!(ps.iter().any(|p| p.template.starts_with("{} px text")));
        assert!(ps.iter().any(|p| p.template.starts_with("Text contrast") && p.level == Level::Error));

        let mut b = Element::new(0, Kind::Button);
        b.x = 100.0;
        b.y = 100.0;
        b.w = 40.0;
        b.h = 40.0;
        let ps = review(&doc_with(b));
        assert!(ps.iter().any(|p| p.template.starts_with("{} px control")));
    }

    #[test]
    fn rounded_messages_never_claim_equality_while_flagging() {
        // 17.6 px text against the 18 px minimum
        let mut t = Element::new(0, Kind::Text);
        t.text = "hola".into();
        t.text_color = [0, 0, 0, 255];
        t.font_size = 17.6;
        let ps = review(&doc_with(t));
        let p = ps.iter().find(|p| p.template.starts_with("{} px text")).unwrap();
        assert_eq!(p.data[0], "17.6");
        assert_eq!(p.data[1], "18");
        // 63.7 px button against 64 px
        let mut b = Element::new(0, Kind::Button);
        b.w = 63.7;
        b.h = 90.0;
        let ps = review(&doc_with(b));
        let p = ps.iter().find(|p| p.template.starts_with("{} px control")).unwrap();
        assert_ne!(p.data[0], p.data[1]);
        // contrast that rounds to the threshold shows the lower value
        assert_eq!(floor1(4.46), "4.4");
        assert_eq!(floor1(4.5), "4.5");
        assert_eq!(below(17.96, 18.0), "17.96");
        assert_eq!(below(12.0, 18.0), "12");
    }

    #[test]
    fn element_opacity_counts_for_the_contrast_background() {
        // dark text on a black panel that is almost transparent: it is read against the white page
        let mut d = Doc::empty(1920.0, 1080.0);
        d.background = [255, 255, 255, 255];
        let mut panel = Element::new(0, Kind::Panel);
        panel.fill = [0, 0, 0, 255];
        panel.opacity = 0.05;
        panel.x = 0.0;
        panel.y = 0.0;
        panel.w = 400.0;
        panel.h = 200.0;
        let pid = d.add(panel);
        let mut t = Element::new(0, Kind::Text);
        t.text = "hola".into();
        t.text_color = [0, 0, 0, 255];
        t.font_size = 30.0;
        t.w = 100.0;
        t.h = 40.0;
        t.x = 10.0;
        t.y = 10.0;
        t.parent = Some(pid);
        d.add(t);
        d.normalize();
        assert!(!review(&d).iter().any(|p| p.template.starts_with("Text contrast")), "{:?}", review(&d).iter().map(|p| p.template).collect::<Vec<_>>());
        // the same panel fully opaque is a real problem
        d.get_mut(pid).unwrap().opacity = 1.0;
        assert!(review(&d).iter().any(|p| p.template.starts_with("Text contrast")));
    }

    #[test]
    fn off_page_and_duplicate_names() {
        let mut b = Element::new(0, Kind::Button);
        b.x = 5000.0;
        let ps = review(&doc_with(b));
        assert!(ps.iter().any(|p| p.level == Level::Error && p.template.contains("outside")));
        let mut d = Doc::empty(1920.0, 1080.0);
        for _ in 0..2 {
            let mut b = Element::new(0, Kind::Button);
            b.name = "Play".into();
            b.x = 100.0;
            b.y = 100.0;
            d.add(b);
        }
        assert!(review(&d).iter().any(|p| p.template.contains("GameObject.Find")));
    }

    #[test]
    fn bundled_templates_without_errors() {
        for i in 0..crate::model::TEMPLATES.len() {
            let d = crate::model::template(i, 1920.0, 1080.0);
            let errors: Vec<String> = review(&d).iter().filter(|p| p.level == Level::Error).map(|p| format!("{}: {}", p.element, describe(p))).collect();
            assert!(errors.is_empty(), "{}: {:?}", crate::model::TEMPLATES[i], errors);
        }
    }
}

/// The last result of the review shown in the window (see `review_window`).
pub struct ReviewCache {
    key: (usize, usize, usize, bool),
    at: f64,
    problems: Vec<Problem>,
}

impl crate::app::EditorApp {
    /// «Review design» window: list of problems; clicking selects the element.
    pub fn review_window(&mut self, ctx: &eframe::egui::Context) {
        use crate::i18n::t;
        use eframe::egui::{self, Color32, RichText};
        if !self.review_open {
            return;
        }
        let now = ctx.input(|i| i.time);
        let key = (self.undo.len(), self.redo.len(), self.real_doc().elements.len(), self.orientation() == crate::docext::Orientation::Portrait);
        // The review is not recomputed every frame: only when the design changed (undo history) or every 2 seconds.
        let stale = match &self.review_cache {
            Some(c) => c.key != key || now - c.at > 2.0,
            None => true,
        };
        if stale {
            let problems = review(self.doc_real());
            self.review_cache = Some(ReviewCache { key, at: now, problems });
        }
        let problems = self.review_cache.as_ref().map(|c| c.problems.clone()).unwrap_or_default();
        let mut open = true;
        let mut choose = None;
        egui::Window::new(t("Review design")).open(&mut open).collapsible(false).default_width(460.0).show(ctx, |ui| {
            let errors = problems.iter().filter(|p| p.level == Level::Error).count();
            let warnings = problems.len() - errors;
            if problems.is_empty() {
                ui.label(RichText::new(t("No issues: the design passes every check.")).color(crate::theme::ACCENT()));
            } else {
                ui.label(crate::i18n::tf("Errors: {} · Warnings: {}. Click one to select it.", &[&errors.to_string(), &warnings.to_string()]));
            }
            ui.add_space(6.0);
            egui::ScrollArea::vertical().max_height(360.0).show(ui, |ui| {
                for p in &problems {
                    let (label, col) = match p.level {
                        Level::Error => (t("Error"), Color32::from_rgb(235, 80, 90)),
                        Level::Warning => (t("Warning"), Color32::from_rgb(230, 170, 60)),
                    };
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new(label).strong().color(col));
                        if ui.link(&p.element).clicked() {
                            choose = Some(p.id);
                        }
                        ui.label(RichText::new(describe(p)).color(crate::theme::MUTED()));
                    });
                }
            });
            ui.add_space(6.0);
            ui.label(RichText::new(t("Minimums are based on the reference resolution (18 px text and 64 px per control at 1080p).")).small().color(crate::theme::MUTED()));
        });
        if let Some(id) = choose {
            self.sel = vec![id];
        }
        self.review_open = open;
    }
}

#[cfg(test)]
mod font_tests {
    use super::*;
    use crate::model::{Doc, Element, Kind};
    use std::sync::Arc;

    fn u16b(v: &mut Vec<u8>, x: u16) {
        v.extend_from_slice(&x.to_be_bytes());
    }
    fn u32b(v: &mut Vec<u8>, x: u32) {
        v.extend_from_slice(&x.to_be_bytes());
    }

    /// cmap table with one (3, 1) subtable of format 4 covering the given inclusive ranges (glyph = code point).
    fn cmap4(ranges: &[(u16, u16)]) -> Vec<u8> {
        let mut segs: Vec<(u16, u16)> = ranges.to_vec();
        segs.push((0xFFFF, 0xFFFF));
        let n = segs.len() as u16;
        let mut sub = Vec::new();
        u16b(&mut sub, 4);
        u16b(&mut sub, 16 + 8 * n); // length
        u16b(&mut sub, 0);
        u16b(&mut sub, n * 2);
        u16b(&mut sub, 0);
        u16b(&mut sub, 0);
        u16b(&mut sub, 0);
        for (_, e) in &segs {
            u16b(&mut sub, *e);
        }
        u16b(&mut sub, 0); // reservedPad
        for (s, _) in &segs {
            u16b(&mut sub, *s);
        }
        for (i, _) in segs.iter().enumerate() {
            // the last segment maps 0xFFFF to glyph 0 (delta 1 => 0)
            u16b(&mut sub, if i + 1 == segs.len() { 1 } else { 0 });
        }
        for _ in &segs {
            u16b(&mut sub, 0);
        }
        let mut t = Vec::new();
        u16b(&mut t, 0);
        u16b(&mut t, 1);
        u16b(&mut t, 3);
        u16b(&mut t, 1);
        u32b(&mut t, 12);
        t.extend(sub);
        t
    }

    /// cmap table with one (3, 10) subtable of format 12.
    fn cmap12(groups: &[(u32, u32)]) -> Vec<u8> {
        let mut sub = Vec::new();
        u16b(&mut sub, 12);
        u16b(&mut sub, 0);
        u32b(&mut sub, 16 + 12 * groups.len() as u32);
        u32b(&mut sub, 0);
        u32b(&mut sub, groups.len() as u32);
        let mut glyph = 1;
        for (s, e) in groups {
            u32b(&mut sub, *s);
            u32b(&mut sub, *e);
            u32b(&mut sub, glyph);
            glyph += e - s + 1;
        }
        let mut t = Vec::new();
        u16b(&mut t, 0);
        u16b(&mut t, 1);
        u16b(&mut t, 3);
        u16b(&mut t, 10);
        u32b(&mut t, 12);
        t.extend(sub);
        t
    }

    /// A minimal sfnt file containing only the cmap table.
    fn font_with(cmap: &[u8]) -> Vec<u8> {
        let mut f = Vec::new();
        u32b(&mut f, 0x0001_0000);
        u16b(&mut f, 1);
        f.extend_from_slice(&[0; 6]);
        f.extend_from_slice(b"cmap");
        u32b(&mut f, 0);
        u32b(&mut f, 28);
        u32b(&mut f, cmap.len() as u32);
        f.extend_from_slice(cmap);
        f
    }

    #[test]
    fn reads_format_4_and_12_from_a_font_file() {
        // Latin letters, digits, and "—" (U+2014); no guillemets
        let f4 = font_with(&cmap4(&[(0x20, 0x7E), (0x2014, 0x2014)]));
        let c = Cmap::from_font(&f4).expect("cmap parsed");
        assert!(c.has('A') && c.has('z') && c.has('—'));
        assert!(!c.has('«') && !c.has('»') && !c.has('…') && !c.has('ñ'));
        let f12 = font_with(&cmap12(&[(0x41, 0x5A), (0x1F600, 0x1F64F)]));
        let c = Cmap::from_font(&f12).unwrap();
        assert!(c.has('Q') && c.has('\u{1F603}'));
        assert!(!c.has('q') && !c.has('«'));
        assert_eq!(missing_chars(&c, "AB « C — «"), vec!['«', '—']);
    }

    #[test]
    fn garbage_and_truncated_fonts_do_not_panic() {
        assert!(Cmap::from_font(&[]).is_none());
        assert!(Cmap::from_font(b"not a font at all").is_none());
        let good = font_with(&cmap4(&[(0x20, 0x7E)]));
        for cut in 0..good.len() {
            if let Some(c) = Cmap::from_font(&good[..cut]) {
                let _ = c.has('A');
            }
        }
        // a table that claims huge counts
        let mut bad = good.clone();
        bad[34] = 0xFF;
        bad[35] = 0xFF;
        if let Some(c) = Cmap::from_font(&bad) {
            let _ = c.has('A');
            let _ = c.has('«');
        }
    }

    #[test]
    fn warns_about_missing_characters_only_for_known_fonts() {
        let cmap = Arc::new(Cmap::parse(cmap4(&[(0x20, 0x7E)])).unwrap());
        let known = {
            let cmap = cmap.clone();
            move |name: &str| if name == "TestSans" { Some(cmap.clone()) } else { None }
        };
        let mut d = Doc::empty(1920.0, 1080.0);
        let mut t = Element::new(0, Kind::Text);
        t.text = "Hola «mundo» — fin".into();
        t.font = "TestSans".into();
        d.add(t);
        let mut other = Element::new(0, Kind::Text);
        other.text = "«mundo»".into();
        other.font = "Unknown font".into();
        d.add(other);
        let mut default_font = Element::new(0, Kind::Text);
        default_font.text = "«mundo»".into();
        d.add(default_font);
        let mut ok = Element::new(0, Kind::Text);
        ok.text = "plain ascii".into();
        ok.font = "TestSans".into();
        d.add(ok);
        let mut out = Vec::new();
        font_problems(&d, &visible_all(&d), &mut out, &known);
        assert_eq!(out.len(), 1, "{:?}", out.iter().map(describe).collect::<Vec<_>>());
        assert_eq!(out[0].level, Level::Warning);
        let text = describe(&out[0]); // translated if another test switched the global language
        assert!(text.contains("TestSans") && text.contains("« » —"), "{}", text);
    }
}

#[cfg(test)]
mod portrait_tests {
    use super::*;
    use crate::docext::ElementOverride;
    use crate::model::{Doc, Element, Kind};

    #[test]
    fn portrait_layout_is_reviewed_on_the_resolved_design() {
        let mut d = Doc::empty(1920.0, 1080.0);
        let mut b = Element::new(0, Kind::Button);
        b.name = "Play".into();
        b.x = 1500.0; // fine in landscape, off the 1080 px wide portrait page
        b.y = 100.0;
        b.w = 300.0;
        b.h = 100.0;
        let id = d.add(b);
        assert!(review(&d).is_empty(), "no portrait layout: nothing about portrait");
        let mut ov = ElementOverride::new(id);
        ov.w = Some(301.0); // creates the portrait layout (1080 x 1920) without moving the button
        assert!(d.set_override(ov));
        let ps = review(&d);
        let off: Vec<&Problem> = ps.iter().filter(|p| p.template.contains("outside")).collect();
        assert_eq!(off.len(), 1, "{:?}", ps.iter().map(describe).collect::<Vec<_>>());
        assert!(off[0].element.contains("(portrait)"));
        // moving it inside the portrait page fixes the portrait review
        let mut ov = ElementOverride::new(id);
        ov.x = Some(300.0);
        d.set_override(ov);
        assert!(review(&d).iter().all(|p| !p.template.contains("outside")));
        // text that is too small in portrait only
        let mut t = Element::new(0, Kind::Text);
        t.text = "Small".into();
        t.text_color = [0, 0, 0, 255];
        t.font_size = 40.0;
        t.x = 100.0;
        t.y = 400.0;
        let tid = d.add(t);
        let mut ov = ElementOverride::new(tid);
        ov.font_size = Some(10.0);
        d.set_override(ov);
        let ps = review(&d);
        assert!(ps.iter().any(|p| p.template.starts_with("{} px text") && p.element.contains("(portrait)")));
        assert!(!ps.iter().any(|p| p.template.starts_with("{} px text") && !p.element.contains("(portrait)")));
    }
}
