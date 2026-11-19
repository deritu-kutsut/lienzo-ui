//! Unity -> Lienzo import: reads a uGUI prefab (`.prefab`, Unity text YAML) or Lienzo's own exported
//! `Designs/*.json` and builds a [`Doc`] (the `.editorgui.json` model), with a list of warnings for
//! everything that could not be mapped. See docs/UNITY_IMPORT.md.
//!
//! The YAML reader is hand-rolled, safe and bounded: files above 20 MB, more than 100 000 documents,
//! nesting deeper than 48 levels, strings above 64 KB and more than 5 000 elements are refused or cut.
//! Nothing in a hostile file can make it panic or loop.

#![allow(dead_code)] // public API used by the editor UI wired on top of this module

use crate::docext::{CanvasRenderMode, ElementOverride, ImageMode, PortraitLayout, clean_label, valid_unity_name};
use crate::model::{Doc, Element, Frame, Kind, LayoutGroup, LayoutMode, RectTransform, Rgba, apply_rect_transform, norm_angle};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Result of an import: the design and what could not be mapped.
pub struct Imported {
    pub doc: Doc,
    pub warnings: Vec<String>,
}

pub const MAX_FILE_BYTES: u64 = 20 * 1024 * 1024;
const MAX_DOCS: usize = 100_000;
const MAX_DEPTH: usize = 48;
const MAX_STR: usize = 65_536;
const MAX_ELEMENTS: usize = 5_000;
const MAX_NEST: usize = 128;
const MAX_META_BYTES: u64 = 8 * 1024 * 1024;
const MAX_META_FILES: usize = 400_000;
const MAX_PREFABS: usize = 50_000;
const MAX_WARNINGS: usize = 400;
const MAX_ITEMS: usize = 100_000;

// ===========================================================================
// 1. Unity YAML reader
// ===========================================================================

/// A parsed YAML value (only what Unity writes: block / flow maps and sequences, plain and quoted scalars).
#[derive(Clone, Debug)]
pub(crate) enum Y {
    Null,
    S(String),
    Seq(Vec<Y>),
    Map(Vec<(String, Y)>),
}

impl Y {
    pub(crate) fn get(&self, k: &str) -> Option<&Y> {
        match self {
            Y::Map(m) => m.iter().find(|(kk, _)| kk == k).map(|(_, v)| v),
            _ => None,
        }
    }
    pub(crate) fn s(&self) -> Option<&str> {
        match self {
            Y::S(s) => Some(s),
            _ => None,
        }
    }
    pub(crate) fn seq(&self) -> &[Y] {
        match self {
            Y::Seq(v) => v,
            _ => &[],
        }
    }
    fn first_value(&self) -> Option<&Y> {
        match self {
            Y::Map(m) => m.first().map(|(_, v)| v),
            _ => None,
        }
    }
}

fn parse_f64(s: &str) -> Option<f64> {
    s.trim().parse::<f64>().ok().filter(|v| v.is_finite())
}

fn nf(y: &Y, k: &str) -> Option<f32> {
    y.get(k)?.s().and_then(parse_f64).map(|v| v.clamp(-1.0e9, 1.0e9) as f32)
}
fn ni(y: &Y, k: &str) -> Option<i64> {
    let s = y.get(k)?.s()?.trim();
    s.parse::<i64>().ok().or_else(|| parse_f64(s).map(|v| v as i64))
}
fn nb(y: &Y, k: &str) -> Option<bool> {
    match y.get(k)?.s()?.trim() {
        "1" | "true" | "True" => Some(true),
        "0" | "false" | "False" => Some(false),
        _ => None,
    }
}
fn ns<'a>(y: &'a Y, k: &str) -> Option<&'a str> {
    y.get(k)?.s()
}
/// `{fileID: n, guid: g, type: t}` -> n
fn file_id_of(y: &Y) -> i64 {
    y.get("fileID").and_then(|v| v.s()).and_then(|s| s.trim().parse::<i64>().ok()).unwrap_or(0)
}
fn rid(y: &Y, k: &str) -> i64 {
    y.get(k).map(file_id_of).unwrap_or(0)
}
fn guid_of(y: &Y) -> Option<&str> {
    y.get("guid")?.s().map(|s| s.trim()).filter(|s| !s.is_empty())
}
fn v2(y: &Y, k: &str) -> Option<[f32; 2]> {
    let m = y.get(k)?;
    Some([nf(m, "x")?, nf(m, "y")?])
}
fn v4(y: &Y, k: &str) -> Option<[f32; 4]> {
    let m = y.get(k)?;
    Some([nf(m, "x")?, nf(m, "y")?, nf(m, "z")?, nf(m, "w")?])
}
fn color_of(y: &Y, k: &str) -> Option<Rgba> {
    let m = y.get(k)?;
    let c = |n: &str| nf(m, n).map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8);
    Some([c("r")?, c("g")?, c("b")?, c("a").unwrap_or(255)])
}

type Line<'a> = (usize, &'a str);

/// Non-empty, non-comment lines as (indent, trimmed text).
fn lines_of(text: &str) -> Vec<Line<'_>> {
    text.split('\n')
        .filter_map(|l| {
            let l = l.trim_end_matches('\r');
            let t = l.trim_start_matches([' ', '\t']);
            if t.is_empty() || t.starts_with('#') {
                return None;
            }
            Some((l.len() - t.len(), t.trim_end()))
        })
        .collect()
}

fn is_seq_item(t: &str) -> bool {
    t == "-" || t.starts_with("- ")
}

/// Splits `key: value` (value may be empty). `None` if the text is not a map entry.
fn key_split(t: &str) -> Option<(String, &str)> {
    if t.starts_with('{') || t.starts_with('[') || is_seq_item(t) {
        return None;
    }
    let bytes = t.as_bytes();
    if bytes[0] == b'"' || bytes[0] == b'\'' {
        let q = bytes[0] as char;
        let end = close_index(t, q)?;
        let rest = t[end + 1..].trim_start();
        let rest = rest.strip_prefix(':')?;
        if !rest.is_empty() && !rest.starts_with(' ') {
            return None;
        }
        return Some((decode_quoted(&t[1..end], q), rest.trim_start()));
    }
    let mut i = 0;
    while let Some(p) = t[i..].find(':') {
        let at = i + p;
        let after = &t[at + 1..];
        if after.is_empty() || after.starts_with(' ') {
            let key: String = t[..at].trim_end().chars().take(256).collect();
            return Some((key, after.trim_start()));
        }
        i = at + 1;
    }
    None
}

/// Byte index of the closing quote of a quoted scalar that starts at index 0 of `s`.
fn close_index(s: &str, q: char) -> Option<usize> {
    let mut it = s.char_indices().skip(1).peekable();
    while let Some((i, c)) = it.next() {
        if q == '"' && c == '\\' {
            it.next();
        } else if c == q {
            if q == '\'' && matches!(it.peek(), Some((_, '\''))) {
                it.next();
            } else {
                return Some(i);
            }
        }
    }
    None
}

fn decode_quoted(inner: &str, q: char) -> String {
    let mut out = String::new();
    let mut it = inner.chars().peekable();
    let mut n = 0usize;
    while let Some(c) = it.next() {
        n += 1;
        if n > MAX_STR {
            break;
        }
        if q == '\'' {
            if c == '\'' && it.peek() == Some(&'\'') {
                it.next();
            }
            out.push(c);
            continue;
        }
        if c != '\\' {
            out.push(c);
            continue;
        }
        match it.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some('0') => out.push('\0'),
            Some('"') => out.push('"'),
            Some('/') => out.push('/'),
            Some(' ') => out.push(' '),
            Some('\\') => out.push('\\'),
            Some(k @ ('x' | 'u' | 'U')) => {
                let len = match k {
                    'x' => 2,
                    'u' => 4,
                    _ => 8,
                };
                let mut v = 0u32;
                let mut got = 0;
                while got < len {
                    match it.peek().and_then(|d| d.to_digit(16)) {
                        Some(d) => {
                            v = v.wrapping_mul(16).wrapping_add(d);
                            it.next();
                            got += 1;
                        }
                        None => break,
                    }
                }
                out.push(char::from_u32(v).unwrap_or('\u{fffd}'));
            }
            Some(other) => out.push(other),
            None => {}
        }
    }
    out
}

struct Parser<'a> {
    lines: Vec<Line<'a>>,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(text: &'a str) -> Self {
        Parser { lines: lines_of(text), pos: 0 }
    }

    fn node(&mut self, min: usize, depth: usize) -> Y {
        let Some(&(ind, t)) = self.lines.get(self.pos) else { return Y::Null };
        if ind < min {
            return Y::Null;
        }
        if depth > MAX_DEPTH {
            while self.lines.get(self.pos).is_some_and(|l| l.0 >= min) {
                self.pos += 1;
            }
            return Y::Null;
        }
        if is_seq_item(t) {
            self.seq(ind, depth)
        } else if key_split(t).is_some() {
            self.map(ind, depth)
        } else {
            self.pos += 1;
            self.value_from(t, ind, depth)
        }
    }

    fn map(&mut self, ind: usize, depth: usize) -> Y {
        let mut out: Vec<(String, Y)> = Vec::new();
        while let Some(&(i, t)) = self.lines.get(self.pos) {
            if i < ind {
                break;
            }
            if i > ind {
                self.pos += 1; // tolerate junk
                continue;
            }
            if is_seq_item(t) {
                break;
            }
            let Some((k, rest)) = key_split(t) else {
                self.pos += 1;
                continue;
            };
            self.pos += 1;
            let v = if rest.is_empty() {
                match self.lines.get(self.pos) {
                    Some(&(nind, _)) if nind > ind => self.node(ind + 1, depth + 1),
                    Some(&(nind, nt)) if nind == ind && is_seq_item(nt) => self.seq(ind, depth + 1),
                    _ => Y::Null,
                }
            } else {
                self.value_from(rest, ind + 1, depth + 1)
            };
            if out.len() < MAX_ITEMS {
                out.push((k, v));
            }
        }
        Y::Map(out)
    }

    fn seq(&mut self, ind: usize, depth: usize) -> Y {
        let mut out = Vec::new();
        while let Some(&(i, t)) = self.lines.get(self.pos) {
            if i != ind || !is_seq_item(t) {
                break;
            }
            let rest = t[1..].trim_start();
            let off = t.len() - rest.len();
            let item = if rest.is_empty() {
                self.pos += 1;
                match self.lines.get(self.pos) {
                    Some(&(nind, _)) if nind > ind => self.node(ind + 1, depth + 1),
                    _ => Y::Null,
                }
            } else if is_seq_item(rest) || key_split(rest).is_some() {
                // "- key: v" / "- - x": the rest of the line starts a nested block at its own column
                self.lines[self.pos] = (ind + off, rest);
                self.node(ind + 1, depth + 1)
            } else {
                self.pos += 1;
                self.value_from(rest, ind + 1, depth + 1)
            };
            if out.len() < MAX_ITEMS {
                out.push(item);
            }
        }
        Y::Seq(out)
    }

    /// A value that starts on the current line (`text`); continuation lines have an indent >= `cont_min`.
    fn value_from(&mut self, text: &str, cont_min: usize, depth: usize) -> Y {
        let first = text.chars().next().unwrap_or(' ');
        match first {
            '{' | '[' => {
                let mut buf = text.to_string();
                let mut guard = 0;
                while !flow_balanced(&buf) && guard < 100_000 && buf.len() < MAX_STR * 16 {
                    let Some(&(_, l)) = self.lines.get(self.pos) else { break };
                    buf.push(' ');
                    buf.push_str(l);
                    self.pos += 1;
                    guard += 1;
                }
                let chars: Vec<char> = buf.chars().collect();
                let mut i = 0;
                flow(&chars, &mut i, depth)
            }
            '"' | '\'' => {
                let mut buf = text.to_string();
                loop {
                    if let Some(end) = close_index(&buf, first) {
                        return Y::S(decode_quoted(&buf[1..end], first));
                    }
                    match self.lines.get(self.pos) {
                        Some(&(_, l)) if buf.len() < MAX_STR * 4 => {
                            if first == '"' && buf.ends_with('\\') {
                                buf.pop();
                            } else {
                                buf.push(' ');
                            }
                            buf.push_str(l);
                            self.pos += 1;
                        }
                        _ => return Y::S(decode_quoted(&buf[1..], first)),
                    }
                }
            }
            '|' | '>' => {
                let mut buf = String::new();
                while let Some(&(i, l)) = self.lines.get(self.pos) {
                    if i < cont_min || buf.len() > MAX_STR * 2 {
                        break;
                    }
                    if !buf.is_empty() {
                        buf.push(if first == '|' { '\n' } else { ' ' });
                    }
                    buf.push_str(l);
                    self.pos += 1;
                }
                Y::S(buf.chars().take(MAX_STR).collect())
            }
            _ => {
                let mut buf = text.to_string();
                while let Some(&(i, l)) = self.lines.get(self.pos) {
                    if i < cont_min || buf.len() > MAX_STR * 2 {
                        break;
                    }
                    buf.push(' ');
                    buf.push_str(l);
                    self.pos += 1;
                }
                Y::S(buf.chars().take(MAX_STR).collect())
            }
        }
    }
}

fn flow_balanced(s: &str) -> bool {
    let mut depth = 0i64;
    let mut quote: Option<char> = None;
    let mut esc = false;
    for c in s.chars() {
        if let Some(q) = quote {
            if esc {
                esc = false;
            } else if q == '"' && c == '\\' {
                esc = true;
            } else if c == q {
                quote = None;
            }
            continue;
        }
        match c {
            '"' | '\'' => quote = Some(c),
            '{' | '[' => depth += 1,
            '}' | ']' => depth -= 1,
            _ => {}
        }
    }
    depth <= 0 && quote.is_none()
}

fn skip_ws(s: &[char], i: &mut usize) {
    while *i < s.len() && s[*i].is_whitespace() {
        *i += 1;
    }
}

fn flow_quoted(s: &[char], i: &mut usize) -> String {
    let q = s[*i];
    let start = *i + 1;
    let mut j = start;
    while j < s.len() {
        if q == '"' && s[j] == '\\' {
            j += 2;
            continue;
        }
        if s[j] == q {
            if q == '\'' && s.get(j + 1) == Some(&'\'') {
                j += 2;
                continue;
            }
            break;
        }
        j += 1;
    }
    let end = j.min(s.len());
    let inner: String = s[start.min(end)..end].iter().collect();
    *i = (j + 1).min(s.len());
    decode_quoted(&inner, q)
}

