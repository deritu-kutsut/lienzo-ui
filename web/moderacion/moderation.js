// Lienzo UI moderation web. No libraries or build step.
// Security: everything coming from users is rendered with textContent (never as HTML); the session lives in sessionStorage
// (cleared when the tab closes); the CSP policy in index.html prevents loading anything outside this site and Supabase.
"use strict";

const SUPABASE_URL = "https://btotvbdhoyfjpdixlcrw.supabase.co";
const PUBLIC_KEY = "sb_publishable_UNvWtzs_M8N_N6fLrZ9VCQ_E6hkxXrL"; // public by design
const RETURN_URL = location.origin + location.pathname;
const $ = (id) => document.getElementById(id);

// ───────── utilities ─────────
function el(tag, props = {}, ...children) {
  const e = document.createElement(tag);
  for (const [k, v] of Object.entries(props)) {
    if (k === "class") e.className = v;
    else if (k === "text") e.textContent = v;
    else if (k.startsWith("on")) e.addEventListener(k.slice(2), v);
    else if (v !== false && v !== undefined) e.setAttribute(k, v === true ? "" : v);
  }
  for (const c of children) if (c) e.append(c);
  return e;
}
function setStatus(text, error = false) {
  const e = $("status");
  e.textContent = text || "";
  e.className = error ? "error" : "";
}
function b64url(bytes) {
  let s = "";
  for (const b of bytes) s += String.fromCharCode(b);
  return btoa(s).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}
function randomBytes(n) {
  return crypto.getRandomValues(new Uint8Array(n));
}
const saveSession = (s) => sessionStorage.setItem("lm_session", JSON.stringify(s));
const loadSession = () => {
  try { return JSON.parse(sessionStorage.getItem("lm_session") || "null"); } catch { return null; }
};
// Only signed links from our Supabase are followed: nothing a user wrote.
const safeUrl = (u) => typeof u === "string" && u.startsWith(SUPABASE_URL + "/storage/v1/object/sign/") ? u : null;
// Public (already published) files: only <kind>/<uuid>/<file> of the public bucket, with plain characters (no «..», no encoded dots).
const PUBLIC_URL = new RegExp("^" + SUPABASE_URL.replace(/[.]/g, "\\.") + "/storage/v1/object/public/publico/(plugin|image|font|template|element|theme|imagen|fuente|plantilla|elemento|tema)/[0-9a-f]{8}(-[0-9a-f]{4}){3}-[0-9a-f]{12}/[A-Za-z0-9_-]([A-Za-z0-9._%-]{0,150}[A-Za-z0-9_-])?$");
const safePublicUrl = (u) => typeof u === "string" && PUBLIC_URL.test(u) && !u.includes("..") && !/%2e/i.test(u) ? u : null;

// ───────── session ─────────
let session = loadSession();

async function signInOAuth(provider) {
  const verifier = b64url(randomBytes(48));
  const challenge = b64url(new Uint8Array(await crypto.subtle.digest("SHA-256", new TextEncoder().encode(verifier))));
  sessionStorage.setItem("lm_pkce", verifier);
  const q = new URLSearchParams({ provider, redirect_to: RETURN_URL, code_challenge: challenge, code_challenge_method: "s256" });
  location.href = `${SUPABASE_URL}/auth/v1/authorize?${q}`;
}

async function exchangeCode(code) {
  const verifier = sessionStorage.getItem("lm_pkce");
  sessionStorage.removeItem("lm_pkce");
  history.replaceState(null, "", RETURN_URL); // the code does not stay in the address bar
  if (!verifier || !/^[A-Za-z0-9_-]{1,300}$/.test(code)) throw new Error("Invalid sign-in.");
  const r = await fetch(`${SUPABASE_URL}/auth/v1/token?grant_type=pkce`, {
    method: "POST",
    headers: { "Content-Type": "application/json", apikey: PUBLIC_KEY },
    body: JSON.stringify({ auth_code: code, code_verifier: verifier }),
  });
  const j = await r.json().catch(() => ({}));
  if (!r.ok || !j.access_token) throw new Error("Could not sign in.");
  session = { type: "person", access: j.access_token };
  saveSession(session);
}

function signOut() {
  sessionStorage.clear();
  session = null;
  render();
  setStatus("");
}

