//! IMGUI export: a MonoBehaviour that draws the design in `OnGUI()` with the
//! `UnityEngine.GUI` API (GUI.DrawTexture with rounded corners, GUI.Label,
//! GUI.Button, GUI.Toggle, GUI.HorizontalSlider, GUI.TextField,
//! GUI.BeginScrollView / BeginGroup). Rectangles are computed from the same
//! anchors as uGUI, so the layout adapts to any screen size.

use super::{cf, class_name, comment_lines, cs_str, radius_of, sanitize_line, ExportCtx, ImageNames};
use crate::model::{Doc, Element, Kind, Rgba};
use std::collections::{HashMap, HashSet};
use std::fmt::Write;

fn color(c: Rgba, op: f32) -> String {
    format!(
        "new Color({}, {}, {}, {})",
        cf(c[0] as f32 / 255.0),
        cf(c[1] as f32 / 255.0),
        cf(c[2] as f32 / 255.0),
        cf(c[3] as f32 / 255.0 * op)
    )
}

fn anchor(e: &Element) -> &'static str {
    let v = if e.kind == Kind::Text { 0 } else { e.valign.min(2) };
    ["UpperLeft", "UpperCenter", "UpperRight", "MiddleLeft", "MiddleCenter", "MiddleRight", "LowerLeft", "LowerCenter", "LowerRight"][(v * 3 + e.align.min(2)) as usize]
}

fn font_style(e: &Element) -> &'static str {
    match (e.bold, e.italic) {
        (true, true) => "BoldAndItalic",
        (true, false) => "Bold",
        (false, true) => "Italic",
        _ => "Normal",
    }
}

/// Unique PascalCase identifiers per element.
fn idents(doc: &Doc) -> HashMap<u64, String> {
    let mut used = HashSet::new();
    let mut out = HashMap::new();
    for e in &doc.elements {
        let base = class_name(&e.name);
        let mut n = base.clone();
        let mut k = 2;
        while !used.insert(n.clone()) {
            n = format!("{}{}", base, k);
            k += 1;
        }
        out.insert(e.id, n);
    }
    out
}

struct Gen<'a> {
    doc: &'a Doc,
    ctx: &'a ExportCtx<'a>,
    imgs: ImageNames,
    ids: HashMap<u64, String>,
    fields: String,
    styles: String,
    body: String,
    methods: String,
    nstyle: usize,
}

impl<'a> Gen<'a> {
    fn style(&mut self, e: &Element, color_override: Option<String>) -> usize {
        let i = self.nstyle;
        self.nstyle += 1;
        let c = color_override.unwrap_or_else(|| color(e.text_color, e.opacity));
        let _ = writeln!(self.styles, "        st[{}] = Txt({}, FontStyle.{}, TextAnchor.{}, {});", i, (e.font_size.round() as i32).max(1), font_style(e), anchor(e), c);
        i
    }