fn flow_plain(s: &[char], i: &mut usize, stops: &[char]) -> String {
    let start = *i;
    while *i < s.len() && !stops.contains(&s[*i]) {
        *i += 1;
    }
    let t: String = s[start..*i].iter().collect();
    t.trim().chars().take(MAX_STR).collect()
}

fn flow(s: &[char], i: &mut usize, depth: usize) -> Y {
    skip_ws(s, i);
    if depth > MAX_DEPTH || *i >= s.len() {
        // too deep: consume the rest as one blob so the caller terminates
        *i = s.len();
        return Y::Null;
    }
    match s[*i] {
        '{' => {
            *i += 1;
            let mut out = Vec::new();
            loop {
                skip_ws(s, i);
                if *i >= s.len() {
                    break;
                }
                if s[*i] == '}' {
                    *i += 1;
                    break;
                }
                let before = *i;
                let key = if s[*i] == '"' || s[*i] == '\'' { flow_quoted(s, i) } else { flow_plain(s, i, &[':', ',', '}']) };
                skip_ws(s, i);
                let val = if *i < s.len() && s[*i] == ':' {
                    *i += 1;
                    flow(s, i, depth + 1)
                } else {
                    Y::Null
                };
                if out.len() < MAX_ITEMS {
                    out.push((key, val));
                }
                skip_ws(s, i);
                if *i < s.len() && s[*i] == ',' {
                    *i += 1;
                }
                if *i == before {
                    *i += 1; // no progress: skip a char
                }
            }
            Y::Map(out)
        }
        '[' => {
            *i += 1;
            let mut out = Vec::new();
            loop {
                skip_ws(s, i);
                if *i >= s.len() {
                    break;
                }
                if s[*i] == ']' {
                    *i += 1;
                    break;
                }
                let before = *i;
                let v = flow(s, i, depth + 1);
                if out.len() < MAX_ITEMS {
                    out.push(v);
                }
                skip_ws(s, i);
                if *i < s.len() && s[*i] == ',' {
                    *i += 1;
                }
                if *i == before {
                    *i += 1;
                }
            }
            Y::Seq(out)
        }
        '"' | '\'' => Y::S(flow_quoted(s, i)),
        _ => Y::S(flow_plain(s, i, &[',', '}', ']'])),
    }
}

/// Parses a plain YAML text (a `.meta` file): one document.
pub(crate) fn parse_plain(text: &str) -> Y {
    let mut p = Parser::new(text);
    p.node(0, 0)
}

struct RawDoc<'a> {
    class: u32,
    file: i64,
    stripped: bool,
    body: &'a str,
}

fn split_docs(text: &str) -> Result<Vec<RawDoc<'_>>, String> {
    let mut starts: Vec<usize> = Vec::new();
    let mut off = 0usize;
    for line in text.split_inclusive('\n') {
        if line.starts_with("--- ") || line.trim_end() == "---" {
            if starts.len() >= MAX_DOCS {
                return Err(format!("too many YAML documents (more than {})", MAX_DOCS));
            }
            starts.push(off);
        }
        off += line.len();
    }
    let mut out = Vec::with_capacity(starts.len());
    for (n, &s) in starts.iter().enumerate() {
        let end = starts.get(n + 1).copied().unwrap_or(text.len());
        let chunk = &text[s..end];
        let (head, body) = chunk.split_once('\n').unwrap_or((chunk, ""));
        let mut class = None;
        let mut file = None;
        let mut stripped = false;
        for tok in head.split_whitespace().skip(1) {
            if let Some(c) = tok.strip_prefix("!u!") {
                class = c.parse::<u32>().ok();
            } else if let Some(f) = tok.strip_prefix('&') {
                file = f.parse::<i64>().ok();
            } else if tok == "stripped" {
                stripped = true;
            }
        }
        if let (Some(class), Some(file)) = (class, file) {
            out.push(RawDoc { class, file, stripped, body });
        }
    }
    Ok(out)
}

// ===========================================================================
// 2. Scene: parsed objects of one prefab
// ===========================================================================

struct Obj {
    class: u32,
    /// The inner mapping (`GameObject: {..}` -> `{..}`); `Y::Null` for classes that are not parsed.
    y: Y,
    /// GameObject that owns this component (the GameObject itself for class 1).
    go: i64,
}

struct Scene {
    objs: HashMap<i64, Obj>,
    order: Vec<i64>,
    duplicates: usize,
    instances: usize,
}

const PARSED_CLASSES: [u32; 6] = [1, 4, 223, 224, 225, 114];

fn quick_go(body: &str) -> i64 {
    for l in body.lines().take(40) {
        let t = l.trim_start();
        if let Some(r) = t.strip_prefix("m_GameObject:") {
            if let Some(p) = r.find("fileID:") {
                let n: String = r[p + 7..].trim_start().chars().take_while(|c| c.is_ascii_digit() || *c == '-').collect();
                return n.parse().unwrap_or(0);
            }
        }
    }
    0
}

fn load_scene(text: &str) -> Result<Scene, String> {
    let docs = split_docs(text)?;
    let mut sc = Scene { objs: HashMap::new(), order: Vec::new(), duplicates: 0, instances: 0 };
    for d in docs {
        if d.class == 1001 {
            sc.instances += 1;
            continue;
        }
        if d.stripped {
            sc.instances += 1;
            continue;
        }
        if sc.objs.contains_key(&d.file) {
            sc.duplicates += 1;
            continue;
        }
        let (y, go) = if PARSED_CLASSES.contains(&d.class) {
            let mut p = Parser::new(d.body);
            let root = p.node(0, 0);
            let inner = root.first_value().cloned().unwrap_or(Y::Null);
            let go = if d.class == 1 { d.file } else { rid(&inner, "m_GameObject") };
            (inner, go)
        } else {
            (Y::Null, quick_go(d.body))
        };
        sc.order.push(d.file);
        sc.objs.insert(d.file, Obj { class: d.class, y, go });
    }
    Ok(sc)
}

impl Scene {
    fn is_transform(&self, id: i64) -> bool {
        self.objs.get(&id).is_some_and(|o| o.class == 224 || o.class == 4)
    }
    fn go_name(&self, go: i64) -> String {
        self.objs.get(&go).and_then(|o| ns(&o.y, "m_Name")).unwrap_or("").to_string()
    }
    /// Component ids of a GameObject, in inspector order.
    fn comp_ids(&self, go: i64) -> Vec<i64> {
        let mut v: Vec<i64> = Vec::new();
        let mut seen: HashSet<i64> = HashSet::new();
        if let Some(o) = self.objs.get(&go) {
            for c in o.y.get("m_Component").map(|y| y.seq()).unwrap_or(&[]) {
                let id = c.get("component").map(file_id_of).unwrap_or_else(|| file_id_of(c));
                if self.objs.get(&id).is_some_and(|c| c.go == go) && seen.insert(id) {
                    v.push(id);
                }
            }
        }
        v
    }
    fn rt_of_go(&self, go: i64) -> Option<i64> {
        self.comp_ids(go).into_iter().find(|c| self.is_transform(*c))
    }
    fn father(&self, rt: i64) -> i64 {
        self.objs.get(&rt).map(|o| rid(&o.y, "m_Father")).unwrap_or(0)
    }
    /// Children (RectTransform ids) in sibling order: `m_Children` first, then any transform that names this one as father.
    fn kids(&self, rt: i64, by_father: &HashMap<i64, Vec<i64>>) -> Vec<i64> {
        let mut out: Vec<i64> = Vec::new();
        let mut seen: HashSet<i64> = HashSet::new();
        if let Some(o) = self.objs.get(&rt) {
            for c in o.y.get("m_Children").map(|y| y.seq()).unwrap_or(&[]) {
                let id = file_id_of(c);
                if id != rt && self.is_transform(id) && seen.insert(id) {
                    out.push(id);
                }
            }
        }
        if let Some(extra) = by_father.get(&rt) {
            for id in extra {
                if *id != rt && seen.insert(*id) {
                    out.push(*id);
                }
            }
        }
        out
    }
    /// Direct child of `parent` whose subtree contains `target` (or `target` itself if it is a direct child).
    fn child_containing(&self, parent: i64, target: i64) -> Option<i64> {
        let mut cur = target;
        for _ in 0..256 {
            if cur == 0 || !self.is_transform(cur) {
                return None;
            }
            let f = self.father(cur);
            if f == parent {
                return Some(cur);
            }
            cur = f;
        }
        None
    }
    /// RectTransform id of the GameObject that owns the component `comp_id` (or `comp_id` itself if it is a transform).
    fn rt_of_ref(&self, id: i64) -> Option<i64> {
        if id == 0 {
            return None;
        }
        if self.is_transform(id) {
            return Some(id);
        }
        let go = self.objs.get(&id)?.go;
        self.rt_of_go(go)
    }
}

// ===========================================================================
// 3. Component identification
// ===========================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CK {
    Canvas,
    CanvasScaler,
    CanvasGroup,
    Ignored,
    Image,
    RawImage,
    Text,
    Tmp,
    Button,
    Toggle,
    Slider,
    Scrollbar,
    InputField,
    Dropdown,
    ScrollRect,
    Mask,
    RectMask2D,
    Hlg,
    Vlg,
    HvUnknown,
    Grid,
    Csf,
    LayoutElement,
    Outline,
    Shadow,
    ShadowUnknown,
    Selectable,
    Other,
}

struct Comp<'a> {
    id: i64,
    kind: CK,
    y: &'a Y,
    enabled: bool,
    name: String,
}

/// Well-known GUIDs of the Unity UI / TextMeshPro scripts (used when `m_EditorClassIdentifier` is empty).
const KNOWN_GUIDS: &[(&str, CK)] = &[
    ("fe87c0e1cc204ed48ad3b37840f39efc", CK::Image),
    ("1344c3c82d62a2a41a3576d8abb8e3ea", CK::RawImage),
    ("5f7201a12d95ffc409449d95f23cf332", CK::Text),
    ("f4688fdb7df04437aeb418b961361dc5", CK::Tmp),
    ("4e29b1a8efbd4b44bb3f3716e73f07ff", CK::Button),
    ("9085046f02f69544eb97fd06b6048fe2", CK::Toggle),
    ("67db9e8f0e2ae9c40bc1e2b64352a6b4", CK::Slider),
    ("2a4db7a114972834c8e4117be1d82ba3", CK::Scrollbar),
    ("d199490a83bb2b844b9695cbf13b01ef", CK::InputField),
    ("2da0c512f12947e489f739169773d7ca", CK::InputField),
    ("0d0b652f32a2cc243917e4028fa0f046", CK::Dropdown),
    ("7b743370ac3e4ec2a1668f5455a8ef8a", CK::Dropdown),
    ("1aa08ab6e0800fa44ae55d278d1423e3", CK::ScrollRect),
    ("31a19414c41e5ae4aae2af33fee712f6", CK::Mask),
    ("3312d7739989d2b4e91e6319e9a96d76", CK::RectMask2D),
    ("0cd44c1031e13a943bb63640046fad76", CK::CanvasScaler),
    ("dc42784cf147c0c48a680349fa168899", CK::Ignored),
    ("30649d3a9faa99c48a7b1166b86bf2a0", CK::Hlg),
    ("59f8146938fff824cb5fd77236b75775", CK::Vlg),
    ("8a8695521f0d02e499659fee002a26c2", CK::Grid),
    ("3245ec927659c4140ac4f8d17403cc18", CK::Csf),
    ("306cc8c2b49d7114eaa3623786fc2126", CK::LayoutElement),
    ("e19747de3f5aca642ab2be37e372fb86", CK::Outline),
    ("cfabb0440166ab443bba8876756fdfa9", CK::Shadow),
    ("76c392e42b5098c458856cdf6ecaaaa1", CK::Ignored), // EventSystem
    ("4f231c4fb786f3946a6b90b886c48677", CK::Ignored), // StandaloneInputModule
];

fn kind_from_type_name(full: &str) -> Option<CK> {
    let n = full.rsplit("::").next().unwrap_or(full).trim();
    Some(match n {
        "UnityEngine.UI.Image" => CK::Image,
        "UnityEngine.UI.RawImage" => CK::RawImage,
        "UnityEngine.UI.Text" => CK::Text,
        "TMPro.TextMeshProUGUI" => CK::Tmp,
        "UnityEngine.UI.Button" => CK::Button,
        "UnityEngine.UI.Toggle" => CK::Toggle,
        "UnityEngine.UI.Slider" => CK::Slider,
        "UnityEngine.UI.Scrollbar" => CK::Scrollbar,
        "UnityEngine.UI.InputField" | "TMPro.TMP_InputField" => CK::InputField,
        "UnityEngine.UI.Dropdown" | "TMPro.TMP_Dropdown" => CK::Dropdown,
        "UnityEngine.UI.ScrollRect" => CK::ScrollRect,
        "UnityEngine.UI.Mask" => CK::Mask,
        "UnityEngine.UI.RectMask2D" => CK::RectMask2D,
        "UnityEngine.UI.CanvasScaler" => CK::CanvasScaler,
        "UnityEngine.UI.GraphicRaycaster" | "UnityEngine.EventSystems.EventSystem" | "UnityEngine.EventSystems.StandaloneInputModule" | "UnityEngine.EventSystems.InputSystemUIInputModule" | "UnityEngine.EventSystems.PhysicsRaycaster" => CK::Ignored,
        "UnityEngine.UI.HorizontalLayoutGroup" => CK::Hlg,
        "UnityEngine.UI.VerticalLayoutGroup" => CK::Vlg,
        "UnityEngine.UI.GridLayoutGroup" => CK::Grid,
        "UnityEngine.UI.ContentSizeFitter" => CK::Csf,
        "UnityEngine.UI.LayoutElement" => CK::LayoutElement,
        "UnityEngine.UI.Outline" => CK::Outline,
        "UnityEngine.UI.Shadow" => CK::Shadow,
        _ => return None,
    })
}