// ───────── API ─────────
// Wire format v2 (English). The function also still answers the old Spanish format to requests without `v: 2`.
async function api(action, body = {}) {
  if (!session) throw new Error("No session.");
  const credential = session.type === "token" ? session.token : session.access;
  const r = await fetch(`${SUPABASE_URL}/functions/v1/review`, {
    method: "POST",
    headers: { "Content-Type": "application/json", Authorization: `Bearer ${credential}`, apikey: PUBLIC_KEY },
    body: JSON.stringify({ v: 2, action, ...body }),
  });
  const j = await r.json().catch(() => ({}));
  if (r.status === 401) {
    signOut();
    throw new Error("The session expired or the credential is no longer valid. Sign in again.");
  }
  if (!r.ok) {
    const e = new Error(j.error || `Error ${r.status}`);
    e.status = r.status;
    throw e;
  }
  return j;
}

// ───────── screen ─────────
let tab = "pending";

function render() {
  const signedIn = !!session;
  $("sign-in").hidden = signedIn;
  $("nav").hidden = !signedIn;
  $("tab-tokens").hidden = !(signedIn && session.type === "person");
  $("pending").hidden = !(signedIn && tab === "pending");
  $("claims").hidden = !(signedIn && tab === "claims");
  $("tokens").hidden = !(signedIn && tab === "tokens");
  $("tab-pending").className = tab === "pending" ? "active" : "";
  $("tab-claims").className = tab === "claims" ? "active" : "";
  $("tab-tokens").className = tab === "tokens" ? "active" : "";
  if (signedIn) (tab === "pending" ? loadPending() : tab === "claims" ? loadClaims() : loadTokens());
}

async function act(action, body, okMessage) {
  setStatus("Working…");
  try {
    await api(action, body);
    setStatus(okMessage);
    await loadPending(false);
  } catch (e) {
    setStatus(e.message, true);
    // 409: someone else decided it, or the author edited it after the list was loaded: show the current version.
    if (e.status === 409) await loadPending(false);
  }
}

function image(url, alt) {
  const u = safeUrl(url) || safePublicUrl(url);
  return u ? el("img", { src: u, alt, loading: "lazy", referrerpolicy: "no-referrer" }) : null;
}

async function showContent(url, target, button) {
  const u = safeUrl(url);
  if (!u) return;
  button.disabled = true;
  try {
    const r = await fetch(u);
    const text = (await r.text()).slice(0, 200000);
    target.textContent = text; // as text: even if the file contains HTML or scripts, they do not run here
    target.hidden = false;
  } catch {
    setStatus("Could not read the file.", true);
  }
  button.disabled = false;
}

function contentCard(p) {
  const fileText = el("pre", { hidden: true });
  const reason = el("input", { type: "text", placeholder: "Rejection reason (the person will see it)", maxlength: "300" });
  const isText = ["plugin", "template", "element", "theme"].includes(p.kind);
  const viewButton = el("button", { text: "View content", onclick: (ev) => showContent(p.temp_url, fileText, ev.target) });
  // The author's CURRENT public name is not shown publicly until it is reviewed: the moderator can accept it together with the approval.
  const acceptName = p.pending_name ? el("input", { type: "checkbox", id: `name-${p.id}` }) : null;
  const publicLink = safePublicUrl(p.public_url);
  // approve sends `updated_at` (the version of the text shown here): if the author edits it meanwhile the server answers 409 and the list reloads.
  const approve = (extra, okMessage) => act("approve", { item: p.id, updated_at: p.updated_at || undefined, approve_name: acceptName && acceptName.checked ? true : undefined, ...extra }, okMessage);
  return el("div", { class: "card" },
    el("h2", { text: p.title }, p.is_18 ? el("span", { class: "badge18", text: "+18" }) : null,
      p.edited_after_approval ? el("span", { class: "badge18", text: "edited after approval" }) : null),
    el("div", { class: "meta", text: `${p.kind} · by ${p.author_name || "—"} · ${p.bytes ?? "?"} bytes · ${p.created_at}${p.updated_at ? ` · text version ${p.updated_at}` : ""}` }),
    p.pending_name ? el("div", { class: "reason", text: `New public name waiting for review: «${p.pending_name}» (shown publicly now: «${p.author_name || "—"}»)` }) : null,
    p.edited_after_approval ? el("div", { class: "reason", text: "There is no new file: the author edited the text of an approved item (or it was hidden by reports). Check the current title, description and tags below; the published files are the ones linked here." }) : null,
    p.rejection_reason ? el("div", { class: "reason", text: `Why it is pending: ${p.rejection_reason}` }) : null,
    p.description ? el("p", { text: p.description }) : null,
    p.tags && p.tags.length ? el("div", { class: "tags", text: p.tags.map((t) => "#" + t).join(" ") }) : null,
    el("div", { class: "images" }, p.kind === "image" ? image(p.temp_url, "Uploaded image") : null, image(p.icon_url, "Icon"), image(p.preview_url, "Preview"),
      p.edited_after_approval && p.kind === "image" ? image(p.public_url, "Published image") : null, p.edited_after_approval ? image(p.public_icon_url, "Published icon") : null,
      p.edited_after_approval ? image(p.public_preview_url, "Published preview") : null),
    p.edited_after_approval && publicLink ? el("a", { href: publicLink, rel: "noopener noreferrer", target: "_blank", text: "Open the published file" }) : null,
    acceptName ? el("div", { class: "row" }, acceptName, el("label", { for: `name-${p.id}`, text: `Also accept the public name «${p.pending_name}»` })) : null,
    isText ? viewButton : null,
    p.kind === "font" && safeUrl(p.temp_url) ? el("a", { href: safeUrl(p.temp_url), rel: "noopener noreferrer", target: "_blank", text: "Download the font (temporary link)" }) : null,
    fileText,
    el("div", { class: "actions" },
      el("button", { class: "ok", text: "Approve", onclick: () => approve({}, "Approved and published.") }),
      el("button", { class: "ok", text: "Approve as +18", onclick: () => approve({ is_18: true }, "Approved as +18.") }),
      reason,
      el("button", { class: "bad", text: "Reject", onclick: () => act("reject", { item: p.id, reason: reason.value.trim() || "Rejected in review." }, "Rejected.") })));
}