    fn el(&mut self, id: u64, pr: &str, pm: &str, in_clip: bool, ind: usize) {
        let Some(e) = self.ctx.get(id).cloned() else { return };
        let n = self.ids[&id].clone();
        let pad = "    ".repeat(ind);
        let rt = self.ctx.rt(&e);
        let op = e.opacity;
        let b = &mut self.body;
        let _ = writeln!(self.fields, "    public bool show{} = {};", n, e.visible);
        let _ = writeln!(b, "{}// {} ({})", pad, sanitize_line(&e.name), e.kind.unity_type());
        let _ = writeln!(b, "{}if (show{})", pad, n);
        let _ = writeln!(b, "{}{{", pad);
        let p2 = format!("{}    ", pad);
        let r = format!("r{}", n);
        let _ = writeln!(
            b,
            "{}var {} = Place({}, new Vector2({}, {}), new Vector2({}, {}), new Vector2({}, {}), new Vector2({}, {}), new Vector2({}, {}));",
            p2,
            r,
            pr,
            cf(rt.anchor_min[0]),
            cf(rt.anchor_min[1]),
            cf(rt.anchor_max[0]),
            cf(rt.anchor_max[1]),
            cf(rt.pivot[0]),
            cf(rt.pivot[1]),
            cf(rt.anchored_position[0]),
            cf(rt.anchored_position[1]),
            cf(rt.size_delta[0]),
            cf(rt.size_delta[1])
        );
        let rotated = rt.rotation_z.abs() > 0.01;
        let mut m = pm.to_string();
        if rotated {
            if in_clip {
                let _ = writeln!(b, "{}// (the {}° rotation is ignored inside an IMGUI mask or scroll)", p2, -rt.rotation_z);
            } else {
                m = format!("m{}", n);
                let _ = writeln!(b, "{}var {} = {} * Rot({}, new Vector2({}, {}), {});", p2, m, pm, r, cf(rt.pivot[0]), cf(rt.pivot[1]), cf(-rt.rotation_z));
                let _ = writeln!(b, "{}GUI.matrix = {};", p2, m);
            }
        }
        let radius = if e.kind == Kind::Ellipse { format!("Mathf.Min({r}.width, {r}.height) * 0.5f") } else { cf(radius_of(&e)) };
        if e.kind.selectable() && !e.interactable {
            let _ = writeln!(b, "{}GUI.enabled = false;", p2);
        }
        // shadow
        if e.shadow.on && e.kind.has_fill() && !matches!(e.kind, Kind::Toggle | Kind::Slider) {
            let _ = writeln!(b, "{}Box(new Rect({r}.x + {}, {r}.y + {}, {r}.width, {r}.height), {}, {});", p2, cf(e.shadow.offset[0]), cf(-e.shadow.offset[1]), color(e.shadow.color, op), radius);
        }
        let (pl, prr, pv) = crate::render::text_padding(&e);
        let inset = format!("Inset({}, {}, {}, {})", r, cf(pl), cf(prr), cf(pv));
        let border = |b: &mut String| {
            if e.stroke_w > 0.0 {
                let _ = writeln!(b, "{}Border({}, {}, {}, {});", p2, r, color(e.stroke, op), cf(e.stroke_w), radius);
            }
        };
        match e.kind {
            Kind::Empty => {}
            Kind::Panel | Kind::Image | Kind::Ellipse | Kind::ScrollView => {
                if !e.image.is_empty() {
                    let _ = writeln!(self.fields, "    public Texture2D image{}; // {}", n, sanitize_line(&self.imgs.file(&e.image)));
                    let _ = writeln!(b, "{}if (image{} != null) GUI.DrawTexture({}, image{}, ScaleMode.StretchToFill, true, 0f, new Color(1f, 1f, 1f, {}), 0f, {});", p2, n, r, n, cf(op), radius);
                    let _ = writeln!(b, "{}else Box({}, {}, {});", p2, r, color(e.fill, op), radius);
                } else {
                    if e.kind == Kind::Image {
                        let _ = writeln!(b, "{}// editor gradient: IMGUI uses the first color", p2);
                    }
                    let _ = writeln!(b, "{}Box({}, {}, {});", p2, r, color(e.fill, op), radius);
                }
                border(b);
                if !e.text.is_empty() && e.kind != Kind::ScrollView {
                    let s = self.style(&e, None);
                    let _ = writeln!(self.body, "{}GUI.Label({}, {}, st[{}]);", p2, inset, cs_str(&e.text), s);
                }
            }
            Kind::Text => {
                if e.fill[3] > 0 {
                    let _ = writeln!(b, "{}Box({}, {}, {});", p2, r, color(e.fill, op), radius);
                }
                let _ = writeln!(self.fields, "    public string text{} = {};", n, cs_str(&e.text));
                let s = self.style(&e, None);
                let _ = writeln!(self.body, "{}GUI.Label({}, text{}, st[{}]);", p2, r, n, s);
            }
            Kind::Button => {
                let _ = writeln!(b, "{}Box({}, GUI.enabled && Hover({}) ? Color.Lerp({}, Color.white, 0.15f) : {}, {});", p2, r, r, color(e.fill, op), color(e.fill, op), radius);
                border(b);
                let _ = writeln!(b, "{}if (GUI.Button({}, GUIContent.none, GUIStyle.none)) {{ Clicked?.Invoke({}); OnClick{}(); }}", p2, r, cs_str(&e.name), n);
                if !e.text.is_empty() {
                    let s = self.style(&e, None);
                    let _ = writeln!(self.body, "{}GUI.Label({}, {}, st[{}]);", p2, inset, cs_str(&e.text), s);
                }
                let mut code = String::new();
                for l in comment_lines(&e.on_click) {
                    let _ = writeln!(code, "        // {}", l);
                }
                let _ = writeln!(self.methods, "    /// Click on “{}”.\n    protected virtual void OnClick{}()\n    {{\n{}    }}\n", sanitize_line(&e.name), n, if code.is_empty() { String::new() } else { format!("        // Editor “on click” code (translate it to C#):\n{}", code) });
            }
            Kind::Toggle => {
                let _ = writeln!(self.fields, "    public bool toggle{} = {};", n, e.value > 0.5);
                let s = e.h.min(e.w) * 0.8;
                let _ = writeln!(b, "{}toggle{n} = GUI.Toggle({r}, toggle{n}, GUIContent.none, GUIStyle.none);", p2);
                let _ = writeln!(b, "{}var box{n} = new Rect({r}.x + {}, {r}.y + ({r}.height - {s}) * 0.5f, {s}, {s});", p2, cf(e.h * 0.1), s = cf(s));
                let _ = writeln!(b, "{}if (toggle{n}) {{ Box(box{n}, {}, {}); GUI.Label(box{n}, \"✓\", tick); }}", p2, color(e.fill, op), cf(s * 0.2));
                let _ = writeln!(b, "{}else {{ Box(box{n}, new Color(1f, 1f, 1f, {}), {}); Border(box{n}, new Color(0.67f, 0.67f, 0.7f, {}), 2f, {}); }}", p2, cf(op), cf(s * 0.2), cf(op), cf(s * 0.2));
                if !e.text.is_empty() {
                    let st = self.style(&e, None);
                    let _ = writeln!(self.body, "{}GUI.Label({}, {}, st[{}]);", p2, inset, cs_str(&e.text), st);
                }
            }
            Kind::Slider => {
                let _ = writeln!(self.fields, "    [Range(0f, 1f)] public float slider{} = {};", n, cf(e.value));
                let k = e.h * 0.5;
                let th = (e.h * 0.32).max(2.0);
                let i = self.nstyle;
                self.nstyle += 1;
                let _ = writeln!(self.styles, "        st[{}] = new GUIStyle {{ fixedWidth = {}, fixedHeight = {} }};", i, cf(2.0 * k), cf(e.h));
                let b = &mut self.body;
                let _ = writeln!(b, "{}slider{n} = GUI.HorizontalSlider({r}, slider{n}, 0f, 1f, GUIStyle.none, st[{i}]);", p2);
                let _ = writeln!(b, "{}var track{n} = new Rect({r}.x + {k}, {r}.center.y - {h2}, {r}.width - 2f * {k}, {th});", p2, k = cf(k), h2 = cf(th / 2.0), th = cf(th));
                let _ = writeln!(b, "{}Box(track{n}, {}, {});", p2, color(e.fill2, op), cf(th / 2.0));
                let _ = writeln!(b, "{}Box(new Rect(track{n}.x, track{n}.y, Mathf.Max({th}, track{n}.width * slider{n}), {th}), {}, {});", p2, color(e.fill, op), cf(th / 2.0), th = cf(th));
                let _ = writeln!(b, "{}var knob{n} = new Rect(track{n}.x + track{n}.width * slider{n} - {kk}, {r}.center.y - {kk}, 2f * {kk}, 2f * {kk});", p2, kk = cf(k * 0.8));
                let _ = writeln!(b, "{}Box(knob{n}, new Color(1f, 1f, 1f, {}), {});", p2, cf(op), cf(k * 0.8));
                let _ = writeln!(b, "{}Border(knob{n}, {}, 2f, {});", p2, color(e.fill, op), cf(k * 0.8));
            }
            Kind::InputField => {
                let _ = writeln!(self.fields, "    public string input{} = \"\";", n);
                let _ = writeln!(b, "{}Box({}, {}, {});", p2, r, color(e.fill, op), radius);
                border(b);
                let ph = self.style(&e, None);
                let i = self.nstyle;
                self.nstyle += 1;
                let _ = writeln!(self.styles, "        st[{}] = Field({}, TextAnchor.{}, {});", i, (e.font_size.round() as i32).max(1), anchor(&e), color(e.text_color, op));
                let b = &mut self.body;
                let _ = writeln!(b, "{}if (string.IsNullOrEmpty(input{n}) && GUI.GetNameOfFocusedControl() != \"in{n}\") GUI.Label({inset}, {}, st[{ph}]);", p2, cs_str(&e.text));
                let _ = writeln!(b, "{}GUI.SetNextControlName(\"in{n}\");", p2);
                let _ = writeln!(b, "{}input{n} = GUI.TextField({inset}, input{n}, st[{i}]);", p2);
            }
            Kind::Dropdown => {
                let opts: Vec<String> = e.options.iter().map(|o| cs_str(o)).collect();
                let _ = writeln!(self.fields, "    public string[] options{} = {{ {} }};", n, opts.join(", "));
                let _ = writeln!(self.fields, "    public int dropdown{} = {};", n, e.value.round().max(0.0) as i64);
                let _ = writeln!(self.fields, "    bool open{};", n);
                let _ = writeln!(b, "{}Box({}, {}, {});", p2, r, color(e.fill, op), radius);
                border(b);
                let s = self.style(&e, None);
                let b = &mut self.body;
                let _ = writeln!(b, "{}if (options{n}.Length > 0) GUI.Label({inset}, options{n}[Mathf.Clamp(dropdown{n}, 0, options{n}.Length - 1)], st[{s}]);", p2);
                let _ = writeln!(b, "{}GUI.Label(new Rect({r}.xMax - {a}, {r}.y, {a}, {r}.height), \"▼\", st[{s}]);", p2, a = cf(e.h * 0.8));
                let _ = writeln!(b, "{}if (GUI.Button({r}, GUIContent.none, GUIStyle.none)) open{n} = !open{n};", p2);
                let _ = writeln!(b, "{}if (open{n})", p2);
                let _ = writeln!(b, "{}{{", p2);
                let _ = writeln!(b, "{}    var mm = GUI.matrix; var rr = {r};", p2);
                let _ = writeln!(b, "{}    System.Action list = () =>", p2);
                let _ = writeln!(b, "{}    {{", p2);
                let _ = writeln!(b, "{}        GUI.matrix = mm;", p2);
                let _ = writeln!(b, "{}        for (int i = 0; i < options{n}.Length; i++)", p2);
                let _ = writeln!(b, "{}        {{", p2);
                let _ = writeln!(b, "{}            var ir = new Rect(rr.x, rr.yMax + 4f + i * rr.height * 0.8f, rr.width, rr.height * 0.8f);", p2);
                let _ = writeln!(b, "{}            Box(ir, i == dropdown{n} ? new Color(0.85f, 0.88f, 1f, 1f) : (Hover(ir) ? new Color(0.94f, 0.95f, 0.98f, 1f) : Color.white), 0f);", p2);
                let _ = writeln!(b, "{}            if (GUI.Button(ir, GUIContent.none, GUIStyle.none)) {{ dropdown{n} = i; open{n} = false; }}", p2);
                let _ = writeln!(b, "{}            GUI.Label(Inset(ir, {}, {}, 0f), options{n}[i], st[{s}]);", p2, cf(pl), cf(pl));
                let _ = writeln!(b, "{}        }}", p2);
                let _ = writeln!(b, "{}    }};", p2);
                if in_clip {
                    let _ = writeln!(b, "{}    list();", p2);
                } else {
                    let _ = writeln!(b, "{}    overlay += list;", p2);
                }
                let _ = writeln!(b, "{}}}", p2);
            }
        }
        if e.kind.selectable() && !e.interactable {
            let _ = writeln!(self.body, "{}GUI.enabled = true;", p2);
        }
        // children
        let kids = self.ctx.children(Some(id)).to_vec();
        if !kids.is_empty() {
            if e.kind == Kind::ScrollView {
                let ch = self.ctx.content_height(&e);
                let _ = writeln!(self.fields, "    Vector2 scroll{};", n);
                let _ = writeln!(self.body, "{}var c{n} = new Rect(0f, 0f, {r}.width - 18f, Mathf.Max({ch}, {r}.height));", p2, ch = cf(ch));
                let _ = writeln!(self.body, "{}scroll{n} = GUI.BeginScrollView({r}, scroll{n}, c{n}, false, false);", p2);
                for k in kids {
                    self.el(k, &format!("c{}", n), &m, true, ind + 1);
                }
                let _ = writeln!(self.body, "{}GUI.EndScrollView();", p2);
            } else if e.mask {
                let _ = writeln!(self.body, "{}GUI.BeginGroup({r});", p2);
                let _ = writeln!(self.body, "{}var c{n} = new Rect(0f, 0f, {r}.width, {r}.height);", p2);
                for k in kids {
                    self.el(k, &format!("c{}", n), &m, true, ind + 1);
                }
                let _ = writeln!(self.body, "{}GUI.EndGroup();", p2);
            } else {
                for k in kids {
                    self.el(k, &r, &m, in_clip, ind + 1);
                }
            }
        }
        if rotated && !in_clip {
            let _ = writeln!(self.body, "{}GUI.matrix = {};", p2, pm);
        }
        let _ = writeln!(self.body, "{}}}", pad);
    }
}