fn classify_mb(y: &Y) -> CK {
    if let Some(id) = ns(y, "m_EditorClassIdentifier") {
        if let Some(k) = kind_from_type_name(id) {
            return k;
        }
    }
    if let Some(g) = y.get("m_Script").and_then(guid_of) {
        if let Some((_, k)) = KNOWN_GUIDS.iter().find(|(kg, _)| *kg == g) {
            return *k;
        }
    }
    let has = |k: &str| y.get(k).is_some();
    if has("m_UiScaleMode") && has("m_ReferenceResolution") {
        CK::CanvasScaler
    } else if has("m_FontData") && has("m_Text") {
        CK::Text
    } else if has("m_text") && (has("m_fontAsset") || has("m_fontSize")) {
        CK::Tmp
    } else if has("m_TextComponent") {
        CK::InputField
    } else if has("m_Options") && has("m_Template") {
        CK::Dropdown
    } else if has("m_FillRect") || (has("m_HandleRect") && has("m_MinValue")) {
        CK::Slider
    } else if has("m_HandleRect") && has("m_Size") {
        CK::Scrollbar
    } else if has("m_IsOn") {
        CK::Toggle
    } else if has("m_Content") && has("m_Viewport") {
        CK::ScrollRect
    } else if has("m_ShowMaskGraphic") {
        CK::Mask
    } else if has("m_Softness") && has("m_Padding") {
        CK::RectMask2D
    } else if has("m_CellSize") {
        CK::Grid
    } else if has("m_ChildControlWidth") && has("m_Spacing") {
        CK::HvUnknown
    } else if has("m_HorizontalFit") || has("m_VerticalFit") {
        CK::Csf
    } else if has("m_IgnoreLayout") || has("m_PreferredWidth") {
        CK::LayoutElement
    } else if has("m_EffectColor") && has("m_EffectDistance") {
        CK::ShadowUnknown
    } else if has("m_Sprite") && (has("m_FillCenter") || has("m_Type")) {
        CK::Image
    } else if has("m_Texture") && has("m_UVRect") {
        CK::RawImage
    } else if has("m_BlockingObjects") || has("m_IgnoreReversedGraphics") {
        CK::Ignored
    } else if has("m_OnClick") && has("m_Transition") {
        CK::Button
    } else if has("m_Transition") {
        CK::Selectable
    } else {
        CK::Other
    }
}

fn native_class_name(class: u32) -> String {
    match class {
        20 => "Camera",
        23 => "MeshRenderer",
        33 => "MeshFilter",
        54 => "Rigidbody",
        64 => "MeshCollider",
        65 => "BoxCollider",
        81 => "AudioListener",
        82 => "AudioSource",
        95 => "Animator",
        108 => "Light",
        111 => "Animation",
        135 => "SphereCollider",
        136 => "CapsuleCollider",
        198 => "ParticleSystem",
        199 => "ParticleSystemRenderer",
        212 => "SpriteRenderer",
        225 => "CanvasGroup",
        320 => "PlayableDirector",
        _ => return format!("component class {}", class),
    }
    .to_string()
}

fn script_display_name(y: &Y, assets: &mut Assets) -> String {
    if let Some(id) = ns(y, "m_EditorClassIdentifier") {
        let n = id.rsplit("::").next().unwrap_or(id).trim();
        if !n.is_empty() {
            return n.to_string();
        }
    }
    if let Some(g) = y.get("m_Script").and_then(guid_of) {
        if let Some(stem) = assets.stem(g) {
            return stem;
        }
        return format!("script {}", g);
    }
    "script".to_string()
}

fn comps_of<'a>(sc: &'a Scene, go: i64, assets: &mut Assets) -> Vec<Comp<'a>> {
    let mut out = Vec::new();
    for id in sc.comp_ids(go) {
        let Some(o) = sc.objs.get(&id) else { continue };
        let enabled = nb(&o.y, "m_Enabled").unwrap_or(true);
        let (kind, name) = match o.class {
            4 | 224 => continue,
            222 => (CK::Ignored, String::new()),
            223 => (CK::Canvas, "Canvas".to_string()),
            225 => (CK::CanvasGroup, "CanvasGroup".to_string()),
            114 => {
                let k = classify_mb(&o.y);
                let name = if k == CK::Other || k == CK::Selectable { script_display_name(&o.y, assets) } else { String::new() };
                (k, name)
            }
            c => (CK::Other, native_class_name(c)),
        };
        out.push(Comp { id, kind, y: &o.y, enabled, name });
    }
    out
}

fn find<'a, 'b>(comps: &'b [Comp<'a>], kinds: &[CK]) -> Option<&'b Comp<'a>> {
    comps.iter().find(|c| kinds.contains(&c.kind))
}

// ===========================================================================
// 4. Project assets: GUID -> path through the `.meta` files
// ===========================================================================

#[derive(Clone, Debug, Default)]
struct SubSprite {
    name: String,
    id: i64,
    border: [f32; 4],
}

#[derive(Clone, Debug, Default)]
struct TexInfo {
    ppu: Option<f32>,
    border: [f32; 4],
    sprites: Vec<SubSprite>,
}

enum SpriteRes {
    None,
    Builtin,
    Missing,
    Found { path: String, sprite: Option<String>, border: Option<[f32; 4]>, ppu: Option<f32> },
}

/// Index of the project's `.meta` files (built the first time a GUID is looked up).
pub(crate) struct Assets {
    /// (directory, path prefix inside the Unity project)
    roots: Vec<(PathBuf, String)>,
    guids: Option<HashMap<String, String>>,
    base: Option<PathBuf>,
    tex: HashMap<String, Option<TexInfo>>,
}

impl Assets {
    pub(crate) fn none() -> Assets {
        Assets { roots: Vec::new(), guids: None, base: None, tex: HashMap::new() }
    }

    /// Finds the Unity project (the folder that holds `Assets`) or the package folder above `file`.
    pub(crate) fn for_file(file: &Path) -> Assets {
        let mut a = Assets::none();
        let abs = std::fs::canonicalize(file).unwrap_or_else(|_| file.to_path_buf());
        for anc in abs.ancestors().skip(1) {
            if anc.file_name().is_some_and(|n| n == "Assets") {
                if let Some(proj) = anc.parent() {
                    return Assets::for_project(proj);
                }
            }
        }
        // a prefab inside a package (Library/PackageCache/...): only that package is scanned
        for anc in abs.ancestors().skip(1) {
            if anc.join("package.json").is_file() {
                let name = anc.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                a.roots.push((anc.to_path_buf(), format!("Packages/{}", name)));
                a.base = Some(anc.to_path_buf());
                break;
            }
        }
        a
    }

    pub(crate) fn for_project(proj: &Path) -> Assets {
        let mut a = Assets::none();
        let assets = proj.join("Assets");
        if assets.is_dir() {
            a.roots.push((assets, "Assets".to_string()));
        }
        let packages = proj.join("Packages");
        if packages.is_dir() {
            a.roots.push((packages, "Packages".to_string()));
        }
        a.base = Some(proj.to_path_buf());
        a
    }

    fn build_index(&mut self) {
        let mut map: HashMap<String, String> = HashMap::new();
        let mut files = 0usize;
        for (dir, prefix) in self.roots.clone() {
            let mut stack: Vec<(PathBuf, String, usize)> = vec![(dir, prefix, 0)];
            while let Some((d, rel, depth)) = stack.pop() {
                let Ok(rd) = std::fs::read_dir(&d) else { continue };
                for ent in rd.flatten() {
                    let Ok(ft) = ent.file_type() else { continue };
                    if ft.is_symlink() {
                        continue;
                    }
                    let name = ent.file_name().to_string_lossy().to_string();
                    if ft.is_dir() {
                        if depth < 40 && !name.starts_with('.') && !name.ends_with('~') {
                            stack.push((ent.path(), format!("{}/{}", rel, name), depth + 1));
                        }
                    } else if let Some(stem) = name.strip_suffix(".meta") {
                        files += 1;
                        if files > MAX_META_FILES {
                            self.guids = Some(map);
                            return;
                        }
                        if let Some(g) = read_guid(&ent.path()) {
                            map.entry(g).or_insert_with(|| format!("{}/{}", rel, stem));
                        }
                    }
                }
            }
        }
        self.guids = Some(map);
    }

    pub(crate) fn path_of(&mut self, guid: &str) -> Option<String> {
        if self.roots.is_empty() {
            return None;
        }
        if self.guids.is_none() {
            self.build_index();
        }
        self.guids.as_ref()?.get(guid).cloned()
    }

    /// File name without folders and extension of the asset behind a GUID.
    fn stem(&mut self, guid: &str) -> Option<String> {
        let p = self.path_of(guid)?;
        let file = p.rsplit('/').next()?;
        Some(file.rsplit_once('.').map(|(s, _)| s).unwrap_or(file).to_string())
    }

    fn disk_path(&self, rel: &str) -> Option<PathBuf> {
        for (dir, prefix) in &self.roots {
            if let Some(rest) = rel.strip_prefix(prefix.as_str()) {
                let rest = rest.trim_start_matches('/');
                let mut p = dir.clone();
                for part in rest.split('/').filter(|s| !s.is_empty()) {
                    if part == ".." || part == "." {
                        return None;
                    }
                    p.push(part);
                }
                return Some(p);
            }
        }
        None
    }

    fn texinfo(&mut self, guid: &str, rel: &str) -> Option<TexInfo> {
        if let Some(t) = self.tex.get(guid) {
            return t.clone();
        }
        let info = (|| {
            let mut p = self.disk_path(rel)?.into_os_string();
            p.push(".meta");
            let p = PathBuf::from(p);
            let md = std::fs::metadata(&p).ok()?;
            if md.len() > MAX_META_BYTES {
                return None;
            }
            let text = String::from_utf8_lossy(&std::fs::read(&p).ok()?).to_string();
            Some(parse_tex_meta(&parse_plain(&text)))
        })();
        self.tex.insert(guid.to_string(), info.clone());
        info
    }

    fn resolve_sprite(&mut self, file_id: i64, guid: &str) -> SpriteRes {
        let g = guid.trim();
        if g.is_empty() || g.chars().all(|c| c == '0') {
            return SpriteRes::None;
        }
        if g.starts_with("0000000000000000") {
            return SpriteRes::Builtin;
        }
        let Some(path) = self.path_of(g) else { return SpriteRes::Missing };
        let info = self.texinfo(g, &path).unwrap_or_default();
        let nz = |b: [f32; 4]| if b.iter().any(|v| *v > 0.0) { Some(b) } else { None };
        if file_id != 21300000 && file_id != 0 {
            if let Some(s) = info.sprites.iter().find(|s| s.id == file_id) {
                return SpriteRes::Found { path, sprite: Some(s.name.clone()), border: nz(s.border), ppu: info.ppu };
            }
        }
        let border = nz(info.border).or_else(|| if info.sprites.len() == 1 { nz(info.sprites[0].border) } else { None });
        SpriteRes::Found { path, sprite: None, border, ppu: info.ppu }
    }
}

/// Sprites Lienzo's own Unity importer generates (`Sprites/Generated/rr_<radius>_<stroke>.png`, `ellipse_<ring>.png`, `grad_<RRGGBBAA>_<RRGGBBAA>.png`).
enum Gen {
    Rounded(u32, u32),
    Ellipse(u32),
    Gradient(Rgba, Rgba),
}

fn generated_sprite(path: &str) -> Option<Gen> {
    let (dir, file) = path.rsplit_once('/')?;
    if !dir.ends_with("/Generated") {
        return None;
    }
    let stem = file.rsplit_once('.').map(|(s, _)| s).unwrap_or(file);
    let stem = stem.strip_suffix("_px").unwrap_or(stem);
    let mut it = stem.split('_');
    match it.next()? {
        "rr" => {
            let r = it.next()?.parse::<u32>().ok()?;
            let s = it.next()?.parse::<u32>().ok()?;
            if it.next().is_some() { None } else { Some(Gen::Rounded(r.min(512), s.min(256))) }
        }
        "ellipse" => {
            let n = it.next()?.parse::<u32>().ok()?;
            if it.next().is_some() { None } else { Some(Gen::Ellipse(n)) }
        }
        "grad" => {
            let a = parse_hex(it.next()?)?;
            let b = parse_hex(it.next()?)?;
            if it.next().is_some() { None } else { Some(Gen::Gradient(a, b)) }
        }
        _ => None,
    }
}

fn read_guid(meta: &Path) -> Option<String> {
    use std::io::Read;
    let mut buf = [0u8; 256];
    let n = std::fs::File::open(meta).ok()?.read(&mut buf).ok()?;
    let s = String::from_utf8_lossy(&buf[..n]);
    for l in s.lines().take(6) {
        if let Some(g) = l.strip_prefix("guid:") {
            let g = g.trim();
            if g.len() == 32 && g.chars().all(|c| c.is_ascii_hexdigit()) {
                return Some(g.to_ascii_lowercase());
            }
        }
    }
    None
}

fn parse_tex_meta(root: &Y) -> TexInfo {
    let mut t = TexInfo::default();
    let Some(imp) = root.get("TextureImporter").or_else(|| root.get("PSDImporter")).or_else(|| root.first_value()) else { return t };
    t.ppu = nf(imp, "spritePixelsToUnits").or_else(|| nf(imp, "spritePixelsPerUnit")).filter(|v| *v > 0.0);
    if let Some(b) = v4(imp, "spriteBorder") {
        t.border = b;
    }
    let mut names: HashMap<i64, String> = HashMap::new();
    for e in imp.get("internalIDToNameTable").map(|y| y.seq()).unwrap_or(&[]) {
        if let (Some(first), Some(name)) = (e.get("first"), ns(e, "second")) {
            if let Y::Map(m) = first {
                if let Some(id) = m.first().and_then(|(_, v)| v.s()).and_then(|s| s.trim().parse::<i64>().ok()) {
                    names.insert(id, name.to_string());
                }
            }
        }
    }
    for s in imp.get("spriteSheet").and_then(|y| y.get("sprites")).map(|y| y.seq()).unwrap_or(&[]) {
        let id = ni(s, "internalID").unwrap_or(0);
        let name = ns(s, "name").map(|n| n.to_string()).or_else(|| names.get(&id).cloned()).unwrap_or_default();
        t.sprites.push(SubSprite { name, id, border: v4(s, "border").unwrap_or([0.0; 4]) });
        if t.sprites.len() > 20_000 {
            break;
        }
    }
    for (id, name) in names {
        if !t.sprites.iter().any(|s| s.id == id) && t.sprites.len() < 20_000 {
            t.sprites.push(SubSprite { name, id, border: [0.0; 4] });
        }
    }
    t
}