function profileCard(p) {
  const reason = el("input", { type: "text", placeholder: "Reason (optional)", maxlength: "300" });
  return el("div", { class: "card" },
    el("h2", { text: `${p.class === "avatar" ? "Profile photo" : "Banner"} of ${p.public_name}` }),
    p.bio ? el("p", { class: "meta", text: p.bio }) : null,
    p.scanned === false ? el("div", { class: "reason", text: "This image was never scanned: approving is refused until it has been rescanned." }) : null,
    el("div", { class: "images" }, image(p.temp_url, p.class)),
    el("div", { class: "actions" },
      el("button", { class: "ok", text: "Approve", onclick: () => act("approve_profile", { user: p.user, class: p.class }, "Profile image approved.") }),
      reason,
      el("button", { class: "bad", text: "Reject", onclick: () => act("reject_profile", { user: p.user, class: p.class }, "Profile image rejected.") })));
}

const CATEGORIES = [
  ["plugin", "Plugins"], ["theme", "Themes"], ["template", "Templates"], ["element", "Elements"], ["font", "Fonts"], ["image", "Images"],
  ["profile", "Profile photos and banners"],
];
let category = "all";
let latest = null; // last list received, to switch category without requesting it again

function renderPending() {
  const box = $("pending");
  if (!latest) return;
  const byCategory = {};
  for (const p of latest.pending) (byCategory[p.kind] ||= []).push(p);
  byCategory.profile = latest.profiles;
  const total = latest.pending.length + latest.profiles.length;
  const chips = [["all", "All", total], ...CATEGORIES.map(([k, n]) => [k, n, (byCategory[k] || []).length])];
  const bar = el("div", { class: "row" }, ...chips.map(([k, n, c]) =>
    el("button", { class: category === k ? "active" : "", text: `${n} (${c})`, onclick: () => { category = k; renderPending(); } })));
  const sections = [];
  for (const [k, name] of CATEGORIES) {
    const list = byCategory[k] || [];
    if (!list.length || (category !== "all" && category !== k)) continue;
    sections.push(el("h3", { class: "section", text: `${name} (${list.length})` }));
    sections.push(...list.map(k === "profile" ? profileCard : contentCard));
  }
  box.replaceChildren(
    el("p", { class: "notice", text: `${total} pending in total. ${latest.note || ""}` }),
    el("div", { class: "row" }, el("button", { text: "Refresh", onclick: () => loadPending() })),
    bar,
    ...(sections.length ? sections : [el("p", { class: "meta", text: "Nothing is pending in this category." })]));
}

async function loadPending(announce = true) {
  try {
    latest = await api("list");
    renderPending();
    if (announce) setStatus("");
  } catch (e) {
    setStatus(e.message, true);
  }
}

// ───────── copyright claims ─────────
// Everything in a claim (claimant, contact, description, counter-notice, titles, names) is user content: textContent only.
const when = (t) => (t ? String(t).slice(0, 16).replace("T", " ") : "—");
const BAN_FOREVER = "9999-";