pub fn script(doc: &Doc) -> String {
    let cls = format!("{}OnGUI", class_name(&doc.title));
    let ctx = ExportCtx::new(doc);
    let mut g = Gen { doc, ctx: &ctx, imgs: ImageNames::new(doc), ids: idents(doc), fields: String::new(), styles: String::new(), body: String::new(), methods: String::new(), nstyle: 0 };
    for &id in ctx.children(None) {
        g.el(id, "root", "m0", false, 2);
    }
    let bg = if doc.has_background() { format!("        if (drawBackground) Box(root, {}, 0f);\n", color(doc.background, 1.0)) } else { String::new() };
    format!(
        r#"// Generated by Lienzo UI · “{title}” — IMGUI interface (UnityEngine.GUI).
// Add this component to any GameObject. It is drawn in OnGUI and adapts to the
// screen like a CanvasScaler "Scale With Screen Size" ({w} x {h}, match {m}).
// It is overwritten on every export: for your logic, inherit from this class and
// override the OnClick… methods or listen to the Clicked event.
using System;
using UnityEngine;

public class {cls} : MonoBehaviour
{{
    [Header("Canvas")]
    public Vector2 referenceResolution = new Vector2({w}f, {h}f);
    [Range(0f, 1f)] public float match = {mf};
    public int depth = 0;
    public bool drawBackground = true;
    [Tooltip("Optional font for all texts")]
    public Font font;

    /// Invoked with the name of the pressed button.
#pragma warning disable CS0067
    public event Action<string> Clicked;
#pragma warning restore CS0067

    // ---- state and elements
{fields}
    GUIStyle[] st;
    GUIStyle tick;
    Action overlay;

    protected virtual void OnGUI()
    {{
        GUI.depth = depth;
        float s = Mathf.Pow(2f, Mathf.Lerp(Mathf.Log(Screen.width / referenceResolution.x, 2f), Mathf.Log(Screen.height / referenceResolution.y, 2f), match));
        var baseMatrix = GUI.matrix;
        var m0 = baseMatrix * Matrix4x4.Scale(new Vector3(s, s, 1f));
        GUI.matrix = m0;
        var root = new Rect(0f, 0f, Screen.width / s, Screen.height / s);
        if (st == null) Styles();
        overlay = null;
{bg}{body}
        if (overlay != null) overlay();
        GUI.matrix = baseMatrix;
    }}

{methods}    // ---- utilidades
    void Styles()
    {{
        st = new GUIStyle[{n}];
{styles}        tick = Txt(20, FontStyle.Bold, TextAnchor.MiddleCenter, Color.white);
    }}

    GUIStyle Txt(int size, FontStyle fs, TextAnchor a, Color c)
    {{
        var s = new GUIStyle(GUI.skin.label) {{ fontSize = size, fontStyle = fs, alignment = a, wordWrap = true, richText = false, clipping = TextClipping.Overflow, padding = new RectOffset(0, 0, 0, 0), margin = new RectOffset(0, 0, 0, 0) }};
        if (font != null) s.font = font;
        s.normal.textColor = s.hover.textColor = s.active.textColor = s.focused.textColor = c;
        return s;
    }}

    GUIStyle Field(int size, TextAnchor a, Color c)
    {{
        var s = new GUIStyle(GUI.skin.textField) {{ fontSize = size, alignment = a, wordWrap = false, richText = false, padding = new RectOffset(0, 0, 0, 0), margin = new RectOffset(0, 0, 0, 0) }};
        if (font != null) s.font = font;
        s.normal.background = s.hover.background = s.focused.background = s.active.background = null;
        s.normal.textColor = s.hover.textColor = s.focused.textColor = s.active.textColor = c;
        return s;
    }}

    /// Rectangle of a RectTransform (anchors, pivot, anchoredPosition, sizeDelta) inside `p` (y grows downward).
    protected static Rect Place(Rect p, Vector2 aMin, Vector2 aMax, Vector2 pivot, Vector2 pos, Vector2 size)
    {{
        float x0 = aMin.x * p.width, x1 = aMax.x * p.width;
        float y0 = aMin.y * p.height, y1 = aMax.y * p.height;
        float w = size.x + (x1 - x0), h = size.y + (y1 - y0);
        float px = x0 + pivot.x * (x1 - x0) + pos.x;
        float py = y0 + pivot.y * (y1 - y0) + pos.y;
        float left = px - pivot.x * w, top = py + (1f - pivot.y) * h;
        return new Rect(p.x + left, p.y + p.height - top, w, h);
    }}

    protected static Matrix4x4 Rot(Rect r, Vector2 pivot, float degrees)
    {{
        var p = new Vector3(r.x + r.width * pivot.x, r.y + r.height * (1f - pivot.y), 0f);
        return Matrix4x4.TRS(p, Quaternion.Euler(0f, 0f, degrees), Vector3.one) * Matrix4x4.TRS(-p, Quaternion.identity, Vector3.one);
    }}

    protected static Rect Inset(Rect r, float left, float right, float vertical) =>
        new Rect(r.x + left, r.y + vertical, Mathf.Max(0f, r.width - left - right), Mathf.Max(0f, r.height - 2f * vertical));

    protected static bool Hover(Rect r) => r.Contains(Event.current.mousePosition);

    protected static void Box(Rect r, Color c, float radius)
    {{
        if (c.a > 0f) GUI.DrawTexture(r, Texture2D.whiteTexture, ScaleMode.StretchToFill, true, 0f, c, 0f, radius);
    }}

    protected static void Border(Rect r, Color c, float width, float radius)
    {{
        if (width > 0f && c.a > 0f) GUI.DrawTexture(r, Texture2D.whiteTexture, ScaleMode.StretchToFill, true, 0f, c, width, radius);
    }}
}}
"#,
        title = sanitize_line(&doc.title),
        cls = cls,
        w = super::f(doc.width),
        h = super::f(doc.height),
        m = super::f(doc.canvas_match),
        mf = cf(doc.canvas_match),
        fields = g.fields,
        bg = bg,
        body = g.body,
        methods = g.methods,
        n = g.nstyle.max(1),
        styles = g.styles,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model;

    #[test]
    fn script_has_controls_and_balanced_braces() {
        for i in 0..model::TEMPLATES.len() {
            let d = model::template(i, 1920.0, 1080.0);
            let s = script(&d);
            assert_eq!(s.matches('{').count(), s.matches('}').count(), "template {}", i);
            assert!(s.contains("void OnGUI()"));
        }
        let s = script(&model::template(3, 1920.0, 1080.0));
        assert!(s.contains("GUI.HorizontalSlider") && s.contains("GUI.TextField") && s.contains("GUI.BeginScrollView") && s.contains("GUI.Toggle"));
    }

    // An input field's typed text uses the design's text colour (it used to be a fixed near-black,
    // unreadable on dark fields).
    #[test]
    fn input_field_text_uses_design_colour() {
        let mut d = model::template(5, 1920.0, 1080.0);
        let mut e = model::Element::new(0, model::Kind::InputField);
        e.text_color = [255, 255, 255, 255];
        d.add(e);
        let s = script(&d);
        assert!(s.contains("= Field(") && s.contains(", new Color(1f, 1f, 1f, 1f));"), "{}", s);
        assert!(!s.contains("new Color(0.1f, 0.1f, 0.13f, 1f)"));
    }
}