// ===========================================================================
// 5. Warnings
// ===========================================================================

#[derive(Default)]
struct Warn {
    list: Vec<String>,
    seen: HashSet<String>,
    groups: Vec<(String, String, usize)>,
    index: HashMap<String, usize>,
}

impl Warn {
    fn add(&mut self, s: impl Into<String>) {
        let s = s.into();
        if self.list.len() < MAX_WARNINGS && self.seen.insert(s.clone()) {
            self.list.push(s);
        }
    }
    /// One warning per `key`, naming the first GameObject and counting the others.
    fn group(&mut self, key: &str, go: &str) {
        match self.index.get(key) {
            Some(&i) => self.groups[i].2 += 1,
            None => {
                self.index.insert(key.to_string(), self.groups.len());
                self.groups.push((key.to_string(), go.to_string(), 1));
            }
        }
    }
    fn finish(mut self) -> Vec<String> {
        for (key, first, n) in std::mem::take(&mut self.groups) {
            if self.list.len() >= MAX_WARNINGS {
                break;
            }
            let msg = if n > 1 { format!("{} on '{}' and {} more", key, first, n - 1) } else { format!("{} on '{}'", key, first) };
            self.list.push(msg);
        }
        self.list
    }
}

// ===========================================================================
// 6. uGUI prefab -> Doc
// ===========================================================================

struct Ctx<'a> {
    doc: Doc,
    w: Warn,
    assets: &'a mut Assets,
    visited: HashSet<i64>,
    fonts: HashMap<String, usize>,
    by_father: HashMap<i64, Vec<i64>>,
    builtin_sprites: usize,
    missing_guids: Vec<String>,
    missing_count: usize,
    onclick: usize,
    too_many: bool,
    too_deep: bool,
}

struct Txt {
    text: String,
    size: f32,
    color: Rgba,
    bold: bool,
    italic: bool,
    underline: bool,
    align: u8,
    valign: u8,
    font: Option<String>,
    raycast: bool,
    enabled: bool,
}

fn quat_euler_z(q: [f32; 4]) -> f32 {
    let [x, y, z, w] = q;
    let (x, y, z, w) = (x as f64, y as f64, z as f64, w as f64);
    let deg = (2.0 * (w * z + x * y)).atan2(1.0 - 2.0 * (y * y + z * z)).to_degrees() as f32;
    norm_angle(deg)
}

fn clampv(v: f32, lim: f32) -> f32 {
    if v.is_finite() { v.clamp(-lim, lim) } else { 0.0 }
}

struct RtData {
    rt: RectTransform,
    scale: [f32; 2],
    tilt: bool,
}

fn read_rt(y: &Y) -> RtData {
    let mut rt = RectTransform { anchor_min: [0.5, 0.5], anchor_max: [0.5, 0.5], pivot: [0.5, 0.5], anchored_position: [0.0, 0.0], size_delta: [0.0, 0.0], rotation_z: 0.0 };
    let fix = |v: [f32; 2], lim: f32| [clampv(v[0], lim), clampv(v[1], lim)];
    if let Some(v) = v2(y, "m_AnchorMin") {
        rt.anchor_min = fix(v, 100.0);
    }
    if let Some(v) = v2(y, "m_AnchorMax") {
        rt.anchor_max = fix(v, 100.0);
    }
    if let Some(v) = v2(y, "m_Pivot") {
        rt.pivot = fix(v, 100.0);
    }
    if let Some(v) = v2(y, "m_AnchoredPosition") {
        rt.anchored_position = fix(v, 1.0e6);
    }
    if let Some(v) = v2(y, "m_SizeDelta") {
        rt.size_delta = fix(v, 1.0e6);
    }
    let mut tilt = false;
    if let Some(q) = v4(y, "m_LocalRotation") {
        let q = q.map(|v| clampv(v, 2.0));
        rt.rotation_z = quat_euler_z(q);
        tilt = q[0].abs() > 1e-3 || q[1].abs() > 1e-3;
    }
    let mut scale = [1.0f32, 1.0];
    if let Some(m) = y.get("m_LocalScale") {
        for (i, k) in ["x", "y"].iter().enumerate() {
            if let Some(v) = nf(m, k) {
                scale[i] = clampv(v, 1000.0).abs();
            }
        }
    }
    RtData { rt, scale, tilt }
}

/// Places a RectTransform in its parent frame (world frame of the page). `pcs` is the accumulated scale of the ancestors.
fn place(d: &RtData, pf: &Frame, pcs: [f32; 2]) -> (Frame, [f32; 2]) {
    let mut r = d.rt;
    r.anchored_position = [r.anchored_position[0] * pcs[0], r.anchored_position[1] * pcs[1]];
    r.size_delta = [r.size_delta[0] * pcs[0], r.size_delta[1] * pcs[1]];
    let mut e = Element::default();
    apply_rect_transform(&mut e, &r, pf);
    let mut f = e.frame();
    let [sx, sy] = d.scale;
    if (sx - 1.0).abs() > 1e-4 || (sy - 1.0).abs() > 1e-4 {
        let pv = [(r.pivot[0] - 0.5) * f.w, (0.5 - r.pivot[1]) * f.h];
        let p = f.to_world(pv);
        let off = Frame { cx: 0.0, cy: 0.0, w: f.w, h: f.h, rot: f.rot }.to_world([pv[0] * sx, pv[1] * sy]);
        f = Frame { cx: p[0] - off[0], cy: p[1] - off[1], w: f.w * sx, h: f.h * sy, rot: f.rot };
    }
    (f, [pcs[0] * sx, pcs[1] * sy])
}

fn is_stretch_full(rt: &RectTransform) -> bool {
    rt.anchor_min[0].abs() < 1e-3 && rt.anchor_min[1].abs() < 1e-3 && (rt.anchor_max[0] - 1.0).abs() < 1e-3 && (rt.anchor_max[1] - 1.0).abs() < 1e-3
}

fn tmp_align(y: &Y) -> (u8, u8) {
    let hb = ni(y, "m_HorizontalAlignment");
    let vb = ni(y, "m_VerticalAlignment");
    let (hb, vb) = if hb.is_some() || vb.is_some() {
        (hb.unwrap_or(2), vb.unwrap_or(512))
    } else {
        match ni(y, "m_textAlignment").or_else(|| ni(y, "m_TextAlignment")) {
            Some(a) if a != 65535 && a >= 0 => (a & 0xFF, a & 0xFF00),
            _ => (2, 512),
        }
    };
    let h = if hb & 1 != 0 || hb & 8 != 0 || hb & 16 != 0 {
        0
    } else if hb & 4 != 0 {
        2
    } else {
        1
    };
    let v = if vb & 256 != 0 {
        0
    } else if vb & 1024 != 0 || vb & 2048 != 0 {
        2
    } else {
        1
    };
    (h, v)
}

fn read_text(c: &Comp, assets: &mut Assets) -> Txt {
    let y = c.y;
    let mut t = Txt { text: String::new(), size: 32.0, color: [255, 255, 255, 255], bold: false, italic: false, underline: false, align: 1, valign: 1, font: None, raycast: nb(y, "m_RaycastTarget").unwrap_or(true), enabled: c.enabled };
    if c.kind == CK::Tmp {
        t.text = ns(y, "m_text").or_else(|| ns(y, "m_Text")).unwrap_or("").to_string();
        t.size = nf(y, "m_fontSize").unwrap_or(36.0).clamp(1.0, 1000.0);
        if nb(y, "m_enableAutoSizing").unwrap_or(false) {
            // auto size: the saved m_fontSize is the size it ended up with, the design size is the maximum
            if let Some(mx) = nf(y, "m_fontSizeMax").filter(|v| *v >= 1.0) {
                t.size = mx.clamp(1.0, 1000.0);
            }
        }
        t.color = color_of(y, "m_fontColor")
            .or_else(|| {
                let n = ni(y, "m_fontColor32").or_else(|| y.get("m_fontColor32").and_then(|m| ni(m, "rgba")))? as u64 as u32;
                Some([(n & 0xFF) as u8, ((n >> 8) & 0xFF) as u8, ((n >> 16) & 0xFF) as u8, (n >> 24) as u8])
            })
            .or_else(|| color_of(y, "m_Color"))
            .unwrap_or([255, 255, 255, 255]);
        let st = ni(y, "m_fontStyle").unwrap_or(0);
        t.bold = st & 1 != 0;
        t.italic = st & 2 != 0;
        t.underline = st & 4 != 0;
        (t.align, t.valign) = tmp_align(y);
        if let Some(g) = y.get("m_fontAsset").and_then(guid_of) {
            t.font = assets.stem(g);
        }
    } else {
        t.text = ns(y, "m_Text").unwrap_or("").to_string();
        t.color = color_of(y, "m_Color").unwrap_or([255, 255, 255, 255]);
        if let Some(fd) = y.get("m_FontData") {
            t.size = nf(fd, "m_FontSize").unwrap_or(14.0).clamp(1.0, 1000.0);
            let st = ni(fd, "m_FontStyle").unwrap_or(0);
            t.bold = st & 1 != 0;
            t.italic = st & 2 != 0;
            let a = ni(fd, "m_Alignment").unwrap_or(4).clamp(0, 8) as u8;
            t.align = a % 3;
            t.valign = a / 3;
            if let Some(g) = fd.get("m_Font").and_then(guid_of) {
                if !g.starts_with("0000000000000000") {
                    t.font = assets.stem(g);
                }
            }
        }
    }
    t
}

struct Img {
    color: Rgba,
    raycast: bool,
    enabled: bool,
    sprite: Option<(i64, String)>,
    ty: i64,
    preserve: bool,
    mult: f32,
}

fn read_image(c: &Comp) -> Img {
    let y = c.y;
    let sprite = y.get("m_Sprite").and_then(|s| guid_of(s).map(|g| (file_id_of(s), g.to_ascii_lowercase())));
    Img {
        color: color_of(y, "m_Color").unwrap_or([255, 255, 255, 255]),
        raycast: nb(y, "m_RaycastTarget").unwrap_or(true),
        enabled: c.enabled,
        sprite,
        ty: ni(y, "m_Type").unwrap_or(0),
        preserve: nb(y, "m_PreserveAspect").unwrap_or(false),
        mult: nf(y, "m_PixelsPerUnitMultiplier").filter(|v| *v > 0.0).unwrap_or(1.0),
    }
}

fn read_layout(c: &Comp, kind: CK, w: &mut Warn, go: &str) -> LayoutGroup {
    let y = c.y;
    let mut l = LayoutGroup::default();
    let pad = |k: &str| y.get("m_Padding").and_then(|p| nf(p, k)).unwrap_or(0.0);
    l.padding = [pad("m_Left"), pad("m_Right"), pad("m_Top"), pad("m_Bottom")];
    l.align = ni(y, "m_ChildAlignment").unwrap_or(0).clamp(0, 8) as u8;
    match kind {
        CK::Grid => {
            l.mode = LayoutMode::Grid;
            l.cell = v2(y, "m_CellSize").unwrap_or([100.0, 100.0]);
            l.spacing = v2(y, "m_Spacing").unwrap_or([0.0, 0.0]);
            if ni(y, "m_Constraint").unwrap_or(0) != 0 || ni(y, "m_StartAxis").unwrap_or(0) != 0 || ni(y, "m_StartCorner").unwrap_or(0) != 0 {
                w.group("Grid constraint / start corner / start axis not supported", go);
            }
        }
        _ => {
            let s = nf(y, "m_Spacing").unwrap_or(0.0);
            if kind == CK::Hlg {
                l.mode = LayoutMode::Horizontal;
                l.spacing = [s, 0.0];
            } else {
                l.mode = LayoutMode::Vertical;
                l.spacing = [0.0, s];
            }
            let b = |k: &str| nb(y, k).unwrap_or(false);
            l.expand_w = b("m_ChildControlWidth") && b("m_ChildForceExpandWidth");
            l.expand_h = b("m_ChildControlHeight") && b("m_ChildForceExpandHeight");
            if b("m_ChildControlWidth") || b("m_ChildControlHeight") {
                w.group("Layout group controls the size of its children: sizes saved in the prefab may be 0 until Unity lays them out", go);
            }
            if b("m_ChildControlWidth") != b("m_ChildForceExpandWidth") || b("m_ChildControlHeight") != b("m_ChildForceExpandHeight") {
                w.group("Child control / force expand flags only partly supported", go);
            }
            if b("m_ReverseArrangement") {
                w.group("Reverse arrangement not supported", go);
            }
        }
    }
    crate::docext::sanitize_layout(&mut l);
    l
}

/// Child that holds only a text (a label): stretched over its parent, no children.
fn is_label_child(sc: &Scene, by_father: &HashMap<i64, Vec<i64>>, rt: i64, assets: &mut Assets) -> bool {
    let Some(o) = sc.objs.get(&rt) else { return false };
    if !sc.kids(rt, by_father).is_empty() {
        return false;
    }
    let d = read_rt(&o.y);
    if !is_stretch_full(&d.rt) {
        return false;
    }
    let comps = comps_of(sc, o.go, assets);
    find(&comps, &[CK::Text, CK::Tmp]).is_some()
}

/// Everything decided about a GameObject before it is placed.
struct Built {
    e: Element,
    /// Child transforms swallowed by this element (standard sub-objects of Unity controls).
    absorbed: HashSet<i64>,
    /// ScrollView: (content transform, chain of transforms from the ScrollView down to it).
    content: Option<i64>,
    alpha: f32,
    /// Border ring made of an `ellipse_<n>` sprite: the stroke width is n / 256 of the short side (needs the final size).
    ring_n: Option<u32>,
}

