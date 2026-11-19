//! `--validate <file>`: checks a `.editorgui.json` against `docs/editorgui.schema.json` (a small JSON Schema
//! subset: type, enum, minimum, maximum, minItems, maxItems, maxLength, items, properties, anyOf, $ref) plus the
//! rules a schema cannot express (unique ids, valid Unity names, portrait overrides pointing at real elements).
//! Errors carry the JSON path, e.g. `$.elements[3].fill[1]`.

use serde_json::Value;

pub const SCHEMA: &str = include_str!("../docs/editorgui.schema.json");

fn type_ok(t: &str, v: &Value) -> bool {
    match t {
        "object" => v.is_object(),
        "array" => v.is_array(),
        "string" => v.is_string(),
        "boolean" => v.is_boolean(),
        "null" => v.is_null(),
        "number" => v.is_number(),
        "integer" => v.is_i64() || v.is_u64() || v.as_f64().is_some_and(|f| f.fract() == 0.0 && f.abs() < 1e15),
        _ => true,
    }
}

fn kind(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn check(root: &Value, schema: &Value, v: &Value, path: &str, out: &mut Vec<String>, depth: u32) {
    if depth > 32 {
        return;
    }
    if let Some(r) = schema.get("$ref").and_then(|r| r.as_str()) {
        if let Some(target) = r.strip_prefix("#/").and_then(|p| p.split('/').try_fold(root, |n, k| n.get(k))) {
            check(root, target, v, path, out, depth + 1);
        }
        return;
    }
    if let Some(alts) = schema.get("anyOf").and_then(|a| a.as_array()) {
        let ok = alts.iter().any(|a| {
            let mut e = Vec::new();
            check(root, a, v, path, &mut e, depth + 1);
            e.is_empty()
        });
        if !ok {
            let mut e = Vec::new();
            if let Some(a) = alts.iter().find(|a| a.get("type").is_none_or(|t| t.as_str().is_none_or(|t| type_ok(t, v)))) {
                check(root, a, v, path, &mut e, depth + 1);
            }
            if e.is_empty() {
                out.push(format!("{}: value does not match any allowed form", path));
            } else {
                out.extend(e);
            }
        }
        return;
    }
    if let Some(t) = schema.get("type") {
        let types: Vec<&str> = match t {
            Value::String(s) => vec![s.as_str()],
            Value::Array(a) => a.iter().filter_map(|x| x.as_str()).collect(),
            _ => vec![],
        };
        if !types.is_empty() && !types.iter().any(|t| type_ok(t, v)) {
            out.push(format!("{}: expected {}, found {}", path, types.join(" or "), kind(v)));
            return;
        }
    }
    if let Some(en) = schema.get("enum").and_then(|e| e.as_array()) {
        if !en.contains(v) {
            let list: Vec<String> = en.iter().map(|x| x.to_string()).collect();
            out.push(format!("{}: {} is not one of {}", path, v, list.join(", ")));
        }
    }
    if let Some(n) = v.as_f64() {
        if let Some(min) = schema.get("minimum").and_then(|m| m.as_f64()) {
            if n < min {
                out.push(format!("{}: {} is below the minimum {}", path, n, min));
            }
        }
        if let Some(max) = schema.get("maximum").and_then(|m| m.as_f64()) {
            if n > max {
                out.push(format!("{}: {} is above the maximum {}", path, n, max));
            }
        }
    }
    if let (Some(s), Some(max)) = (v.as_str(), schema.get("maxLength").and_then(|m| m.as_u64())) {
        if s.chars().count() as u64 > max {
            out.push(format!("{}: text is longer than {} characters", path, max));
        }
    }
    if let Some(a) = v.as_array() {
        if let Some(min) = schema.get("minItems").and_then(|m| m.as_u64()) {
            if (a.len() as u64) < min {
                out.push(format!("{}: needs at least {} items, found {}", path, min, a.len()));
            }
        }
        if let Some(max) = schema.get("maxItems").and_then(|m| m.as_u64()) {
            if a.len() as u64 > max {
                out.push(format!("{}: allows at most {} items, found {}", path, max, a.len()));
            }
        }
        if let Some(items) = schema.get("items") {
            for (i, x) in a.iter().enumerate() {
                check(root, items, x, &format!("{}[{}]", path, i), out, depth + 1);
            }
        }
    }
    if let (Some(o), Some(props)) = (v.as_object(), schema.get("properties").and_then(|p| p.as_object())) {
        for (k, sub) in props {
            if let Some(x) = o.get(k) {
                check(root, sub, x, &format!("{}.{}", path, k), out, depth + 1);
            }
        }
    }
}

/// Errors (with JSON paths) of a parsed design file. Empty = valid.
pub fn validate_value(v: &Value) -> Vec<String> {
    let mut out = Vec::new();
    let schema: Value = serde_json::from_str(SCHEMA).unwrap_or(Value::Null);
    check(&schema, &schema, v, "$", &mut out, 0);
    if !out.is_empty() {
        return out;
    }
    semantic(v, &mut out);
    if out.is_empty() {
        if let Err(e) = serde_json::from_value::<crate::model::Doc>(v.clone()) {
            out.push(format!("$: {}", e));
        }
    }
    out
}

fn semantic(v: &Value, out: &mut Vec<String>) {
    let mut ids = std::collections::HashSet::new();
    let els = v.get("elements").and_then(|e| e.as_array()).cloned().unwrap_or_default();
    for (i, e) in els.iter().enumerate() {
        if let Some(id) = e.get("id").and_then(|i| i.as_u64()).filter(|i| *i > crate::model::MAX_ID) {
            out.push(format!("$.elements[{}].id: {} is above the maximum {}", i, id, crate::model::MAX_ID));
        }
        if let Some(id) = e.get("id").and_then(|i| i.as_u64()).filter(|i| *i != 0) {
            if !ids.insert(id) {
                out.push(format!("$.elements[{}].id: duplicate id {}", i, id));
            }
        }
        if let Some(u) = e.get("unity_name").and_then(|u| u.as_str()) {
            if !crate::docext::valid_unity_name(u) {
                out.push(format!("$.elements[{}].unity_name: invalid GameObject name (empty, contains '/' or control characters, longer than 128, or one of the names the importer generates: __border, __text, __tick, __safe_area, __canvas_background)", i));
            }
        }
        if let Some(f) = e.get("font").and_then(|f| f.as_str()) {
            if !crate::docext::valid_font_name(f) {
                out.push(format!("$.elements[{}].font: a font name must not contain '/', '\', ':' or '..'", i));
            }
        }
    }
    for (i, e) in els.iter().enumerate() {
        if let Some(p) = e.get("parent").and_then(|p| p.as_u64()) {
            if !ids.contains(&p) {
                out.push(format!("$.elements[{}].parent: no element has id {}", i, p));
            }
        }
    }
    if let Some(u) = v.get("unity") {
        if let Some(s) = u.get("scripts_dir").and_then(|s| s.as_str()) {
            // The loader normalises '' to '/' (and trims a trailing slash), so those are valid here too.
            if crate::docext::normalize_scripts_dir(s).is_none() {
                out.push("$.unity.scripts_dir: must be a relative path starting with Assets (use '/' or '\\' as separators), no '..', no drive letter, none of <>\"|?*, no trailing dot or space and no Windows device names (CON, NUL, COM1...)".into());
            }
        }
        let fonts = u.get("default_font").into_iter().chain(u.get("fallback_fonts").and_then(|f| f.as_array()).into_iter().flatten());
        for f in fonts.filter_map(|f| f.as_str()) {
            if !crate::docext::valid_font_name(f) {
                out.push(format!("$.unity: the font name {:?} must not contain '/', '\', ':' or '..'", f));
            }
        }
        if let Some(s) = u.get("assembly").and_then(|s| s.as_str()) {
            if !crate::docext::valid_assembly(s) {
                out.push("$.unity.assembly: must look like an identifier (letters, digits, '_', '.', '-')".into());
            }
        }
    }
    if let Some(ov) = v.get("portrait").and_then(|p| p.get("overrides")).and_then(|o| o.as_array()) {
        let mut seen = std::collections::HashSet::new();
        for (i, o) in ov.iter().enumerate() {
            if let Some(id) = o.get("id").and_then(|i| i.as_u64()) {
                if !ids.contains(&id) {
                    out.push(format!("$.portrait.overrides[{}].id: no element has id {}", i, id));
                } else if !seen.insert(id) {
                    // the loader keeps the first one and drops the rest
                    out.push(format!("$.portrait.overrides[{}].id: duplicate override for element {} (only the first one is used)", i, id));
                }
            }
        }
    }
}

/// `--validate <file>`: prints the errors and returns the exit code (0 valid, 1 invalid or unreadable).
pub fn validate_file(path: &str) -> i32 {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{}: {}", path, e);
            return 1;
        }
    };
    let t = text.strip_prefix('\u{feff}').unwrap_or(&text);
    let v: Value = match serde_json::from_str(t) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("$: not valid JSON ({})", e);
            return 1;
        }
    };
    let errors = validate_value(&v);
    if errors.is_empty() {
        println!("{}: valid", path);
        0
    } else {
        for e in &errors {
            eprintln!("{}", e);
        }
        eprintln!("{}: {} error(s)", path, errors.len());
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn schema_is_valid_json_and_lists_every_field() {
        let s: Value = serde_json::from_str(SCHEMA).expect("schema parses");
        let doc = serde_json::to_value(crate::model::Doc::empty(100.0, 100.0)).unwrap();
        for k in doc.as_object().unwrap().keys() {
            assert!(s["properties"].get(k).is_some(), "Doc field {} missing from the schema", k);
        }
        let mut e = crate::model::Element::new(1, crate::model::Kind::Button);
        e.unity_name = Some("A".into());
        e.slice = Some([1.0; 4]);
        e.ppu = Some(1.0);
        e.sprite = Some("s".into());
        e.image_mode = Some(crate::docext::ImageMode::Tiled);
        e.preserve_aspect = true;
        let ev = serde_json::to_value(&e).unwrap();
        for k in ev.as_object().unwrap().keys() {
            assert!(s["$defs"]["Element"]["properties"].get(k).is_some(), "Element field {} missing from the schema", k);
        }
    }

    #[test]
    fn bundled_designs_and_templates_are_valid() {
        for t in 0..crate::model::TEMPLATES.len() {
            let v = serde_json::to_value(crate::model::template(t, 1920.0, 1080.0)).unwrap();
            assert_eq!(validate_value(&v), Vec::<String>::new(), "template {}", t);
        }
        for f in std::fs::read_dir("designs").unwrap().flatten() {
            if f.path().extension().is_some_and(|e| e == "json") {
                let v: Value = serde_json::from_str(crate::library::strip_bom(&std::fs::read_to_string(f.path()).unwrap())).unwrap();
                assert_eq!(validate_value(&v), Vec::<String>::new(), "{:?}", f.path());
            }
        }
    }

    #[test]
    fn errors_carry_json_paths() {
        let v = json!({"elements":[{"id":1,"fill":[1,2,300,255],"kind":"Nope"},{"id":1,"w":"wide","unity_name":"a/b"}]});
        let e = validate_value(&v);
        assert!(e.iter().any(|m| m.starts_with("$.elements[0].fill[2]")), "{:?}", e);
        assert!(e.iter().any(|m| m.starts_with("$.elements[0].kind")), "{:?}", e);
        assert!(e.iter().any(|m| m.starts_with("$.elements[1].w: expected number")), "{:?}", e);
        let v = json!({"elements":[{"id":1},{"id":1,"unity_name":"a/b"}],"unity":{"scripts_dir":"../x"}});
        let e = validate_value(&v);
        assert!(e.iter().any(|m| m.starts_with("$.elements[1].id: duplicate")), "{:?}", e);
        assert!(e.iter().any(|m| m.starts_with("$.elements[1].unity_name")), "{:?}", e);
        assert!(e.iter().any(|m| m.starts_with("$.unity.scripts_dir")), "{:?}", e);
    }

    #[test]
    fn validate_agrees_with_the_loader_on_scripts_dir_and_duplicate_overrides() {
        let base = |dir: &str, ovs: Value| json!({"elements":[{"id":1,"name":"a","kind":"Panel","w":10,"h":10}],"unity":{"scripts_dir":dir},"portrait":{"width":10,"height":20,"overrides":ovs}});
        // backslashes and a trailing slash are fine: the loader normalises them
        assert!(validate_value(&base(r"Assets\Game\UI\", json!([]))).is_empty());
        let d: crate::model::Doc = serde_json::from_value(base(r"Assets\Game\UI", json!([]))).unwrap();
        let mut d = d;
        d.sanitize();
        assert_eq!(d.unity.scripts_dir.as_deref(), Some("Assets/Game/UI"));
        // still invalid when the loader would drop it
        assert!(!validate_value(&base(r"..\x", json!([]))).is_empty());
        assert!(!validate_value(&base(r"Assets\CON", json!([]))).is_empty());
        // two overrides for one element: the loader keeps the first, validation reports it
        let e = validate_value(&base("Assets/X", json!([{"id":1,"x":5},{"id":1,"x":9}])));
        assert!(e.iter().any(|m| m.starts_with("$.portrait.overrides[1].id: duplicate")), "{:?}", e);
        let mut d: crate::model::Doc = serde_json::from_value(base("Assets/X", json!([{"id":1,"x":5},{"id":1,"x":9}]))).unwrap();
        d.sanitize();
        assert_eq!(d.portrait.unwrap().overrides[0].x, Some(5.0));
    }

    #[test]
    fn minimal_file_is_valid() {
        let v = json!({"elements":[{"id":1,"name":"a","kind":"Panel","x":0,"y":0,"w":10,"h":10}]});
        assert!(validate_value(&v).is_empty());
        assert!(!validate_value(&json!([1])).is_empty());
    }
}
