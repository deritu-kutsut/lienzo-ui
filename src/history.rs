//! Bounded undo / redo history: capped by number of steps AND by an approximate memory budget.
//! Each step is a full design snapshot; when the budget is exceeded the oldest steps are dropped.

use crate::model::{Doc, Element};
use std::collections::VecDeque;

pub const MAX_STEPS: usize = 200;
pub const MAX_BYTES: usize = 64 * 1024 * 1024;

/// Cheap estimate of the memory a design snapshot takes (struct size + the heap of its strings), no allocation.
pub fn approx_bytes(d: &Doc) -> usize {
    let opt = |s: &Option<String>| s.as_ref().map_or(0, |s| s.len());
    let mut n = std::mem::size_of::<Doc>() + d.title.len();
    for e in &d.elements {
        n += std::mem::size_of::<Element>() + e.name.len() + e.text.len() + e.font.len() + e.on_click.len() + e.image.len() + opt(&e.unity_name) + opt(&e.sprite);
        n += e.options.iter().map(|o| o.len() + std::mem::size_of::<String>()).sum::<usize>();
    }
    if let Some(p) = &d.portrait {
        n += std::mem::size_of_val(p.overrides.as_slice());
    }
    let u = &d.unity;
    n += opt(&u.scripts_dir) + opt(&u.assembly) + opt(&u.default_font);
    n += u.fallback_fonts.iter().map(|f| f.len() + std::mem::size_of::<String>()).sum::<usize>();
    n
}

pub struct History {
    items: VecDeque<(Doc, usize)>,
    bytes: usize,
    max_steps: usize,
    max_bytes: usize,
    trimmed: bool,
}

impl Default for History {
    fn default() -> Self {
        Self::with_limits(MAX_STEPS, MAX_BYTES)
    }
}

impl History {
    pub fn with_limits(max_steps: usize, max_bytes: usize) -> Self {
        History { items: VecDeque::new(), bytes: 0, max_steps: max_steps.max(1), max_bytes, trimmed: false }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn bytes(&self) -> usize {
        self.bytes
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.bytes = 0;
    }

    /// Adds a step, then drops the oldest ones while over the step or byte budget (the newest step is always kept).
    pub fn push(&mut self, d: Doc) {
        let size = approx_bytes(&d);
        self.bytes += size;
        self.items.push_back((d, size));
        while self.items.len() > self.max_steps || (self.bytes > self.max_bytes && self.items.len() > 1) {
            if let Some((_, s)) = self.items.pop_front() {
                self.bytes = self.bytes.saturating_sub(s);
                self.trimmed = true;
            }
        }
    }

    pub fn pop(&mut self) -> Option<Doc> {
        let (d, s) = self.items.pop_back()?;
        self.bytes = self.bytes.saturating_sub(s);
        Some(d)
    }

    /// True once after the oldest steps were dropped since the last call.
    pub fn take_trimmed(&mut self) -> bool {
        std::mem::take(&mut self.trimmed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Kind;

    fn big(n: usize) -> Doc {
        let mut d = Doc::empty(1920.0, 1080.0);
        for i in 0..n {
            let mut e = Element::new(i as u64 + 1, Kind::Text);
            e.text = format!("text {}", i);
            d.elements.push(e);
        }
        d.next_id = n as u64 + 1;
        d
    }

    #[test]
    fn caps_steps_and_lifo_order() {
        let mut h = History::with_limits(3, usize::MAX);
        for i in 0..5 {
            let mut d = Doc::empty(10.0, 10.0);
            d.title = i.to_string();
            h.push(d);
        }
        assert_eq!(h.len(), 3);
        assert!(h.take_trimmed());
        assert!(!h.take_trimmed(), "reported once");
        assert_eq!(h.pop().unwrap().title, "4");
        assert_eq!(h.pop().unwrap().title, "3");
        assert_eq!(h.pop().unwrap().title, "2");
        assert!(h.pop().is_none() && h.bytes() == 0);
    }

    #[test]
    fn approx_bytes_counts_every_string_field() {
        let base = approx_bytes(&big(3));
        let mut d = big(3);
        d.elements[0].unity_name = Some("u".repeat(1000));
        d.elements[1].sprite = Some("s".repeat(2000));
        d.unity.scripts_dir = Some("d".repeat(300));
        d.unity.fallback_fonts = vec!["f".repeat(400)];
        assert!(approx_bytes(&d) >= base + 1000 + 2000 + 300 + 400);
        let mut p = big(3);
        p.portrait = Some(crate::docext::PortraitLayout { overrides: (0..100).map(crate::docext::ElementOverride::new).collect(), ..Default::default() });
        assert!(approx_bytes(&p) >= base + 100 * std::mem::size_of::<crate::docext::ElementOverride>());
    }

    #[test]
    fn caps_bytes_but_keeps_the_newest() {
        let one = approx_bytes(&big(100));
        let mut h = History::with_limits(1000, one * 3 + one / 2);
        for _ in 0..10 {
            h.push(big(100));
        }
        assert_eq!(h.len(), 3);
        assert!(h.bytes() <= one * 4);
        // a single snapshot over the budget is still kept
        let mut h = History::with_limits(10, 10);
        h.push(big(50));
        assert_eq!(h.len(), 1);
    }

    #[test]
    fn two_thousand_elements_and_500_edits_stay_under_the_budget() {
        let mut h = History::default();
        let mut d = big(2000);
        for i in 0..500 {
            d.elements[i % 2000].x += 1.0;
            h.push(d.clone());
        }
        assert!(h.len() <= MAX_STEPS);
        assert!(h.bytes() <= MAX_BYTES, "{} bytes", h.bytes());
        assert!(h.take_trimmed(), "500 steps exceed the 200-step cap");
    }

    #[test]
    fn editor_history_is_bounded_and_notifies_once() {
        let _l = crate::i18n::test_lock();
        let mut a = crate::app::EditorApp::new(None);
        a.settings.read_only = true;
        a.doc = big(10);
        for _ in 0..4 {
            a.commit_state(false); // settle
        }
        for i in 0..(MAX_STEPS + 20) {
            a.doc.elements[0].x = i as f32 + 0.5;
            a.commit_state(false);
        }
        assert_eq!(a.undo.len(), MAX_STEPS);
        // other tests (`EditorApp::new` applies the saved language) can switch the global language at any time: accept the notice in any of them
        let notice = "History trimmed: the oldest undo steps were dropped";
        assert!(crate::i18n::LANGUAGES.iter().any(|(c, _, _)| crate::i18n::t_in(c, notice) == a.status), "{:?}", a.status);
        a.status.clear();
        a.doc.elements[0].x = 9999.0;
        a.commit_state(false);
        assert!(a.status.is_empty(), "the notice appears only once");
        a.undo();
        assert_eq!(a.doc.elements[0].x, (MAX_STEPS + 19) as f32 + 0.5);
        a.redo();
        assert_eq!(a.doc.elements[0].x, 9999.0);
    }
}