fn warn_unsupported(cx: &mut Ctx, comps: &[Comp], name: &str, used: &[i64]) {
    for c in comps {
        if used.contains(&c.id) {
            continue;
        }
        match c.kind {
            CK::Other => cx.w.group(&format!("Unsupported component {}", c.name), name),
            CK::Selectable => cx.w.group(&format!("Unsupported component {} (Selectable)", c.name), name),
            CK::Scrollbar => cx.w.group("Scrollbar imported as a plain container (no Scrollbar element)", name),
            CK::RawImage => cx.w.group("RawImage imported as a plain panel (texture not mapped)", name),
            CK::Csf => cx.w.group("ContentSizeFitter ignored", name),
            CK::LayoutElement => cx.w.group("LayoutElement ignored", name),
            CK::Canvas => cx.w.group("Nested Canvas imported as a container", name),
            CK::CanvasScaler => cx.w.group("Nested CanvasScaler ignored", name),
            _ => {}
        }
    }
}

fn build(sc: &Scene, cx: &mut Ctx, rt_id: i64, go: i64, alpha_in: f32) -> Built {
    let name_raw = sc.go_name(go);
    let comps = comps_of(sc, go, cx.assets);
    let kids = sc.kids(rt_id, &cx.by_father);
    let mut used: Vec<i64> = Vec::new();
    let mut absorbed: HashSet<i64> = HashSet::new();
    let mut content: Option<i64> = None;

    let slider = find(&comps, &[CK::Slider]);
    let toggle = find(&comps, &[CK::Toggle]);
    let dropdown = find(&comps, &[CK::Dropdown]);
    let input = find(&comps, &[CK::InputField]);
    let scroll = find(&comps, &[CK::ScrollRect]);
    let button = find(&comps, &[CK::Button]);
    let img_c = find(&comps, &[CK::Image]);
    let txt_c = find(&comps, &[CK::Text, CK::Tmp]);

    let kind = if slider.is_some() {
        Kind::Slider
    } else if toggle.is_some() {
        Kind::Toggle
    } else if dropdown.is_some() {
        Kind::Dropdown
    } else if input.is_some() {
        Kind::InputField
    } else if scroll.is_some() {
        Kind::ScrollView
    } else if button.is_some() {
        Kind::Button
    } else if txt_c.is_some() {
        Kind::Text
    } else if img_c.is_some() {
        Kind::Panel // may become Image below
    } else if find(&comps, &[CK::RawImage, CK::Scrollbar]).is_some() {
        Kind::Panel
    } else {
        Kind::Empty
    };

    let mut e = Element::new(0, kind);
    // the kind defaults (radius, border, background) are not part of a Unity object
    e.radius = 0.0;
    e.stroke_w = 0.0;
    if img_c.is_none() {
        e.fill[3] = 0;
    }
    let label = clean_label(&name_raw, 128).unwrap_or_else(|| kind.label().to_string());
    e.name = label.replace('/', "-");
    let uname = name_raw.trim().to_string();
    if valid_unity_name(&uname) {
        e.unity_name = Some(uname);
    } else if !name_raw.trim().is_empty() {
        cx.w.group("GameObject name is not a valid Lienzo unity_name (contains '/' or control characters), using a cleaned label", &name_raw);
    }
    e.visible = sc.objs.get(&go).and_then(|o| nb(&o.y, "m_IsActive")).unwrap_or(true);
    e.mask = false;
    e.raycast = false;

    // CanvasGroup alpha folds into this element and its descendants
    let mut alpha = alpha_in;
    if let Some(cg) = find(&comps, &[CK::CanvasGroup]) {
        used.push(cg.id);
        if let Some(a) = nf(cg.y, "m_Alpha") {
            if (a - 1.0).abs() > 1e-4 {
                alpha *= a.clamp(0.0, 1.0);
                cx.w.group("CanvasGroup alpha folded into the opacity of the element and its children", &name_raw);
            }
        }
    }

    // ---- Image (background, sprite)
    let mut has_picture = false;
    let mut generated = false;
    let mut as_ellipse = false;
    let mut gradient: Option<(Rgba, Rgba)> = None;
    if let Some(c) = img_c {
        used.push(c.id);
        let im = read_image(c);
        e.fill = im.color;
        if !im.enabled {
            e.fill[3] = 0;
        }
        e.raycast = im.raycast;
        e.preserve_aspect = im.preserve;
        if let Some((fid, guid)) = &im.sprite {
            match cx.assets.resolve_sprite(*fid, guid) {
                SpriteRes::Found { path, sprite, border, ppu } => match generated_sprite(&path) {
                    // sprites Lienzo itself generated on export: back to a rounded panel / circle / gradient
                    Some(Gen::Rounded(r, _)) => {
                        generated = true;
                        e.radius = (r as f32).clamp(0.0, 10_000.0);
                    }
                    Some(Gen::Ellipse(_)) => {
                        generated = true;
                        if kind == Kind::Panel {
                            as_ellipse = true;
                        }
                    }
                    Some(Gen::Gradient(a, b)) => {
                        generated = true;
                        gradient = Some((a, b));
                    }
                    None => {
                        has_picture = true;
                        e.image = format!("unity:{}", path);
                        e.sprite = sprite;
                        e.slice = border;
                        let base = ppu.unwrap_or(if im.mult != 1.0 { 100.0 } else { 0.0 });
                        if base > 0.0 {
                            e.ppu = Some(base * im.mult);
                        }
                    }
                },
                SpriteRes::Builtin => cx.builtin_sprites += 1,
                SpriteRes::Missing => {
                    cx.missing_count += 1;
                    if cx.missing_guids.len() < 5 && !cx.missing_guids.contains(guid) {
                        cx.missing_guids.push(guid.clone());
                    }
                }
                SpriteRes::None => {}
            }
        }
        e.image_mode = match im.ty {
            _ if generated => None,
            1 => Some(ImageMode::Sliced),
            2 => Some(ImageMode::Tiled),
            3 => Some(ImageMode::Filled),
            _ => None,
        };
        if let Some((a, b)) = gradient {
            e.fill = a;
            e.fill2 = b;
            if kind == Kind::Panel {
                e.kind = Kind::Image;
            }
            e.opacity = (im.color[3] as f32 / 255.0).clamp(0.0, 1.0);
        }
        if as_ellipse {
            e.kind = Kind::Ellipse;
        }
        if has_picture {
            if kind == Kind::Panel {
                e.kind = Kind::Image;
            }
            e.opacity = (im.color[3] as f32 / 255.0).clamp(0.0, 1.0);
        }
        if im.ty == 3 {
            cx.w.group("Filled image: fill amount / method not supported", &name_raw);
        }
    }

    // ---- selectable state shared by all controls
    for c in [button, toggle, slider, input, dropdown].into_iter().flatten() {
        used.push(c.id);
        e.interactable = nb(c.y, "m_Interactable").unwrap_or(true);
        if !c.enabled {
            e.interactable = false;
        }
        let calls = c.y.get("m_OnClick").or_else(|| c.y.get("m_OnValueChanged")).and_then(|u| u.get("m_PersistentCalls")).and_then(|p| p.get("m_Calls")).map(|v| v.seq().len()).unwrap_or(0);
        if calls > 0 {
            cx.onclick += calls;
        }
    }

    let apply_txt = |e: &mut Element, t: &Txt, fonts: &mut HashMap<String, usize>| {
        e.text = t.text.clone();
        e.font_size = t.size;
        e.text_color = t.color;
        e.bold = t.bold;
        e.italic = t.italic;
        e.underline = t.underline;
        e.align = t.align;
        e.valign = t.valign;
        if let Some(f) = &t.font {
            *fonts.entry(f.clone()).or_insert(0) += 1;
            e.font = f.clone();
        }
    };

    let label_child = |cx: &mut Ctx, absorbed: &mut HashSet<i64>, only_named: Option<&str>| -> Option<Txt> {
        for k in &kids {
            if absorbed.contains(k) {
                continue;
            }
            let Some(ko) = sc.objs.get(k) else { continue };
            let kg = ko.go;
            if let Some(n) = only_named {
                if sc.go_name(kg) != n {
                    continue;
                }
            }
            if is_label_child(sc, &cx.by_father, *k, &mut *cx.assets) {
                let kc = comps_of(sc, kg, cx.assets);
                if let Some(tc) = find(&kc, &[CK::Text, CK::Tmp]) {
                    let t = read_text(tc, cx.assets);
                    absorbed.insert(*k);
                    return Some(t);
                }
            }
        }
        None
    };

    // ---- per-kind details
    match kind {
        Kind::Text => {
            if let Some(c) = txt_c {
                used.push(c.id);
                let t = read_text(c, cx.assets);
                apply_txt(&mut e, &t, &mut cx.fonts);
                e.raycast = t.raycast;
                if !t.enabled {
                    e.text.clear();
                }
            }
        }
        Kind::Button => {
            if let Some(t) = label_child(cx, &mut absorbed, None) {
                apply_txt(&mut e, &t, &mut cx.fonts);
            }
        }
        Kind::Panel | Kind::Image => {
            // Lienzo exports a caption of a panel as a stretched child called "Texto"
            if let Some(t) = label_child(cx, &mut absorbed, Some("Texto")) {
                apply_txt(&mut e, &t, &mut cx.fonts);
            }
        }
        Kind::Toggle => {
            let c = toggle.unwrap();
            e.value = if nb(c.y, "m_IsOn").unwrap_or(false) { 1.0 } else { 0.0 };
            if let Some(t) = label_child(cx, &mut absorbed, None) {
                apply_txt(&mut e, &t, &mut cx.fonts);
            }
            // checkmark colour is the toggle colour in Lienzo; swallow the box (Background / Checkmark)
            let mut check_rt = None;
            for k in ["graphic", "m_TargetGraphic"] {
                if let Some(r) = sc.rt_of_ref(rid(c.y, k)) {
                    if let Some(ch) = sc.child_containing(rt_id, r) {
                        absorbed.insert(ch);
                    }
                    if k == "graphic" {
                        check_rt = Some(r);
                    }
                }
            }
            if let Some(r) = check_rt {
                let g = sc.objs.get(&r).map(|o| o.go).unwrap_or(0);
                let gc = comps_of(sc, g, cx.assets);
                if let Some(ic) = find(&gc, &[CK::Image]) {
                    e.fill = read_image(ic).color;
                }
            }
            e.fill[3] = 255;
        }
        Kind::Slider => {
            let c = slider.unwrap();
            let (mn, mx) = (nf(c.y, "m_MinValue").unwrap_or(0.0), nf(c.y, "m_MaxValue").unwrap_or(1.0));
            let v = nf(c.y, "m_Value").unwrap_or(0.0);
            e.value = if mx > mn { ((v - mn) / (mx - mn)).clamp(0.0, 1.0) } else { 0.5 };
            if ni(c.y, "m_Direction").unwrap_or(0) >= 2 {
                cx.w.group("Vertical slider imported as a horizontal one", &name_raw);
            }
            // fill colour from the Fill image, track colour from the Background child
            if let Some(r) = sc.rt_of_ref(rid(c.y, "m_FillRect")) {
                let g = sc.objs.get(&r).map(|o| o.go).unwrap_or(0);
                let gc = comps_of(sc, g, cx.assets);
                if let Some(ic) = find(&gc, &[CK::Image]) {
                    e.fill = read_image(ic).color;
                }
                if let Some(ch) = sc.child_containing(rt_id, r) {
                    absorbed.insert(ch);
                }
            }
            if let Some(r) = sc.rt_of_ref(rid(c.y, "m_HandleRect")) {
                if let Some(ch) = sc.child_containing(rt_id, r) {
                    absorbed.insert(ch);
                }
            }
            for k in &kids {
                let g = sc.objs.get(k).map(|o| o.go).unwrap_or(0);
                let n = sc.go_name(g);
                if n == "Background" {
                    let gc = comps_of(sc, g, cx.assets);
                    if let Some(ic) = find(&gc, &[CK::Image]) {
                        e.fill2 = read_image(ic).color;
                    }
                    absorbed.insert(*k);
                }
            }
        }
        Kind::InputField => {
            let c = input.unwrap();
            let mut text = ns(c.y, "m_Text").unwrap_or("").to_string();
            let mut style: Option<Txt> = None;
            let use_placeholder = text.is_empty();
            let style_key = if use_placeholder && sc.rt_of_ref(rid(c.y, "m_Placeholder")).is_some() { "m_Placeholder" } else { "m_TextComponent" };
            if let Some(r) = sc.rt_of_ref(rid(c.y, style_key)) {
                let g = sc.objs.get(&r).map(|o| o.go).unwrap_or(0);
                let gc = comps_of(sc, g, cx.assets);
                if let Some(tc) = find(&gc, &[CK::Text, CK::Tmp]) {
                    style = Some(read_text(tc, cx.assets));
                }
            }
            for k in ["m_TextComponent", "m_Placeholder", "m_TextViewport"] {
                if let Some(r) = sc.rt_of_ref(rid(c.y, k)) {
                    if let Some(ch) = sc.child_containing(rt_id, r) {
                        absorbed.insert(ch);
                    }
                }
            }
            if text.is_empty() {
                if let Some(r) = sc.rt_of_ref(rid(c.y, "m_Placeholder")) {
                    let g = sc.objs.get(&r).map(|o| o.go).unwrap_or(0);
                    let gc = comps_of(sc, g, cx.assets);
                    if let Some(tc) = find(&gc, &[CK::Text, CK::Tmp]) {
                        text = read_text(tc, cx.assets).text;
                    }
                }
            }
            if let Some(mut t) = style {
                t.text = text;
                apply_txt(&mut e, &t, &mut cx.fonts);
            } else {
                e.text = text;
            }
        }
        Kind::Dropdown => {
            let c = dropdown.unwrap();
            let opts: Vec<String> = c.y.get("m_Options").and_then(|o| o.get("m_Options")).map(|o| o.seq()).unwrap_or(&[]).iter().take(500).filter_map(|o| ns(o, "m_Text")).map(|s| s.to_string()).collect();
            e.options = opts;
            e.value = ni(c.y, "m_Value").unwrap_or(0).max(0) as f32;
            if let Some(r) = sc.rt_of_ref(rid(c.y, "m_CaptionText")) {
                let g = sc.objs.get(&r).map(|o| o.go).unwrap_or(0);
                let gc = comps_of(sc, g, cx.assets);
                if let Some(tc) = find(&gc, &[CK::Text, CK::Tmp]) {
                    let t = read_text(tc, cx.assets);
                    apply_txt(&mut e, &t, &mut cx.fonts);
                }
            }
            for k in ["m_CaptionText", "m_CaptionImage", "m_Template", "m_ItemText", "m_ItemImage"] {
                if let Some(r) = sc.rt_of_ref(rid(c.y, k)) {
                    if let Some(ch) = sc.child_containing(rt_id, r) {
                        absorbed.insert(ch);
                    }
                }
            }
            for k in &kids {
                let g = sc.objs.get(k).map(|o| o.go).unwrap_or(0);
                if sc.go_name(g) == "Arrow" {
                    absorbed.insert(*k);
                }
            }
        }
        Kind::ScrollView => {
            let c = scroll.unwrap();
            let vp = sc.rt_of_ref(rid(c.y, "m_Viewport"));
            let ct = sc.rt_of_ref(rid(c.y, "m_Content"));
            let mut mask = false;
            if let Some(v) = vp {
                if let Some(ch) = sc.child_containing(rt_id, v) {
                    absorbed.insert(ch);
                }
                let g = sc.objs.get(&v).map(|o| o.go).unwrap_or(0);
                let vc = comps_of(sc, g, cx.assets);
                mask = find(&vc, &[CK::Mask, CK::RectMask2D]).is_some();
            }
            for k in ["m_HorizontalScrollbar", "m_VerticalScrollbar"] {
                if let Some(r) = sc.rt_of_ref(rid(c.y, k)) {
                    if let Some(ch) = sc.child_containing(rt_id, r) {
                        absorbed.insert(ch);
                    }
                }
            }
            e.mask = mask;
            content = ct;
            if let Some(ct) = ct {
                let g = sc.objs.get(&ct).map(|o| o.go).unwrap_or(0);
                let cc = comps_of(sc, g, cx.assets);
                let nm = sc.go_name(g);
                if let Some(l) = find(&cc, &[CK::Hlg, CK::Vlg, CK::HvUnknown, CK::Grid]) {
                    let kind_l = layout_kind(l, sc, ct, cx);
                    e.layout = read_layout(l, kind_l, &mut cx.w, &nm);
                }
                for c in &cc {
                    if c.kind == CK::Csf {
                        cx.w.group("ContentSizeFitter ignored", &nm);
                    }
                }
            }
        }
        _ => {}
    }

    // ---- ring border swallowed ("Borde": stretched Image child with no children)
    let mut ring_n: Option<u32> = None;
    if e.kind.has_fill() {
        for k in &kids {
            if absorbed.contains(k) {
                continue;
            }
            let Some(o) = sc.objs.get(k) else { continue };
            if sc.go_name(o.go) != "Borde" || !sc.kids(*k, &cx.by_father).is_empty() || !is_stretch_full(&read_rt(&o.y).rt) {
                continue;
            }
            let kc = comps_of(sc, o.go, cx.assets);
            if let Some(ic) = find(&kc, &[CK::Image]) {
                let im = read_image(ic);
                e.stroke = im.color;
                absorbed.insert(*k);
                let ring = match &im.sprite {
                    Some((fid, guid)) => match cx.assets.resolve_sprite(*fid, guid) {
                        SpriteRes::Found { path, .. } => match generated_sprite(&path) {
                            Some(Gen::Rounded(_, s)) => Some(s as f32),
                            Some(Gen::Ellipse(n)) if n > 0 => {
                                ring_n = Some(n);
                                Some(2.0)
                            }
                            _ => None,
                        },
                        _ => None,
                    },
                    None => None,
                };
                match ring {
                    Some(s) => e.stroke_w = s.clamp(0.0, 1000.0),
                    None => {
                        e.stroke_w = 2.0;
                        cx.w.group("Border ring imported with an approximate stroke width of 2 px", &name_raw);
                    }
                }
            }
        }
    }

    // ---- masks, layouts, effects on this GameObject
    if let Some(m) = find(&comps, &[CK::Mask, CK::RectMask2D]) {
        used.push(m.id);
        e.mask = true;
        if m.kind == CK::Mask {
            cx.w.group("Mask imported as RectMask2D (rectangular clip)", &name_raw);
        }
    }
    if let Some(l) = find(&comps, &[CK::Hlg, CK::Vlg, CK::HvUnknown, CK::Grid]) {
        used.push(l.id);
        let kind_l = layout_kind(l, sc, rt_id, cx);
        e.layout = read_layout(l, kind_l, &mut cx.w, &name_raw);
    }
    for c in &comps {
        match c.kind {
            CK::Outline => {
                used.push(c.id);
                if let (Some(col), Some(d)) = (color_of(c.y, "m_EffectColor"), v2(c.y, "m_EffectDistance")) {
                    if e.kind.has_fill() {
                        e.stroke = col;
                        e.stroke_w = d[0].abs().max(d[1].abs()).clamp(0.0, 1000.0);
                    } else {
                        cx.w.group("Outline on text not supported", &name_raw);
                    }
                }
            }
            CK::Shadow | CK::ShadowUnknown => {
                used.push(c.id);
                if let (Some(col), Some(d)) = (color_of(c.y, "m_EffectColor"), v2(c.y, "m_EffectDistance")) {
                    e.shadow.on = true;
                    e.shadow.color = col;
                    e.shadow.offset = [clampv(d[0], 1000.0), clampv(d[1], 1000.0)];
                }
            }
            CK::Ignored => used.push(c.id),
            _ => {}
        }
    }
    // components that are consumed above (Text on text elements, controls) were pushed to `used`
    if let Some(c) = txt_c {
        used.push(c.id);
    }
    for c in [scroll].into_iter().flatten() {
        used.push(c.id);
    }
    warn_unsupported(cx, &comps, &name_raw, &used);

    if alpha < 1.0 {
        e.opacity = (e.opacity * alpha).clamp(0.0, 1.0);
    }
    Built { e, absorbed, content, alpha, ring_n }
}

