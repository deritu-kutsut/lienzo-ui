//! "View" scripts for the uGUI prefab: a generated base class with typed
//! references to the elements (serialized fields filled by the importer; path lookup only as a fallback
//! for old prefabs) and click/value events, plus a
//! user class (written once, never overwritten) to put the game logic in.

use super::{class_name, comment_lines, cs_str, sanitize_line, unity_names, unity_path};
use crate::model::{Doc, Kind};
use std::collections::HashSet;
use std::fmt::Write;

fn field_type(k: Kind) -> Option<&'static str> {
    Some(match k {
        Kind::Button => "Button",
        Kind::Toggle => "Toggle",
        Kind::Slider => "Slider",
        Kind::InputField => "TMP_InputField",
        Kind::Dropdown => "TMP_Dropdown",
        Kind::ScrollView => "ScrollRect",
        Kind::Text => "TextMeshProUGUI",
        Kind::Image | Kind::Ellipse | Kind::Panel => "Image",
        Kind::Empty => "RectTransform",
    })
}

struct Item {
    id: u64,
    field: String,
    method: String,
    ty: &'static str,
    path: String,
    kind: Kind,
    name: String,
    code: String,
}

/// `[A-Za-z_][A-Za-z0-9_]*` (ASCII): usable verbatim as a C# member name.
fn is_cs_ident(s: &str) -> bool {
    let mut c = s.chars();
    matches!(c.next(), Some(f) if f.is_ascii_alphabetic() || f == '_') && c.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Members of the generated class a field must not be called (they would hide or clash with them).
const RESERVED_MEMBERS: &[&str] = &["Bind", "Find", "Awake", "OnEnable", "OnDisable"];

fn items(doc: &Doc) -> Vec<Item> {
    let names = unity_names(doc);
    let explicit = |e: &crate::model::Element| e.unity_name.as_deref().is_some_and(crate::docext::valid_unity_name);
    let mut used = HashSet::new();
    let mut used_fields = HashSet::new();
    let mut out = Vec::new();
    for e in &doc.elements {
        let Some(ty) = field_type(e.kind) else { continue };
        // The final GameObject name (a valid `unity_name` verbatim): members are named after it.
        let final_name = names.get(&e.id).cloned().unwrap_or_else(|| e.name.clone());
        let base = class_name(&final_name);
        let mut m = base.clone();
        let mut k = 2;
        while !used.insert(m.clone()) {
            m = format!("{}{}", base, k);
            k += 1;
        }
        let camel = {
            let mut c = m.chars();
            match c.next() {
                Some(f) => f.to_ascii_lowercase().to_string() + c.as_str(),
                None => "element".into(),
            }
        };
        // An explicit `unity_name` that is already a C# identifier becomes the member verbatim.
        let mut field = if explicit(e) && is_cs_ident(&final_name) { final_name.clone() } else { camel };
        field = super::safe_field(&field);
        if RESERVED_MEMBERS.contains(&field.as_str()) {
            field.push_str("Ui");
        }
        let root = field.clone();
        let mut k = 2;
        while !used_fields.insert(field.clone()) {
            field = format!("{}{}", root, k);
            k += 1;
        }
        let m = if e.kind == Kind::Button { format!("{}Click", m) } else { format!("{}Changed", m) };
        out.push(Item { id: e.id, field, method: m, ty, path: view_path(doc, &names, e.id), kind: e.kind, name: final_name, code: e.on_click.clone() });
    }
    out
}

/// Path inside the prefab root (`transform.Find`): the safe-area container comes first when the design uses it.
fn view_path(doc: &Doc, names: &std::collections::HashMap<u64, String>, id: u64) -> String {
    let p = unity_path(doc, names, id);
    if doc.canvas.safe_area { format!("{}/{}", super::GEN_SAFE_AREA, p) } else { p }
}

/// Name of the serialized field the importer fills for each element (element id -> field).
pub fn field_names(doc: &Doc) -> std::collections::HashMap<u64, String> {
    items(doc).into_iter().map(|i| (i.id, i.field)).collect()
}

pub fn base_script(doc: &Doc) -> String {
    let cls = class_name(&doc.title);
    let its = items(doc);
    let mut fields = String::new();
    let mut bind = String::new();
    let mut on = String::new();
    let mut off = String::new();
    let mut methods = String::new();
    for it in &its {
        let _ = writeln!(fields, "    public {} {}; // {}", it.ty, it.field, sanitize_line(&it.name));
        let _ = writeln!(bind, "        if (!{f}) {f} = Find<{t}>({p});", f = it.field, t = it.ty, p = cs_str(&it.path));
        let (ev, sig, arg) = match it.kind {
            Kind::Button => ("onClick", "()", ""),
            Kind::Toggle => ("onValueChanged", "(bool isOn)", "bool"),
            Kind::Slider => ("onValueChanged", "(float value)", "float"),
            Kind::InputField => ("onEndEdit", "(string text)", "string"),
            Kind::Dropdown => ("onValueChanged", "(int index)", "int"),
            _ => continue,
        };
        let _ = writeln!(on, "        if ({f}) {f}.{ev}.AddListener(On{m});", f = it.field, ev = ev, m = it.method);
        let _ = writeln!(off, "        if ({f}) {f}.{ev}.RemoveListener(On{m});", f = it.field, ev = ev, m = it.method);
        let mut body = String::new();
        if !it.code.trim().is_empty() {
            body.push_str("        // Editor “on click” code (translate it to C#):
");
            for l in comment_lines(&it.code) {
                let _ = writeln!(body, "        // {}", l);
            }
        }
        let _ = arg;
        let _ = writeln!(methods, "    protected virtual void On{}{}
    {{
{}    }}
", it.method, sig, body);
    }
    format!(
        r#"// Generated by Lienzo UI · “{title}”. Overwritten on every export: write your code in {cls}View.cs.
using TMPro;
using UnityEngine;
using UnityEngine.UI;

/// References to the elements of the prefab “{title}” and its events.
/// The Lienzo importer assigns these fields as serialized references when it builds the prefab, so renaming an
/// element in Lienzo or in Unity does not break the code. `Bind()` only fills the ones that are still empty
/// (prefabs generated by older versions) by looking the elements up by their path.
public abstract class {cls}ViewBase : MonoBehaviour
{{
{fields}
    protected virtual void Awake() => Bind();

    /// Fills the empty references by path (fallback for old prefabs).
    [ContextMenu("Bind elements")]
    public void Bind()
    {{
{bind}    }}

    protected virtual void OnEnable()
    {{
{on}    }}

    protected virtual void OnDisable()
    {{
{off}    }}

{methods}    protected T Find<T>(string path) where T : Component
    {{
        var t = transform.Find(path);
        if (t == null) {{ Debug.LogWarning("[{cls}] Not found: " + path, this); return null; }}
        return t.GetComponent<T>();
    }}
}}
"#,
        title = sanitize_line(&doc.title),
        cls = cls,
        fields = fields,
        bind = bind,
        on = on,
        off = off,
        methods = methods,
    )
}

pub fn user_script(doc: &Doc) -> String {
    let cls = class_name(&doc.title);
    let its = items(doc);
    let example = its
        .iter()
        .find(|i| i.kind == Kind::Button)
        .map(|i| format!("    protected override void On{}()\n    {{\n        Debug.Log({});\n    }}\n", i.method, cs_str(&format!("Click on {}", i.name))))
        .unwrap_or_default();
    format!(
        r#"// Your logic for “{title}”. Lienzo UI creates this file once and never touches it again.
using UnityEngine;

public class {cls}View : {cls}ViewBase
{{
{example}}}
"#,
        title = sanitize_line(&doc.title),
        cls = cls,
        example = example,
    )
}