async function claimAct(action, body, okMessage) {
  setStatus("Working…");
  try {
    const j = await api(action, body);
    setStatus(j.restored === false && j.restore_error ? `${okMessage} The item could not be restored automatically (${j.restore_error}): open the Pending tab.` : okMessage);
    await loadClaims(false);
  } catch (e) {
    setStatus(e.message, true);
    if (e.status === 409) await loadClaims(false);
  }
}

function accountLine(a) {
  if (!a) return "author unknown (account deleted)";
  const ban = a.upload_banned_until ? (String(a.upload_banned_until).startsWith(BAN_FOREVER) ? "upload ban: permanent" : `upload ban until ${when(a.upload_banned_until)}`) : "no upload ban";
  return `${a.public_name || "—"} · strikes: ${a.strikes ?? 0} of 3 · ${ban}${a.suspended ? " · SUSPENDED" : ""}`;
}

function claimCard(c) {
  const note = el("textarea", { rows: "3", maxlength: "2000", placeholder: "Counter-notice text received from the uploader (copy it here)" });
  const force = c.status === "counter" ? el("input", { type: "checkbox", id: `force-${c.id}` }) : null;
  return el("div", { class: "card" },
    el("h2", { text: c.item ? c.item.title : "(item deleted)" }, el("span", { class: "badge18", text: c.status === "counter" ? "counter-notice" : "open" })),
    el("div", { class: "meta", text: `${c.item ? `${c.item.kind} · by ${c.item.author_name || "—"} · ${c.item.status}` : "—"} · filed ${when(c.created_at)}` }),
    el("div", { class: "reason", text: accountLine(c.author) }),
    el("p", { text: `Claimant: ${c.claimant}${c.claimant_contact ? ` · contact: ${c.claimant_contact}` : c.claimant_contact_masked ? " · contact: hidden (your credential cannot decide claims)" : ""}` }),
    el("div", { class: "meta", text: `Filed by: ${c.reporter ? `${c.reporter.public_name || "(name unavailable)"} (${c.reporter.id})` : "a moderator, or the account was deleted"}` }),
    c.flag === "author_flood" ? el("div", { class: "reason", text: "FLAGGED: more than 3 claims against this author in 24 h. The item was NOT hidden: check for a harassment campaign before deciding." }) : null,
    c.item && c.item.edited_while_hidden ? el("div", { class: "reason", text: "The title, description or tags were EDITED after this claim was filed. Rejecting the claim will NOT republish it: it stays in the Pending tab for a normal review." }) : null,
    el("p", { text: c.description }),
    c.counter_notice ? el("div", { class: "reason", text: `Counter-notice (${when(c.counter_at)}): ${c.counter_notice}` }) : null,
    c.restore_after ? el("div", { class: "meta", text: `Earliest restore without override: ${when(c.restore_after)}` }) : null,
    el("div", { class: "actions" },
      el("button", { class: "bad", text: "Uphold: take down + strike", onclick: () => {
        if (confirm("Take the item down for good and give the author a strike? At 3 strikes uploads are disabled.")) claimAct("claim_uphold", { claim: c.id }, "Claim upheld: item removed and strike recorded.");
      } }),
      el("button", { class: "ok", text: "Reject claim: restore item", onclick: () => claimAct("claim_reject", { claim: c.id, force: force && force.checked ? true : undefined }, "Claim rejected.") }),
      force ? el("label", { for: `force-${c.id}`, text: "Override the waiting period" }) : null,
      force ? force : null),
    c.status === "open" ? el("div", {}, note, el("div", { class: "actions" },
      el("button", { text: "Record counter-notice", onclick: () => claimAct("claim_counter", { claim: c.id, note: note.value.trim() }, "Counter-notice recorded: the item stays hidden.") }))) : null);
}

async function loadClaims(announce = true) {
  const box = $("claims");
  try {
    const j = await api("claim_list");
    const accounts = j.accounts || [];
    box.replaceChildren(
      el("p", { class: "notice", text: `${j.claims.length} open or counter-noticed claim(s). ${j.note || ""}` }),
      el("div", { class: "row" }, el("button", { text: "Refresh", onclick: () => loadClaims() })),
      ...(j.claims.length ? j.claims.map(claimCard) : [el("p", { class: "meta", text: "No open claims." })]),
      el("h3", { class: "section", text: `Accounts with strikes, bans or suspension (${accounts.length})` }),
      accounts.length ? el("table", {},
        el("thead", {}, el("tr", {}, ...["Name", "Strikes", "Upload ban", "Suspended"].map((t) => el("th", { text: t })))),
        el("tbody", {}, ...accounts.map((a) => el("tr", {},
          el("td", { text: a.public_name || a.id }), el("td", { text: String(a.strikes ?? 0) }),
          el("td", { text: a.upload_banned_until ? (String(a.upload_banned_until).startsWith(BAN_FOREVER) ? "permanent" : `until ${when(a.upload_banned_until)}`) : "—" }),
          el("td", { text: a.suspended ? "yes" : "no" }))))) : el("p", { class: "meta", text: "None." }));
    if (announce) setStatus("");
  } catch (e) {
    setStatus(e.message, true);
  }
}