/// Horizontal or vertical when the script itself does not say: look at how the children are laid out.
fn layout_kind(c: &Comp, sc: &Scene, rt: i64, cx: &Ctx) -> CK {
    match c.kind {
        CK::Hlg | CK::Vlg | CK::Grid => c.kind,
        _ => {
            let kids = sc.kids(rt, &cx.by_father);
            let pos: Vec<[f32; 2]> = kids.iter().filter_map(|k| sc.objs.get(k)).filter_map(|o| v2(&o.y, "m_AnchoredPosition")).collect();
            let spread = |i: usize| {
                let lo = pos.iter().map(|p| p[i]).fold(f32::MAX, f32::min);
                let hi = pos.iter().map(|p| p[i]).fold(f32::MIN, f32::max);
                hi - lo
            };
            if pos.len() >= 2 && spread(0) > spread(1) { CK::Hlg } else { CK::Vlg }
        }
    }
}

fn walk(sc: &Scene, cx: &mut Ctx, rt_id: i64, parent: Option<u64>, pf: Frame, pcs: [f32; 2], alpha: f32, depth: usize, rt_override: Option<RectTransform>) {
    if depth > MAX_NEST {
        if !cx.too_deep {
            cx.too_deep = true;
            cx.w.add(format!("Hierarchy deeper than {} levels: the deeper objects were not imported", MAX_NEST));
        }
        return;
    }
    if !cx.visited.insert(rt_id) {
        return;
    }
    let Some(rt_obj) = sc.objs.get(&rt_id) else { return };
    let go = rt_obj.go;
    if rt_obj.class == 4 {
        // a plain Transform (not UI): its children continue at the same place
        for k in sc.kids(rt_id, &cx.by_father) {
            walk(sc, cx, k, parent, pf, pcs, alpha, depth + 1, None);
        }
        return;
    }
    if cx.doc.elements.len() >= MAX_ELEMENTS {
        if !cx.too_many {
            cx.too_many = true;
            cx.w.add(format!("More than {} elements: the rest was not imported", MAX_ELEMENTS));
        }
        return;
    }
    let mut b = build(sc, cx, rt_id, go, alpha);
    let mut data = read_rt(&rt_obj.y);
    if let Some(o) = rt_override {
        data.rt = o;
    }
    if data.tilt {
        cx.w.group("3D rotation (X / Y) ignored, only the Z rotation was imported", &b.e.name.clone());
    }
    if data.scale != [1.0, 1.0] {
        cx.w.group("RectTransform scale folded into the size and position", &b.e.name.clone());
    }
    let (frame, cs) = place(&data, &pf, pcs);
    b.e.set_frame(&frame);
    if let Some(n) = b.ring_n {
        b.e.stroke_w = (n as f32 * frame.w.min(frame.h) / 256.0).clamp(0.0, 1000.0);
    }
    b.e.anchor_min = data.rt.anchor_min;
    b.e.anchor_max = data.rt.anchor_max;
    b.e.pivot = data.rt.pivot;
    b.e.parent = parent;
    let id = cx.doc.add(b.e);
    let alpha = b.alpha;

    // children
    let mut handled: HashSet<i64> = b.absorbed.clone();
    if let Some(ct) = b.content {
        // chain ScrollView -> ... -> Content, placed one by one
        let mut chain = Vec::new();
        let mut cur = ct;
        for _ in 0..64 {
            if cur == rt_id || cur == 0 {
                break;
            }
            chain.push(cur);
            cur = sc.father(cur);
        }
        if cur == rt_id {
            chain.reverse();
            // Like Lienzo's own export, the Viewport is taken as the full ScrollView box (the width Unity takes
            // away for a visible scrollbar at runtime is not modelled); only the Content is placed.
            let (mut f, mut c) = (frame, cs);
            if let Some(o) = chain.last().and_then(|r| sc.objs.get(r)) {
                let d = read_rt(&o.y);
                (f, c) = place(&d, &f, c);
            }
            cx.visited.extend(chain.iter().copied());
            for k in sc.kids(ct, &cx.by_father) {
                walk(sc, cx, k, Some(id), f, c, alpha, depth + 1, None);
            }
        }
        handled.extend(chain);
    }
    for k in sc.kids(rt_id, &cx.by_father) {
        if handled.contains(&k) {
            continue;
        }
        walk(sc, cx, k, Some(id), frame, cs, alpha, depth + 1, None);
    }
}

fn read_canvas(canvas: &Comp, doc: &mut Doc) {
    let y = canvas.y;
    doc.canvas.render_mode = match ni(y, "m_RenderMode").unwrap_or(0) {
        1 => CanvasRenderMode::ScreenSpaceCamera,
        2 => CanvasRenderMode::WorldSpace,
        _ => CanvasRenderMode::ScreenSpaceOverlay,
    };
    doc.canvas.sort_order = ni(y, "m_SortingOrder").unwrap_or(0).clamp(-32768, 32767) as i32;
    if let Some(p) = nf(y, "m_PlaneDistance") {
        doc.canvas.plane_distance = p;
    }
    doc.pixel_art = nb(y, "m_PixelPerfect").unwrap_or(false);
}

