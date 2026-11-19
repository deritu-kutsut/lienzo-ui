//! "View" scripts for the uGUI prefab: a generated base class with typed
//! references to the elements (serialized fields filled by the importer; path lookup only as a fallback
//! for old prefabs) and click/value events, plus a
//! user class (written once, never overwritten) to put the game logic in.

use super::{class_name, comment_lines, cs_str, sanitize_line, unity_names, unity_paths};
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
    let paths = unity_paths(doc, &names);
    let cls = class_name(&doc.title);
    let base_cls = format!("{}ViewBase", cls);
    let explicit = |e: &crate::model::Element| e.unity_name.as_deref().is_some_and(crate::docext::valid_unity_name);
    // elements that get a member, in document order
    let cand: Vec<(&crate::model::Element, &'static str, String)> = doc
        .elements
        .iter()
        .filter_map(|e| field_type(e.kind).map(|ty| (e, ty, names.get(&e.id).cloned().unwrap_or_else(|| e.name.clone()))))
        .collect();
    // Explicit `unity_name`s are served first (they keep their name; the automatic ones are suffixed), then the rest.
    let mut order: Vec<usize> = (0..cand.len()).filter(|&i| explicit(cand[i].0)).collect();
    order.extend((0..cand.len()).filter(|&i| !explicit(cand[i].0)));
    // member (event method) names
    let mut used: HashSet<String> = HashSet::new();
    let mut next_k: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut members: Vec<String> = vec![String::new(); cand.len()];
    for &i in &order {
        let base = class_name(&cand[i].2);
        let mut m = base.clone();
        if used.contains(&m) {
            let k = next_k.entry(base.clone()).or_insert(2);
            loop {
                m = format!("{}{}", base, *k);
                *k += 1;
                if !used.contains(&m) {
                    break;
                }
            }
        }
        used.insert(m.clone());
        members[i] = m;
    }
    let method_of = |i: usize| if cand[i].0.kind == Kind::Button { format!("{}Click", members[i]) } else { format!("{}Changed", members[i]) };
    // Every `On{method}` the class declares: a field must not be called like one of them.
    let handlers: HashSet<String> = (0..cand.len()).filter(|&i| matches!(cand[i].0.kind, Kind::Button | Kind::Toggle | Kind::Slider | Kind::InputField | Kind::Dropdown)).map(|i| format!("On{}", method_of(i))).collect();
    let blocked = |f: &str| RESERVED_MEMBERS.contains(&f) || f == base_cls || handlers.contains(f);
    // fields
    let mut used_fields: HashSet<String> = HashSet::new();
    let mut next_f: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut fields: Vec<String> = vec![String::new(); cand.len()];
    for &i in &order {
        let (e, _, final_name) = &cand[i];
        let camel = {
            let mut c = members[i].chars();
            match c.next() {
                Some(f) => f.to_ascii_lowercase().to_string() + c.as_str(),
                None => "element".into(),
            }
        };
        // An explicit `unity_name` that is already a C# identifier becomes the member verbatim.
        let mut field = if explicit(e) && is_cs_ident(final_name) { final_name.clone() } else { camel };
        field = super::safe_field(&field);
        while blocked(&field) {
            field.push_str("Ui");
        }
        let root = field.clone();
        if used_fields.contains(&field) {
            let k = next_f.entry(root.clone()).or_insert(2);
            loop {
                field = format!("{}{}", root, *k);
                *k += 1;
                if !used_fields.contains(&field) && !blocked(&field) {
                    break;
                }
            }
        }
        used_fields.insert(field.clone());
        fields[i] = field;
    }
    cand.iter()
        .enumerate()
        .map(|(i, (e, ty, final_name))| Item {
            id: e.id,
            field: fields[i].clone(),
            method: method_of(i),
            ty,
            path: view_path(doc, &paths, e.id),
            kind: e.kind,
            name: final_name.clone(),
            code: e.on_click.clone(),
        })
        .collect()
}

/// Path inside the prefab root (`transform.Find`): the safe-area container comes first when the design uses it.
fn view_path(doc: &Doc, paths: &std::collections::HashMap<u64, String>, id: u64) -> String {
    let p = paths.get(&id).cloned().unwrap_or_default();
    if doc.canvas.safe_area { format!("{}/{}", super::GEN_SAFE_AREA, p) } else { p }
}

// Name of the serialized field the importer fills for each element (element id -> field).
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
        if (t == null) {{ global::UnityEngine.Debug.LogWarning("[{cls}] Not found: " + path, this); return null; }}
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
        .map(|i| format!("    protected override void On{}()\n    {{\n        global::UnityEngine.Debug.Log({});\n    }}\n", i.method, cs_str(&format!("Click on {}", i.name))))
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