// ───────── tokens (people only) ─────────
async function loadTokens() {
  const box = $("tokens");
  try {
    const { tokens } = await api("list_tokens");
    const name = el("input", { type: "text", placeholder: "Name (e.g. «Claude reviewer»)", maxlength: "60" });
    const days = el("input", { type: "number", min: "1", max: "90", value: "30" });
    const checks = ["read", "approve", "reject"].map((a) => ({ a, c: el("input", { type: "checkbox", checked: a !== "approve" }) }));
    const created = el("div");
    const create = el("button", { text: "Create token", onclick: async () => {
      try {
        const scopes = checks.filter((m) => m.c.checked).map((m) => m.a);
        const j = await api("create_token", { name: name.value.trim(), scopes, days: Number(days.value) });
        const field = el("input", { type: "text", readonly: true, value: j.token });
        created.replaceChildren(el("div", { class: "new-token" },
          el("strong", { text: "Token created. Copy it now: it cannot be shown again." }), field,
          el("div", { class: "meta", text: `Expires: ${j.expires_at}` })));
        field.select();
        setStatus("Token created.");
        await loadTokens(); // refreshes the list but keeps the new token notice
        $("tokens").prepend(created);
      } catch (e) { setStatus(e.message, true); }
    } });
    box.replaceChildren(
      el("p", { class: "notice", text: "Credentials for automations and scripts. Give them only what they need: «read» to list, «reject» to take content down and «approve» only if you trust the script. A token cannot create other tokens." }),
      el("label", { text: "Name" }), name,
      el("label", { text: "Scopes" }), el("div", { class: "row" }, ...checks.map((m) => el("label", {}, m.c, ` ${m.a}`))),
      el("label", { text: "Expires in (days, max. 90)" }), days,
      el("div", { class: "row" }, create),
      created,
      el("table", {},
        el("thead", {}, el("tr", {}, ...["Name", "Scopes", "Expires", "Last used", ""].map((t) => el("th", { text: t })))),
        el("tbody", {}, ...tokens.map((t) => el("tr", {},
          el("td", { text: t.name }), el("td", { text: t.scopes.join(", ") }), el("td", { text: t.expires_at.slice(0, 10) }),
          el("td", { text: t.last_used_at ? t.last_used_at.slice(0, 16).replace("T", " ") : "never" }),
          el("td", {}, t.revoked ? el("span", { class: "meta", text: "revoked" }) : el("button", { class: "bad", text: "Revoke", onclick: async () => {
            try { await api("revoke_token", { id: t.id }); setStatus("Token revoked."); await loadTokens(); } catch (e) { setStatus(e.message, true); }
          } })))))));
  } catch (e) {
    setStatus(e.message, true);
  }
}

// ───────── startup ─────────
$("github").addEventListener("click", () => signInOAuth("github"));
$("google").addEventListener("click", () => signInOAuth("google"));
$("sign-out").addEventListener("click", signOut);
$("tab-pending").addEventListener("click", () => { tab = "pending"; render(); });
$("tab-claims").addEventListener("click", () => { tab = "claims"; render(); });
$("tab-tokens").addEventListener("click", () => { tab = "tokens"; render(); });
$("token-sign-in").addEventListener("click", () => {
  const t = $("token").value.trim();
  $("token").value = "";
  if (!/^lm_[A-Za-z0-9_-]{43}$/.test(t)) return setStatus("That does not look like a moderation token (lm_…).", true);
  session = { type: "token", token: t };
  saveSession(session);
  setStatus("");
  render();
});

(async () => {
  const q = new URLSearchParams(location.search);
  if (q.get("code")) {
    try { await exchangeCode(q.get("code")); } catch (e) { setStatus(e.message, true); }
  } else if (q.get("error")) {
    history.replaceState(null, "", RETURN_URL);
    setStatus("Sign-in was cancelled.", true);
  }
  render();
})();