/// Imports the text of a prefab. `title` becomes the design title.
pub(crate) fn import_prefab_text(text: &str, title: &str, assets: &mut Assets) -> Result<Imported, String> {
    let sc = load_scene(text.strip_prefix('\u{feff}').unwrap_or(text))?;
    if !sc.objs.values().any(|o| o.class == 1) {
        return Err(if sc.instances > 0 { "this prefab only holds prefab instances (a prefab variant): open the source prefab".to_string() } else { "no GameObject found: not a Unity prefab".to_string() });
    }
    let transforms: Vec<i64> = sc.order.iter().copied().filter(|id| sc.is_transform(*id)).collect();
    if !transforms.iter().any(|id| sc.objs.get(id).is_some_and(|o| o.class == 224)) {
        return Err("no RectTransform found: this prefab has no UI hierarchy".to_string());
    }
    let mut by_father: HashMap<i64, Vec<i64>> = HashMap::new();
    for id in &transforms {
        let f = sc.father(*id);
        if f != 0 && sc.is_transform(f) {
            by_father.entry(f).or_default().push(*id);
        }
    }
    let mut cx = Ctx {
        doc: Doc::empty(1920.0, 1080.0),
        w: Warn::default(),
        assets,
        visited: HashSet::new(),
        fonts: HashMap::new(),
        by_father,
        builtin_sprites: 0,
        missing_guids: Vec::new(),
        missing_count: 0,
        onclick: 0,
        too_many: false,
        too_deep: false,
    };
    cx.doc.title = clean_label(title, 128).unwrap_or_else(|| "Imported design".to_string());
    // A Unity canvas has no background: keep the page transparent so no "Fondo del lienzo" is exported.
    cx.doc.background = [0, 0, 0, 0];

    if sc.duplicates > 0 {
        cx.w.add(format!("{} duplicate fileID document(s) ignored (the first one of each was used)", sc.duplicates));
    }
    if sc.instances > 0 {
        cx.w.add(format!("{} nested prefab instance(s) / stripped object(s) are not expanded: open the nested prefabs separately", sc.instances));
    }

    // roots: transforms without a (known) father; plain Transforms are looked through
    let mut roots: Vec<i64> = Vec::new();
    for id in &transforms {
        let f = sc.father(*id);
        if f == 0 || !sc.is_transform(f) {
            roots.push(*id);
        }
    }
    if roots.is_empty() {
        return Err("the hierarchy has no root (cyclic m_Father?)".to_string());
    }
    let mut ui_roots: Vec<i64> = Vec::new();
    let mut stack: Vec<i64> = roots.iter().rev().copied().collect();
    let mut seen: HashSet<i64> = HashSet::new();
    while let Some(r) = stack.pop() {
        if !seen.insert(r) {
            continue;
        }
        if sc.objs.get(&r).is_some_and(|o| o.class == 4) {
            for k in sc.kids(r, &cx.by_father).into_iter().rev() {
                stack.push(k);
            }
        } else {
            ui_roots.push(r);
        }
        if seen.len() > MAX_ELEMENTS * 4 {
            break;
        }
    }
    if ui_roots.is_empty() {
        return Err("no RectTransform root found".to_string());
    }

    let root_has = |r: i64, kinds: &[u32]| {
        let go = sc.objs.get(&r).map(|o| o.go).unwrap_or(0);
        sc.comp_ids(go).iter().any(|c| sc.objs.get(c).is_some_and(|o| kinds.contains(&o.class)))
    };
    let canvas_root = ui_roots.iter().copied().find(|r| root_has(*r, &[223]));
    let page_root = canvas_root.unwrap_or(ui_roots[0]);
    let page_go = sc.objs.get(&page_root).map(|o| o.go).unwrap_or(0);
    let page_rt = read_rt(&sc.objs[&page_root].y);
    let page_comps = comps_of(&sc, page_go, cx.assets);

    let (mut pw, mut ph) = (1920.0f32, 1080.0f32);
    let mut page_is_root_element = false;
    if let Some(cv) = find(&page_comps, &[CK::Canvas]) {
        read_canvas(cv, &mut cx.doc);
        if let Some(scaler) = find(&page_comps, &[CK::CanvasScaler]) {
            let mode = ni(scaler.y, "m_UiScaleMode").unwrap_or(0);
            let res = v2(scaler.y, "m_ReferenceResolution");
            if mode == 1 {
                if let Some(r) = res {
                    pw = r[0];
                    ph = r[1];
                }
                cx.doc.canvas_match = nf(scaler.y, "m_MatchWidthOrHeight").unwrap_or(0.5).clamp(0.0, 1.0);
                if ni(scaler.y, "m_ScreenMatchMode").unwrap_or(0) != 0 {
                    cx.w.add("CanvasScaler uses Expand / Shrink: imported as Match Width Or Height");
                }
            } else {
                cx.w.add(format!("CanvasScaler mode is {}: the reference resolution is not used, the page size is taken from the canvas size or 1920x1080", if mode == 0 { "Constant Pixel Size" } else { "Constant Physical Size" }));
                if !page_rt.rt.size_delta.iter().any(|v| *v <= 0.0) {
                    pw = page_rt.rt.size_delta[0];
                    ph = page_rt.rt.size_delta[1];
                }
            }
        } else if !page_rt.rt.size_delta.iter().any(|v| *v <= 0.0) {
            pw = page_rt.rt.size_delta[0];
            ph = page_rt.rt.size_delta[1];
        } else {
            cx.w.add("Canvas without CanvasScaler: page size 1920x1080 assumed");
        }
        // a background Image on the canvas itself
        if let Some(im) = find(&page_comps, &[CK::Image]) {
            cx.doc.background = read_image(im).color;
        }
    } else {
        // no Canvas: a prefab meant to live under the game's canvas
        cx.doc.canvas.embed = true;
        let pure = page_comps.iter().all(|c| matches!(c.kind, CK::Ignored | CK::CanvasGroup | CK::Hlg | CK::Vlg | CK::HvUnknown | CK::Grid | CK::Csf | CK::LayoutElement));
        let stretch = page_rt.rt.stretch_x() || page_rt.rt.stretch_y();
        if pure {
            if !stretch && page_rt.rt.size_delta.iter().all(|v| *v > 0.0) {
                pw = page_rt.rt.size_delta[0];
                ph = page_rt.rt.size_delta[1];
            } else {
                cx.w.add("The root RectTransform stretches: page size 1920x1080 assumed");
            }
            if page_comps.iter().any(|c| matches!(c.kind, CK::Hlg | CK::Vlg | CK::HvUnknown | CK::Grid)) {
                cx.w.add("The layout group on the prefab root was ignored (the root is the page)");
            }
        } else {
            page_is_root_element = true;
            cx.w.add("The prefab has no Canvas: it was placed at the center of a 1920x1080 page (embed mode)");
        }
    }
    cx.doc.width = pw.clamp(1.0, 16384.0);
    cx.doc.height = ph.clamp(1.0, 16384.0);
    let page = Frame::page(cx.doc.width, cx.doc.height);
    let page_alpha = find(&page_comps, &[CK::CanvasGroup]).and_then(|c| nf(c.y, "m_Alpha")).unwrap_or(1.0).clamp(0.0, 1.0);

    if page_is_root_element {
        let mut o = page_rt.rt;
        let stretch = o.stretch_x() || o.stretch_y();
        o.anchor_min = [0.5, 0.5];
        o.anchor_max = [0.5, 0.5];
        o.anchored_position = [0.0, 0.0];
        if stretch {
            o.size_delta = [cx.doc.width, cx.doc.height];
        }
        walk(&sc, &mut cx, page_root, None, page, [1.0, 1.0], 1.0, 0, Some(o));
    } else {
        cx.visited.insert(page_root);
        let mut handled: HashSet<i64> = HashSet::new();
        // the canvas background of a Lienzo export
        let kids = sc.kids(page_root, &cx.by_father);
        if canvas_root.is_some() {
            for k in &kids {
                let Some(o) = sc.objs.get(k) else { continue };
                if sc.go_name(o.go) == "Fondo del lienzo" && sc.kids(*k, &cx.by_father).is_empty() && is_stretch_full(&read_rt(&o.y).rt) {
                    let kc = comps_of(&sc, o.go, cx.assets);
                    if let Some(im) = find(&kc, &[CK::Image]) {
                        cx.doc.background = read_image(im).color;
                        handled.insert(*k);
                    }
                }
            }
        }
        for c in &page_comps {
            if matches!(c.kind, CK::Other | CK::Selectable) {
                cx.w.group(&format!("Unsupported component {}", c.name), &sc.go_name(page_go));
            }
        }
        for k in kids {
            if handled.contains(&k) {
                continue;
            }
            walk(&sc, &mut cx, k, None, page, [1.0, 1.0], page_alpha, 1, None);
        }
    }
    // other roots (scene-like files)
    let extra: Vec<i64> = ui_roots.iter().copied().filter(|r| *r != page_root).collect();
    if !extra.is_empty() {
        cx.w.add(format!("{} extra root object(s) in the prefab were imported as top-level elements", extra.len()));
        for r in extra {
            walk(&sc, &mut cx, r, None, page, [1.0, 1.0], 1.0, 1, None);
        }
    }

    // default font: the most used TMP font asset
    if let Some((f, _)) = cx.fonts.iter().max_by(|a, b| a.1.cmp(b.1).then_with(|| b.0.cmp(a.0))) {
        let f = f.clone();
        cx.doc.unity.default_font = Some(f.clone());
        for e in cx.doc.elements.iter_mut() {
            if e.font == f {
                e.font.clear();
            }
        }
    }
    if cx.builtin_sprites > 0 {
        let n = cx.builtin_sprites;
        cx.w.add(format!("{} Image(s) use Unity built-in sprites (UISprite, Background, Knob...): left as plain color panels", n));
    }
    if cx.missing_count > 0 {
        let list = cx.missing_guids.join(", ");
        let n = cx.missing_count;
        cx.w.add(format!("{} sprite reference(s) could not be resolved to an asset (no matching .meta in the project); e.g. GUID {}", n, list));
    }
    if cx.onclick > 0 {
        let n = cx.onclick;
        cx.w.add(format!("{} UnityEvent listener(s) (OnClick / OnValueChanged) are not imported; hook them up again in your game code", n));
    }
    if cx.doc.elements.is_empty() {
        cx.w.add("The prefab produced no elements");
    }
    let mut doc = cx.doc;
    let warnings = cx.w.finish();
    doc.migrate_groups();
    Ok(Imported { doc, warnings })
}

// ===========================================================================
// 7. Lienzo export JSON (`Designs/*.json`) -> Doc
// ===========================================================================

fn jf(v: &serde_json::Value, k: &str) -> Option<f32> {
    v.get(k)?.as_f64().filter(|x| x.is_finite()).map(|x| x.clamp(-1.0e9, 1.0e9) as f32)
}
fn jb(v: &serde_json::Value, k: &str) -> Option<bool> {
    v.get(k)?.as_bool()
}
fn js<'a>(v: &'a serde_json::Value, k: &str) -> Option<&'a str> {
    v.get(k)?.as_str()
}
fn jarr(v: &serde_json::Value, k: &str, n: usize) -> Option<Vec<f32>> {
    let a = v.get(k)?.as_array()?;
    if a.len() < n {
        return None;
    }
    let out: Vec<f32> = a.iter().take(n).filter_map(|x| x.as_f64()).filter(|x| x.is_finite()).map(|x| x.clamp(-1.0e9, 1.0e9) as f32).collect();
    if out.len() == n { Some(out) } else { None }
}
fn jv2(v: &serde_json::Value, k: &str) -> Option<[f32; 2]> {
    let a = jarr(v, k, 2)?;
    Some([a[0], a[1]])
}
fn parse_hex(s: &str) -> Option<Rgba> {
    let h = s.trim().trim_start_matches('#');
    if (h.len() != 6 && h.len() != 8) || !h.is_ascii() {
        return None;
    }
    let b = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).ok();
    Some([b(0)?, b(2)?, b(4)?, if h.len() == 8 { b(6)? } else { 255 }])
}
fn jcolor(v: &serde_json::Value, k: &str) -> Option<Rgba> {
    parse_hex(js(v, k)?)
}

fn json_layout(l: &serde_json::Value) -> LayoutGroup {
    let mut g = LayoutGroup::default();
    g.mode = match l.get("mode").and_then(|m| m.as_i64()).unwrap_or(0) {
        1 => LayoutMode::Horizontal,
        2 => LayoutMode::Vertical,
        3 => LayoutMode::Grid,
        _ => LayoutMode::None,
    };
    if let Some(p) = jarr(l, "padding", 4) {
        g.padding = [p[0], p[1], p[2], p[3]];
    }
    if let Some(s) = jv2(l, "spacing") {
        g.spacing = s;
    }
    if let Some(c) = jv2(l, "cellSize") {
        g.cell = c;
    }
    g.align = l.get("childAlignment").and_then(|a| a.as_i64()).unwrap_or(0).clamp(0, 8) as u8;
    g.expand_w = jb(l, "expandWidth").unwrap_or(false);
    g.expand_h = jb(l, "expandHeight").unwrap_or(false);
    crate::docext::sanitize_layout(&mut g);
    g
}

pub(crate) fn import_export_json(text: &str, title_hint: &str) -> Result<Imported, String> {
    let t = text.strip_prefix('\u{feff}').unwrap_or(text);
    let v: serde_json::Value = serde_json::from_str(t).map_err(|e| e.to_string())?;
    let mut warnings = Warn::default();
    let fmt = js(&v, "format").unwrap_or("");
    if !fmt.starts_with("editor-gui/ugui") {
        if v.get("elements").is_some() {
            // already a Lienzo design file
            let doc = Doc::from_json_str(t)?;
            warnings.add("The file is already a Lienzo design (.editorgui.json); it was loaded as is");
            return Ok(Imported { doc, warnings: warnings.finish() });
        }
        return Err("not a Lienzo export (no \"format\": \"editor-gui/ugui/...\") and not a design file".to_string());
    }
    if fmt != "editor-gui/ugui/2" {
        warnings.add(format!("Export format {} is not the one this version knows (editor-gui/ugui/2): importing what is understood", fmt));
    }
    let empty = serde_json::Value::Null;
    let cv = v.get("canvas").unwrap_or(&empty);
    let mut doc = Doc::empty(1920.0, 1080.0);
    doc.title = clean_label(js(cv, "name").unwrap_or(title_hint), 128).unwrap_or_else(|| "Imported design".to_string());
    if let Some(r) = jv2(cv, "referenceResolution") {
        doc.width = r[0].clamp(1.0, 16384.0);
        doc.height = r[1].clamp(1.0, 16384.0);
    }
    doc.canvas_match = jf(cv, "match").unwrap_or(0.5).clamp(0.0, 1.0);
    doc.pixel_art = jb(cv, "pixelArt").unwrap_or(false);
    doc.background = jcolor(cv, "background").unwrap_or([255, 255, 255, 255]);
    match js(cv, "renderMode") {
        Some("ScreenSpaceCamera") => doc.canvas.render_mode = CanvasRenderMode::ScreenSpaceCamera,
        Some("WorldSpace") => doc.canvas.render_mode = CanvasRenderMode::WorldSpace,
        _ => {}
    }
    if let Some(s) = cv.get("sortOrder").and_then(|x| x.as_f64()) {
        doc.canvas.sort_order = s.clamp(-32768.0, 32767.0) as i32;
    }
    doc.canvas.safe_area = jb(cv, "safeArea").unwrap_or(false);
    doc.canvas.embed = jb(cv, "embed").unwrap_or(false);
    if let Some(p) = jf(cv, "planeDistance") {
        doc.canvas.plane_distance = p;
    }
    if let Some(u) = v.get("unity") {
        doc.unity.scripts_dir = js(u, "scriptsDir").map(|s| s.to_string());
        doc.unity.assembly = js(u, "assembly").map(|s| s.to_string());
        doc.unity.default_font = js(u, "defaultFont").map(|s| s.to_string());
        if let Some(a) = u.get("fallbackFonts").and_then(|a| a.as_array()) {
            doc.unity.fallback_fonts = a.iter().filter_map(|s| s.as_str().map(|s| s.to_string())).take(16).collect();
        }
        if let Some(g) = u.get("generate") {
            doc.unity.generate.prefab = jb(g, "prefab").unwrap_or(true);
            doc.unity.generate.uitk = jb(g, "uitk").unwrap_or(true);
            doc.unity.generate.imgui = Some(jb(g, "imgui").unwrap_or(false));
            doc.unity.generate.view_scripts = jb(g, "viewScripts").unwrap_or(true);
        }
    }

    let els = v.get("elements").and_then(|e| e.as_array()).cloned().unwrap_or_default();
    if els.len() > MAX_ELEMENTS {
        warnings.add(format!("More than {} elements: the rest was not imported", MAX_ELEMENTS));
    }
    let els: Vec<&serde_json::Value> = els.iter().take(MAX_ELEMENTS).collect();
    let mut ids: Vec<u64> = Vec::with_capacity(els.len());
    let mut used_ids: HashSet<u64> = HashSet::new();
    let mut next = 1u64;
    for j in &els {
        let want = j.get("id").and_then(|i| i.as_u64()).filter(|i| *i > 0 && *i < u64::MAX / 4);
        let id = match want {
            Some(i) if used_ids.insert(i) => i,
            _ => {
                while used_ids.contains(&next) {
                    next += 1;
                }
                used_ids.insert(next);
                next
            }
        };
        ids.push(id);
    }
    let mut frames: Vec<Frame> = Vec::with_capacity(els.len());
    let mut kinds: Vec<Kind> = Vec::with_capacity(els.len());
    let mut unknown_types: HashSet<String> = HashSet::new();
    for (i, j) in els.iter().enumerate() {
        let ty = js(j, "type").unwrap_or("Panel");
        let shape = js(j, "shape").unwrap_or("rect");
        let kind = match ty {
            "Panel" => Kind::Panel,
            "Image" if shape == "ellipse" => Kind::Ellipse,
            "Image" => Kind::Image,
            "Text" => Kind::Text,
            "Button" => Kind::Button,
            "Toggle" => Kind::Toggle,
            "Slider" => Kind::Slider,
            "InputField" => Kind::InputField,
            "Dropdown" => Kind::Dropdown,
            "ScrollView" => Kind::ScrollView,
            "Empty" => Kind::Empty,
            other => {
                if unknown_types.insert(other.to_string()) {
                    warnings.add(format!("Unknown element type '{}': imported as Panel", other));
                }
                Kind::Panel
            }
        };
        kinds.push(kind);
        let mut e = Element::new(ids[i], kind);
        e.parent = None;
        let name = js(j, "name").unwrap_or("");
        e.name = clean_label(name, 128).unwrap_or_else(|| kind.label().to_string()).replace('/', "-");
        let un = js(j, "unityName").unwrap_or(name);
        if valid_unity_name(un) {
            e.unity_name = Some(un.to_string());
        }
        e.visible = jb(j, "active").unwrap_or(true);
        e.interactable = jb(j, "interactable").unwrap_or(true);
        e.raycast = jb(j, "raycastTarget").unwrap_or(true);
        e.mask = jb(j, "mask").unwrap_or(false);
        e.value = jf(j, "value").unwrap_or(e.value);
        e.on_click = js(j, "onClick").unwrap_or("").to_string();
        if let Some(o) = j.get("options").and_then(|o| o.as_array()) {
            e.options = o.iter().filter_map(|s| s.as_str().map(|s| s.to_string())).take(500).collect();
        }
        let empty = serde_json::Value::Null;
        let im = j.get("image").unwrap_or(&empty);
        if let Some(c) = jcolor(im, "color") {
            e.fill = c;
        }
        if let Some(c) = jcolor(im, "gradientTo") {
            e.fill2 = c;
        }
        e.opacity = jf(im, "alpha").unwrap_or(1.0).clamp(0.0, 1.0);
        e.preserve_aspect = jb(im, "preserveAspect").unwrap_or(false);
        e.radius = jf(im, "radius").unwrap_or(0.0).max(0.0);
        if let Some(src) = js(im, "source").filter(|s| !s.is_empty()) {
            e.image = if src.starts_with("Assets/") || src.starts_with("Packages/") { format!("unity:{}", src) } else { src.to_string() };
        }
        let ol = j.get("outline").unwrap_or(&empty);
        if let Some(c) = jcolor(ol, "color") {
            e.stroke = c;
        }
        e.stroke_w = jf(ol, "width").unwrap_or(0.0).max(0.0);
        let sh = j.get("shadow").unwrap_or(&empty);
        e.shadow.on = jb(sh, "enabled").unwrap_or(false);
        if let Some(c) = jcolor(sh, "color") {
            e.shadow.color = c;
        }
        if let Some(o) = jv2(sh, "offset") {
            e.shadow.offset = o;
        }
        let tx = j.get("text").unwrap_or(&empty);
        e.text = js(tx, "value").unwrap_or("").to_string();
        if let Some(s) = jf(tx, "fontSize") {
            e.font_size = s.clamp(1.0, 1000.0);
        }
        if let Some(c) = jcolor(tx, "color") {
            e.text_color = c;
        }
        e.bold = jb(tx, "bold").unwrap_or(false);
        e.italic = jb(tx, "italic").unwrap_or(false);
        e.underline = jb(tx, "underline").unwrap_or(false);
        e.align = tx.get("alignH").and_then(|a| a.as_i64()).unwrap_or(1).clamp(0, 2) as u8;
        e.valign = tx.get("alignV").and_then(|a| a.as_i64()).unwrap_or(1).clamp(0, 2) as u8;
        e.font = js(tx, "font").unwrap_or("").to_string();
        if let Some(l) = j.get("layout") {
            e.layout = json_layout(l);
        }
        if let Some(sl) = jarr(j, "slice", 4) {
            e.slice = Some([sl[0], sl[1], sl[2], sl[3]]);
        }
        e.ppu = jf(j, "ppu").filter(|p| *p > 0.0);
        e.sprite = js(j, "sprite").map(|s| s.to_string());
        e.image_mode = match js(j, "imageMode") {
            Some("Sliced") => Some(ImageMode::Sliced),
            Some("Tiled") => Some(ImageMode::Tiled),
            Some("Filled") => Some(ImageMode::Filled),
            Some("Simple") => Some(ImageMode::Simple),
            _ => None,
        };
        // parent
        let parent_idx = j.get("parent").and_then(|p| p.as_i64()).unwrap_or(-1);
        if parent_idx >= 0 && (parent_idx as usize) < i {
            e.parent = Some(ids[parent_idx as usize]);
        } else if parent_idx >= 0 {
            warnings.add(format!("Element '{}' names a parent that does not come before it: moved to the top level", e.name));
        }
        // geometry: the inverse of the exporter (ScrollView children are measured against their Content)
        let r = j.get("rect").unwrap_or(&empty);
        let mut rt = RectTransform { anchor_min: [0.5, 0.5], anchor_max: [0.5, 0.5], pivot: [0.5, 0.5], anchored_position: [0.0, 0.0], size_delta: [0.0, 0.0], rotation_z: 0.0 };
        let fix2 = |v: [f32; 2], lim: f32| [clampv(v[0], lim), clampv(v[1], lim)];
        if let Some(v) = jv2(r, "anchorMin") {
            rt.anchor_min = fix2(v, 100.0);
        }
        if let Some(v) = jv2(r, "anchorMax") {
            rt.anchor_max = fix2(v, 100.0);
        }
        if let Some(v) = jv2(r, "pivot") {
            rt.pivot = fix2(v, 100.0);
        }
        if let Some(v) = jv2(r, "anchoredPosition") {
            rt.anchored_position = fix2(v, 1.0e6);
        }
        match jv2(r, "sizeDelta") {
            Some(v) => rt.size_delta = fix2(v, 1.0e6),
            None => {
                if let Some(s) = jv2(j, "size") {
                    rt.size_delta = fix2(s, 1.0e6);
                }
            }
        }
        rt.rotation_z = clampv(jf(r, "rotationZ").unwrap_or(0.0), 100_000.0);
        let pf = if parent_idx >= 0 && (parent_idx as usize) < i {
            let pi = parent_idx as usize;
            let pframe = frames[pi];
            if kinds[pi] == Kind::ScrollView {
                let ch = jf(els[pi], "contentHeight").filter(|h| *h > 0.0).unwrap_or(pframe.h).max(0.0);
                let c = pframe.to_world([0.0, -pframe.h / 2.0 + ch / 2.0]);
                Frame { cx: c[0], cy: c[1], w: pframe.w, h: ch, rot: pframe.rot }
            } else {
                pframe
            }
        } else {
            doc.page_frame()
        };
        apply_rect_transform(&mut e, &rt, &pf);
        frames.push(e.frame());
        doc.elements.push(e);
    }
    doc.next_id = used_ids.iter().copied().max().unwrap_or(0) + 1;

    // portrait layout
    if let Some(p) = v.get("portrait") {
        let mut pl = PortraitLayout::default();
        pl.width = jf(p, "width").unwrap_or(1080.0);
        pl.height = jf(p, "height").unwrap_or(1920.0);
        for o in p.get("overrides").and_then(|o| o.as_array()).map(|a| a.as_slice()).unwrap_or(&[]).iter().take(MAX_ELEMENTS) {
            let Some(id) = o.get("id").and_then(|i| i.as_u64()) else { continue };
            let mut ov = ElementOverride::new(id);
            ov.x = jf(o, "x");
            ov.y = jf(o, "y");
            ov.w = jf(o, "w");
            ov.h = jf(o, "h");
            ov.rot = jf(o, "rot");
            ov.font_size = jf(o, "fontSize");
            ov.anchor_min = jv2(o, "anchorMin");
            ov.anchor_max = jv2(o, "anchorMax");
            ov.pivot = jv2(o, "pivot");
            ov.visible = jb(o, "visible");
            ov.layout = o.get("layout").map(json_layout);
            pl.overrides.push(ov);
        }
        doc.portrait = Some(pl);
    }
    let mut doc = doc;
    doc.migrate_groups();
    Ok(Imported { doc, warnings: warnings.finish() })
}

// ===========================================================================
// 8. Public API
// ===========================================================================

fn read_bounded(path: &Path) -> Result<String, String> {
    let md = std::fs::metadata(path).map_err(|e| format!("{}: {}", path.display(), e))?;
    if !md.is_file() {
        return Err(format!("{}: not a file", path.display()));
    }
    if md.len() > MAX_FILE_BYTES {
        return Err(format!("{}: file too large ({} MB, the limit is {} MB)", path.display(), md.len() / (1024 * 1024), MAX_FILE_BYTES / (1024 * 1024)));
    }
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {}", path.display(), e))?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn title_of(path: &Path) -> String {
    path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "Imported design".to_string())
}

/// Imports a `.prefab` (uGUI YAML) or a Lienzo export `.json` into a design.
pub fn import_file(path: &Path) -> Result<Imported, String> {
    let ext = path.extension().map(|e| e.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
    match ext.as_str() {
        "prefab" => {
            let text = read_bounded(path)?;
            let mut assets = Assets::for_file(path);
            import_prefab_text(&text, &title_of(path), &mut assets).map_err(|e| format!("{}: {}", path.display(), e))
        }
        "json" => {
            let text = read_bounded(path)?;
            import_export_json(&text, &title_of(path)).map_err(|e| format!("{}: {}", path.display(), e))
        }
        _ => Err(format!("{}: unsupported file type (use .prefab or .json)", path.display())),
    }
}

/// File name safe on Windows, macOS and Linux (no separators, no reserved device names, no trailing dot / space).
pub fn sanitize_file_name(s: &str) -> String {
    let mut t: String = s.chars().map(|c| if c.is_control() || matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') { '_' } else { c }).collect();
    t = t.trim().trim_end_matches(['.', ' ']).trim_start_matches('.').to_string();
    if t.chars().count() > 100 {
        t = t.chars().take(100).collect();
    }
    let stem = t.split('.').next().unwrap_or("").trim().to_ascii_uppercase();
    let reserved = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL") || ((stem.starts_with("COM") || stem.starts_with("LPT")) && stem.len() == 4 && stem.as_bytes()[3].is_ascii_digit() && stem.as_bytes()[3] != b'0');
    if t.is_empty() {
        return "prefab".to_string();
    }
    if reserved {
        t.insert(0, '_');
    }
    t
}

/// What `import_unity_project_report` did with each prefab.
#[derive(Default)]
pub struct ProjectReport {
    /// (file written inside the output folder, number of elements)
    pub written: Vec<(String, usize)>,
    /// prefabs that are not UI (no RectTransform), relative to the project
    pub skipped: Vec<String>,
    /// (prefab, error)
    pub failed: Vec<(String, String)>,
    /// (prefab, warning)
    pub warnings: Vec<(String, String)>,
}

fn collect_prefabs(assets: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![(assets.to_path_buf(), 0usize)];
    while let Some((d, depth)) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else { continue };
        for ent in rd.flatten() {
            let Ok(ft) = ent.file_type() else { continue };
            if ft.is_symlink() {
                continue;
            }
            let p = ent.path();
            if ft.is_dir() {
                let n = ent.file_name().to_string_lossy().to_string();
                if depth < 40 && !n.starts_with('.') && !n.ends_with('~') {
                    stack.push((p, depth + 1));
                }
            } else if p.extension().is_some_and(|e| e.eq_ignore_ascii_case("prefab")) {
                out.push(p);
                if out.len() >= MAX_PREFABS {
                    return out;
                }
            }
        }
    }
    out.sort();
    out
}

/// Converts every UI prefab under `dir/Assets` into `<name>.editorgui.json` inside `out_dir`.
pub fn import_unity_project_report(dir: &Path, out_dir: &Path) -> Result<ProjectReport, String> {
    let assets_dir = dir.join("Assets");
    if !assets_dir.is_dir() {
        return Err(format!("{}: not a Unity project (no Assets folder)", dir.display()));
    }
    std::fs::create_dir_all(out_dir).map_err(|e| format!("{}: {}", out_dir.display(), e))?;
    let prefabs = collect_prefabs(&assets_dir);
    let mut assets = Assets::for_project(dir);
    let mut rep = ProjectReport::default();
    let mut used: HashSet<String> = HashSet::new();
    for p in prefabs {
        let rel = p.strip_prefix(dir).unwrap_or(&p).to_string_lossy().replace('\\', "/");
        let text = match read_bounded(&p) {
            Ok(t) => t,
            Err(e) => {
                rep.failed.push((rel, e));
                continue;
            }
        };
        if !text.contains("--- !u!224 ") {
            rep.skipped.push(rel);
            continue;
        }
        let imp = match import_prefab_text(&text, &title_of(&p), &mut assets) {
            Ok(i) => i,
            Err(e) if e.starts_with("no RectTransform") => {
                rep.skipped.push(rel);
                continue;
            }
            Err(e) => {
                rep.failed.push((rel, e));
                continue;
            }
        };
        if imp.doc.elements.is_empty() {
            rep.skipped.push(rel);
            continue;
        }
        let base = sanitize_file_name(&title_of(&p));
        let mut name = base.clone();
        let mut n = 2;
        while !used.insert(name.to_lowercase()) {
            name = format!("{}_{}", base, n);
            n += 1;
        }
        let file = format!("{}.editorgui.json", name);
        let json = serde_json::to_string_pretty(&imp.doc).map_err(|e| e.to_string())?;
        let target = out_dir.join(&file);
        std::fs::write(&target, json).map_err(|e| format!("{}: {}", target.display(), e))?;
        for w in imp.warnings {
            rep.warnings.push((rel.clone(), w));
        }
        rep.written.push((file, imp.doc.elements.len()));
    }
    Ok(rep)
}

/// Like [`import_unity_project_report`] but returns only (file written, element count) per converted prefab.
pub fn import_unity_project(dir: &Path, out_dir: &Path) -> Result<Vec<(String, usize)>, String> {
    import_unity_project_report(dir, out_dir).map(|r| r.written)
}
