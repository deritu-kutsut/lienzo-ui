// SQL test for migration 0016 (abuse hardening) in PGlite.
//   PGLITE=file:///<path to pglite dist/index.js> node supabase/tests/0016_abuse_hardening_pglite.mjs
// Two databases replay 0001-0015 and the SAME fixtures and attacks. The first one stays on 0015 and must be VULNERABLE (every attack succeeds: this proves each
// test reproduces the finding); the second gets 0016 and must refuse them. Then legacy client paths, the rollback and a re-apply.
import fs from "node:fs"; import path from "node:path"; import { fileURLToPath } from "node:url";
const here = path.dirname(fileURLToPath(import.meta.url));
const { PGlite } = await import(process.env.PGLITE ?? "@electric-sql/pglite");
const boot = fs.readFileSync(path.join(here, "0012_pglite.mjs"), "utf8").match(/const BOOT = `([\s\S]*?)`;/)[1];
const A = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa", B = "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb", C = "cccccccc-cccc-4ccc-8ccc-cccccccccccc";
const mig = path.join(here, "..", "migrations"), rb = path.join(here, "..", "rollbacks");
const read = (dir, f) => fs.readFileSync(path.join(dir, f), "utf8").replace(/create extension if not exists pgcrypto;/, "");
let fails = 0, passes = 0;
const check = (n, c, x) => { if (c) passes++; else { fails++; console.log("FAIL", n, x === undefined ? "" : JSON.stringify(x)); } };
const nid = () => crypto.randomUUID();

async function mk(label) {
  const db = new PGlite();
  await db.exec(boot.replaceAll("${A}", A).replaceAll("${B}", B).replaceAll("${C}", C)
    + `create table auth.identities(id uuid primary key default gen_random_uuid(), user_id uuid not null references auth.users(id) on delete cascade, provider text not null, identity_data jsonb not null default '{}');`);
  for (const f of fs.readdirSync(mig).filter((f) => /^00(0[1-9]|1[0-5])_/.test(f)).sort()) await db.exec(read(mig, f));
  const as = async (uid, role, sql, params) => {
    await db.exec(`reset role; set role ${role}; select set_config('request.jwt.claim.sub', '${uid ?? ""}', false), set_config('request.jwt.claim.role', '${role}', false);`);
    try { const r = await db.query(sql, params); return { ok: true, rows: r.rows }; } catch (e) { return { ok: false, err: String(e.message), hint: e.hint, code: e.code }; }
    finally { await db.exec("reset role; select set_config('request.jwt.claim.role', '', false), set_config('request.jwt.claim.sub', '', false);"); }
  };
  const owner = async (sql, p) => { try { const r = await db.query(sql, p); return { ok: true, rows: r.rows }; } catch (e) { return { ok: false, err: String(e.message), hint: e.hint }; } };
  const H = { db, label, as, owner, user: (u, s, p) => as(u, "authenticated", s, p), admin: (s, p) => as(null, "service_role", s, p),
    apply: async (f, dir = mig) => { await db.exec(read(dir, f)); } };
  // Accounts. `days` = age of BOTH the auth account and the profile (0 = brand new). `terms` = accepted through accept_terms (terms_version).
  H.acct = async (days, { terms = true, legacyTerms = false, name } = {}) => {
    const u = nid(), nm = name ?? "u" + u.slice(0, 8);
    await owner(`insert into auth.users(id, created_at) values ('${u}', now() - interval '${days} days')`);
    await owner(`insert into public.profiles(id, public_name) values ('${u}', '${nm}')`);
    await owner(`update public.profiles set name_reviewed = public_name, created_at = now() - interval '${days} days'${terms ? ", terms_version = '1.0', terms_accepted_at = now()" : legacyTerms ? ", terms_accepted_at = now()" : ""} where id = '${u}'`);
    return u;
  };
  H.item = async (author, { files = true } = {}) => {
    const id = nid();
    await owner(`delete from public.upload_log where author = '${author}'`); // the fixtures create far more than 20 items per author
    const ins = await owner(`insert into public.community_items(id, author, kind, title) values ('${id}', '${author}', 'template', 'Item ${id.slice(0, 6)}')`);
    if (!ins.ok) throw new Error("fixture item failed: " + ins.err);
    await H.admin(`update public.community_items set status = 'approved', reviewed_at = now() - interval '1 hour'${files ? `, public_path = 'template/${id}/f.json', public_icon = 'template/${id}/icon.png', public_preview = 'template/${id}/preview.png'` : ""} where id = '${id}'`);
    return id;
  };
  H.status = async (id) => (await owner(`select status::text s, rejection_reason r from public.community_items where id = '${id}'`)).rows[0];
  H.queued = async (id) => (await owner(`select action, reason from public.storage_cleanup where item = '${id}' and done_at is null order by path`)).rows;
  return H;
}

const old = await mk("0015"), neu = await mk("0016");
for (const H of [old, neu]) {
  await H.owner(`insert into public.profiles (id, public_name, terms_accepted_at) values ('${A}', 'alice', now()), ('${B}', 'bobby', null), ('${C}', 'carol', now()); update public.profiles set name_reviewed = public_name;`);
}
// Fixture that must be repaired by 0016: a profile whose created_at was forged to be OLDER than its auth account (an attack that worked on 0015).
const FORGED = nid();
for (const H of [old, neu]) {
  await H.owner(`insert into auth.users(id, created_at) values ('${FORGED}', now() - interval '1 hour')`);
  await H.owner(`insert into public.profiles(id, public_name) values ('${FORGED}', 'forgedold')`);
  await H.owner(`update public.profiles set created_at = '2020-01-01' where id = '${FORGED}'`);
}
try { await neu.apply("0016_abuse_hardening.sql"); await neu.apply("0016_abuse_hardening.sql"); } catch (e) { console.log("FAILED to apply 0016", e.message); process.exit(1); }
check("0016 applies twice (idempotent)", true);
let r = await neu.owner(`select created_at > now() - interval '2 hours' as fixed from public.profiles where id = '${FORGED}'`);
check("M1: a profile forged older than its auth account is repaired by the migration", r.rows[0].fixed === true, r);
r = await old.owner(`select created_at < '2021-01-01' as still from public.profiles where id = '${FORGED}'`);
check("M1 baseline: on 0015 it stays forged", r.rows[0].still === true, r);

const VICTIM = await Promise.all([old, neu].map(async (H) => H.acct(60, { name: "victim" }))); // same name on both: ids differ, that is fine

// ═════════ M1: a fresh account back-dates its profile and files a copyright claim ═════════
for (const [i, H] of [old, neu].entries()) {
  const fixed = H === neu, tag = `M1 [${H.label}]`;
  const V = VICTIM[i];
  const item = await H.item(V);
  const F = nid();
  await H.owner(`insert into auth.users(id, created_at) values ('${F}', now())`);
  r = await H.user(F, `insert into public.profiles(id, public_name, created_at, terms_accepted_at) values ('${F}', 'fresh${F.slice(0, 4)}', '2020-01-01', '2020-01-01') returning created_at, terms_accepted_at`);
  check(`${tag}: a person can insert their own profile`, r.ok, r);
  const when = new Date(r.rows[0].created_at).getTime(), tsa = new Date(r.rows[0].terms_accepted_at).getTime();
  check(`${tag}: created_at ${fixed ? "is forced to now()" : "keeps the forged 2020 date (the bug)"}`, fixed ? Date.now() - when < 60000 : when < Date.parse("2021-01-01"), r);
  check(`${tag}: terms_accepted_at ${fixed ? "is stamped now() (not back-dated)" : "keeps the forged date (the bug)"}`, fixed ? Date.now() - tsa < 60000 : tsa < Date.parse("2021-01-01"), r);
  await H.user(F, `select public.accept_terms('1.0')`);
  r = await H.user(F, `select public.file_copyright_claim('${item}', 'This is my work, see the original files', 'Fake Rights Holder')`);
  if (fixed) check(`${tag}: the claim is refused (profile_too_new)`, !r.ok && r.hint === "profile_too_new", r);
  else check(`${tag}: ATTACK WORKS on 0015: the fresh account hid someone's item`, r.ok && r.rows[0].file_copyright_claim.hidden === true, r);
  // legacy view with a forged creado_en
  const G = nid();
  await H.owner(`insert into auth.users(id, created_at) values ('${G}', now())`);
  r = await H.user(G, `insert into public.perfiles(id, nombre_publico, acepto_terminos_en, creado_en) values ('${G}', 'old${G.slice(0, 4)}', now(), '2019-05-05') returning creado_en, acepto_terminos_en`);
  check(`${tag}: old client creates its profile through perfiles (works on both)`, r.ok && r.rows[0].acepto_terminos_en !== null, r);
  check(`${tag}: creado_en ${fixed ? "is forced to now()" : "stays forged"}`, fixed ? Date.now() - new Date(r.rows[0].creado_en).getTime() < 60000 : new Date(r.rows[0].creado_en) < Date.parse("2020-01-01"), r);
  if (fixed) {
    // terms_accepted_at: first acceptance by an old client works (stamped), later changes are ignored
    const Hh = nid();
    await H.owner(`insert into auth.users(id) values ('${Hh}')`);
    await H.user(Hh, `insert into public.perfiles(id, nombre_publico) values ('${Hh}', 'later${Hh.slice(0, 4)}')`);
    r = await H.user(Hh, `update public.perfiles set acepto_terminos_en = '2001-01-01' where id = '${Hh}' returning acepto_terminos_en`);
    check(`${tag}: old client: first acceptance through UPDATE works and is stamped now()`, r.ok && Date.now() - new Date(r.rows[0].acepto_terminos_en).getTime() < 60000, r);
    const first = r.rows[0].acepto_terminos_en;
    r = await H.user(Hh, `update public.perfiles set acepto_terminos_en = '2001-01-01' where id = '${Hh}' returning acepto_terminos_en`);
    check(`${tag}: once accepted it can not be changed by the person`, r.ok && new Date(r.rows[0].acepto_terminos_en).getTime() === new Date(first).getTime(), r);
    r = await H.user(Hh, `update public.perfiles set acepto_terminos_en = null where id = '${Hh}' returning acepto_terminos_en`);
    check(`${tag}: ...nor cleared`, r.ok && r.rows[0].acepto_terminos_en !== null, r);
    r = await H.user(Hh, `update public.profiles set created_at = '2001-01-01' where id = '${Hh}'`);
    check(`${tag}: created_at is not updatable (no column grant)`, !r.ok, r);
    r = await H.owner(`update public.profiles set created_at = '2001-01-01' where id = '${Hh}' returning created_at`);
    check(`${tag}: the owner / service role can still repair a row`, r.ok && new Date(r.rows[0].created_at) < Date.parse("2002-01-01"), r);
  }
  // legacy-timestamp-only account (old client) with a 3-day-old profile: can not file a claim once 0016 is in
  const L = await H.acct(3, { terms: false, legacyTerms: true });
  const item2 = await H.item(V);
  r = await H.user(L, `select public.file_copyright_claim('${item2}', 'This is my work, see the original files', 'Someone')`);
  if (fixed) check(`${tag}: a legacy timestamp alone does not allow a claim (terms_required)`, !r.ok && r.hint === "terms_required", r);
  else check(`${tag}: baseline: the legacy timestamp passed`, r.ok, r);
  // a real, old account with accepted terms still files a claim
  const R = await H.acct(10);
  const item3 = await H.item(V);
  r = await H.user(R, `select public.file_copyright_claim('${item3}', 'This is my work, see the original files', 'Real Holder')`);
  check(`${tag}: a genuine old account with accepted terms still files a claim`, r.ok, r);
}

// ═════════ M2: sybil reports ═════════
for (const [i, H] of [old, neu].entries()) {
  const fixed = H === neu, tag = `M2 [${H.label}]`, V = VICTIM[i];
  // three brand-new accounts report an approved item (one of them forged its age first)
  const X = await H.item(V);
  const sy = [];
  for (let k = 0; k < 3; k++) {
    const s = nid(); sy.push(s);
    await H.owner(`insert into auth.users(id, created_at) values ('${s}', now())`);
    await H.user(s, `insert into public.profiles(id, public_name, created_at) values ('${s}', 'syb${s.slice(0, 5)}', '2019-01-01')`);
  }
  const rr = [];
  rr.push(await H.user(sy[0], `insert into public.reports(item, reporter, reason) values ('${X}', '${sy[0]}', 'malware')`));
  rr.push(await H.user(sy[1], `insert into public.reports(item, reporter, reason) values ('${X}', '${sy[1]}', 'copyright')`));
  rr.push(await H.user(sy[2], `insert into public.reports(item, reporter, reason) values ('${X}', '${sy[2]}', 'other')`));
  const st = await H.status(X), q = await H.queued(X);
  if (fixed) {
    check(`${tag}: brand-new accounts can not report at all`, rr.every((x) => !x.ok), rr);
    check(`${tag}: the item stays approved and its files stay public`, st.s === "approved" && q.length === 0, { st, q });
  } else {
    check(`${tag}: ATTACK WORKS on 0015: 3 fresh accounts hid the item`, rr.every((x) => x.ok) && st.s === "pending", { rr, st });
    check(`${tag}: ATTACK WORKS on 0015: 2 malware/copyright reports queued the public files`, q.length === 3 && q.every((x) => x.action === "to_quarantine"), q);
  }
  // reporter's report timestamp: the legacy view let the client choose it
  const T = await H.acct(10, { terms: false, legacyTerms: true });
  const Y = await H.item(V);
  r = await H.user(T, `insert into public.reportes(item, reportante, motivo, creado_en) values ('${Y}', '${T}', 'otro', '2000-01-01')`);
  check(`${tag}: old client can report through the legacy view (aged account, no terms flag)`, r.ok, r);
  r = await H.owner(`select created_at < '2001-01-01' as forged from public.reports where item = '${Y}'`);
  check(`${tag}: report created_at ${fixed ? "is stamped by the server" : "stays forged (bug)"}`, r.rows[0].forged === !fixed, r);
  if (fixed) {
    // weights: three 30-day accounts hide it (score 3) but queue NOTHING, even with malware; 3-day accounts count half
    const W = await H.item(V);
    const olds = [await H.acct(30), await H.acct(30), await H.acct(30)];
    for (const [k, o] of olds.entries()) {
      r = await H.user(o, `insert into public.reports(item, reporter, reason) values ('${W}', '${o}', 'malware')`);
      check(`${tag}: aged account may report (${k})`, r.ok, r);
      if (k === 1) check(`${tag}: 2 reports do not hide`, (await H.status(W)).s === "approved");
    }
    check(`${tag}: 3 aged reports hide the item (pending human review)`, (await H.status(W)).s === "pending");
    check(`${tag}: ...but never queue the public files (malware / copyright included)`, (await H.queued(W)).length === 0, await H.queued(W));
    const W2 = await H.item(V);
    const mids = []; for (let k = 0; k < 6; k++) mids.push(await H.acct(3));
    for (const [k, o] of mids.entries()) {
      await H.user(o, `insert into public.reports(item, reporter, reason) values ('${W2}', '${o}', 'other')`);
      if (k === 4) check(`${tag}: 5 three-day-old accounts (score 2.5) do not hide`, (await H.status(W2)).s === "approved");
    }
    check(`${tag}: 6 three-day-old accounts (score 3) hide`, (await H.status(W2)).s === "pending");
    // the item's own author reporting it counts nothing; a suspended reporter counts nothing
    const W3 = await H.item(V);
    const a30 = [await H.acct(30), await H.acct(30)];
    for (const o of a30) await H.user(o, `insert into public.reports(item, reporter, reason) values ('${W3}', '${o}', 'other')`);
    await H.owner(`insert into public.reports(item, reporter, reason) values ('${W3}', '${V}', 'other')`);
    check(`${tag}: a report by the author themself counts nothing`, (await H.status(W3)).s === "approved");
    const susp = await H.acct(30); await H.admin(`select public.set_suspension('${susp}', true)`);
    await H.owner(`insert into public.reports(item, reporter, reason) values ('${W3}', '${susp}', 'other')`);
    check(`${tag}: a report by a suspended account counts nothing`, (await H.status(W3)).s === "approved");
    // daily cap
    const cap = await H.acct(30);
    let lastOk = 0, lastErr;
    for (let k = 0; k < 11; k++) { const it = await H.item(V, { files: false }); const x = await H.user(cap, `insert into public.reports(item, reporter, reason) values ('${it}', '${cap}', 'spam')`); if (x.ok) lastOk++; else lastErr = x; }
    check(`${tag}: at most 10 reports per account per day (report_rate_limited)`, lastOk === 10 && lastErr?.hint === "report_rate_limited", { lastOk, lastErr });
    // terms flag: NULL = not required; set = terms_version required
    const nt = await H.acct(30, { terms: false, legacyTerms: true });
    const Z = await H.item(V, { files: false });
    await H.owner(`update public.app_settings set terms_enforced_from = now() - interval '1 second'`);
    r = await H.user(nt, `insert into public.reports(item, reporter, reason) values ('${Z}', '${nt}', 'other')`);
    check(`${tag}: terms enforced: a legacy-timestamp account can not report`, !r.ok, r);
    await H.owner(`update public.profiles set terms_version = '1.0' where id = '${nt}'`);
    r = await H.user(nt, `insert into public.reports(item, reporter, reason) values ('${Z}', '${nt}', 'other')`);
    check(`${tag}: ...after accepting it can`, r.ok, r);
    await H.owner(`update public.app_settings set terms_enforced_from = null`);
    const nt2 = await H.acct(30, { terms: false });
    r = await H.user(nt2, `insert into public.reports(item, reporter, reason) values ('${Z}', '${nt2}', 'other')`);
    check(`${tag}: terms flag NULL: an account without terms still reports (old clients)`, r.ok, r);
  }
}

// ═════════ B3: banned accounts can neither claim nor report ═════════
for (const [i, H] of [old, neu].entries()) {
  const fixed = H === neu, tag = `B3 [${H.label}]`, V = VICTIM[i];
  const banned = await H.acct(30);
  await H.owner(`update public.profiles set upload_banned_until = now() + interval '7 days' where id = '${banned}'`);
  const it1 = await H.item(V), it2 = await H.item(V);
  const rc = await H.user(banned, `select public.file_copyright_claim('${it1}', 'This is my work, see the original files', 'Banned Person')`);
  const rp = await H.user(banned, `insert into public.reports(item, reporter, reason) values ('${it2}', '${banned}', 'other')`);
  if (fixed) {
    check(`${tag}: a banned account can not file a claim (upload_banned)`, !rc.ok && rc.hint === "upload_banned", rc);
    check(`${tag}: ...nor a report`, !rp.ok, rp);
    await H.owner(`update public.profiles set upload_banned_until = now() - interval '1 second' where id = '${banned}'`);
    const rp2 = await H.user(banned, `insert into public.reports(item, reporter, reason) values ('${it2}', '${banned}', 'other')`);
    check(`${tag}: an expired ban lets them report again`, rp2.ok, rp2);
  } else check(`${tag}: ATTACK WORKS on 0015: a banned account files claims and reports`, rc.ok && rp.ok, { rc, rp });
}

// ═════════ M3: suspension takes files down; lifting hands them to review ═════════
for (const [i, H] of [old, neu].entries()) {
  const fixed = H === neu, tag = `M3 [${H.label}]`;
  const S = await H.acct(30);
  const its = [await H.item(S), await H.item(S)];
  await H.admin(`select public.set_suspension('${S}', true)`);
  await H.admin(`select public.set_suspension('${S}', true)`); // idempotent
  const sts = await Promise.all(its.map(H.status)), qs = await Promise.all(its.map(H.queued));
  if (fixed) {
    check(`${tag}: suspended: items go pending (hidden: author suspended)`, sts.every((x) => x.s === "pending" && x.r === "Hidden: author suspended"), sts);
    check(`${tag}: ...and every public file is queued to_quarantine (3 per item, no duplicates)`, qs.every((q) => q.length === 3 && q.every((x) => x.action === "to_quarantine")), qs);
    r = await H.admin(`select count(*)::int n from public.moderation_log where item = '${its[0]}' and detail like 'author suspended%'`);
    check(`${tag}: logged`, r.rows[0].n >= 1, r);
    await H.admin(`select public.set_suspension('${S}', false)`);
    const after = await Promise.all(its.map(H.status));
    check(`${tag}: unsuspended: items wait for re-approval with a clear reason (not public)`, after.every((x) => x.s === "pending" && x.r === "Pending re-review: suspension lifted"), after);
    r = await H.user(S, `select public.accept_terms('1.0')`);
    // the pending cap ignores them
    const many = await H.acct(30);
    for (let k = 0; k < 10; k++) await H.item(many);
    await H.admin(`select public.set_suspension('${many}', true)`); await H.admin(`select public.set_suspension('${many}', false)`);
    const nid2 = nid();
    r = await H.user(many, `insert into public.community_items(id, author, kind, title, quarantine_path) values ('${nid2}', '${many}', 'template', 'New one', '${many}/${nid2}/x.json')`);
    check(`${tag}: items parked by a suspension do not count in the 10-pending cap`, r.ok, r);
    r = await H.admin(`select public.set_suspension('${nid()}', true)`);
    check(`${tag}: an unknown user id is a no-op`, r.ok, r);
  } else {
    check(`${tag}: ATTACK WORKS on 0015: suspended creator's items stay approved with public files and nothing is queued`, sts.every((x) => x.s === "approved") && qs.every((q) => q.length === 0), { sts, qs });
  }
}

// ═════════ P13: EXECUTE for anon ═════════
for (const H of [old, neu]) {
  const fixed = H === neu;
  const priv = async (fn) => (await H.owner(`select has_function_privilege('anon', '${fn}', 'execute') as p`)).rows[0].p;
  for (const fn of ["legacy.raise_code(text,text,text)", "legacy.license_ok(text)", "legacy.source_url_ok(text)"]) {
    check(`P13 [${H.label}]: anon ${fixed ? "can no longer execute" : "can execute (baseline)"} ${fn}`, (await priv(fn)) === !fixed);
  }
  check(`P13 [${H.label}]: anon keeps legacy.is_suspended (the restrictive policies need it)`, await priv("legacy.is_suspended(uuid)"));
  r = await H.as(null, "anon", `select count(*)::int n from public.community_items`);
  check(`P13 [${H.label}]: anon still browses the catalog`, r.ok, r);
  r = await H.as(null, "anon", `select count(*)::int n from public.public_profiles`);
  check(`P13 [${H.label}]: anon still reads public profiles`, r.ok, r);
  r = await H.user(B, `insert into public.community_items(id, author, kind, title, license, source_url) values ('${nid()}', '${B}', 'template', 'x', 'bogus', null)`);
  check(`P13 [${H.label}]: authenticated still gets license_invalid from the CHECK / trigger`, !r.ok && /license_invalid|check/i.test(r.err), r);
  if (fixed) check(`P13 [${H.label}]: new helpers are not executable by anon / authenticated`, (await H.owner(`select not has_function_privilege('anon','legacy.account_since(uuid)','execute') and not has_function_privilege('authenticated','legacy.account_since(uuid)','execute') and not has_function_privilege('authenticated','legacy.report_weight(uuid)','execute') and not has_function_privilege('authenticated','legacy.identity_hashes(uuid)','execute') and not has_function_privilege('authenticated','legacy.snapshot_tombstone(uuid)','execute') as ok`)).rows[0].ok);
}

// ═════════ P6: ban evasion by deleting the account ═════════
for (const H of [old, neu]) {
  const fixed = H === neu, tag = `P6 [${H.label}]`;
  const sub = "gh-" + nid();
  const mkId = async (uid, s) => H.owner(`insert into auth.identities(user_id, provider, identity_data) values ('${uid}', 'github', '{"sub":"${s}"}')`);
  const T1 = await H.acct(30); await mkId(T1, sub);
  await H.owner(`update public.profiles set strikes = 3, upload_banned_until = '9999-12-31 00:00:00+00' where id = '${T1}'`);
  const clean = await H.acct(30); await mkId(clean, "gh-" + nid());
  r = await H.owner(`delete from auth.users where id = '${T1}'`);
  check(`${tag}: deleting a banned account works`, r.ok, r);
  r = await H.owner(`delete from auth.users where id = '${clean}'`);
  check(`${tag}: deleting a clean account works`, r.ok, r);
  // the same person signs in again with the same GitHub account: new auth id, new profile
  const T2 = nid();
  await H.owner(`insert into auth.users(id, created_at) values ('${T2}', now())`); await mkId(T2, sub);
  r = await H.user(T2, `insert into public.profiles(id, public_name) values ('${T2}', 'again${T2.slice(0, 4)}') returning strikes, upload_banned_until, suspended`);
  if (fixed) {
    check(`${tag}: the new profile inherits strikes and the permanent ban`, r.ok && r.rows[0].strikes === 3 && new Date(r.rows[0].upload_banned_until) >= Date.parse("9999-01-01"), r);
    const iid = nid();
    r = await H.user(T2, `insert into public.community_items(id, author, kind, title, quarantine_path) values ('${iid}', '${T2}', 'template', 'Evasion', '${T2}/${iid}/x.json')`);
    check(`${tag}: ...so it can not upload (upload_banned)`, !r.ok && r.hint === "upload_banned", r);
    r = await H.owner(`select count(*)::int n, bool_and(key_hash ~ '^[0-9a-f]{64}$' and key_hash not like '%github%') as hashed from public.profile_tombstones`);
    check(`${tag}: only the flagged account left a tombstone, and it holds a hash (no identity)`, r.rows[0].n === 1 && r.rows[0].hashed === true, r);
    r = await H.user(T2, `select * from public.profile_tombstones`); check(`${tag}: people can not read tombstones`, !r.ok, r);
    r = await H.user(T2, `select * from public.tombstone_secret`); check(`${tag}: ...nor the secret`, !r.ok, r);
    r = await H.as(null, "anon", `select * from public.profile_tombstones`); check(`${tag}: ...nor anon`, !r.ok, r);
    // a different GitHub account does not inherit
    const T3 = nid();
    await H.owner(`insert into auth.users(id, created_at) values ('${T3}', now())`); await mkId(T3, "gh-other-" + nid());
    r = await H.user(T3, `insert into public.profiles(id, public_name) values ('${T3}', 'other${T3.slice(0, 4)}') returning strikes, upload_banned_until`);
    check(`${tag}: a different identity starts clean`, r.ok && r.rows[0].strikes === 0 && r.rows[0].upload_banned_until === null, r);
    // a client can not dodge it by sending clean values, and the profile trigger alone also covers a direct profile delete
    const T4 = await H.acct(30); const sub4 = "gh-" + nid(); await mkId(T4, sub4);
    await H.owner(`update public.profiles set strikes = 1, suspended = true where id = '${T4}'`);
    r = await H.owner(`delete from public.profiles where id = '${T4}'`); check(`${tag}: deleting only the profile works and snapshots (suspension)`, r.ok, r);
    r = await H.user(T4, `insert into public.profiles(id, public_name, strikes, suspended) values ('${T4}', 'back${T4.slice(0, 4)}', 0, false) returning strikes, suspended`);
    check(`${tag}: recreated profile is suspended again with its strike, whatever the client sends`, r.ok && r.rows[0].strikes === 1 && r.rows[0].suspended === true, r);
    // expired tombstones are ignored
    await H.owner(`update public.profile_tombstones set expires_at = now() - interval '1 day'`);
    const T5 = nid();
    await H.owner(`insert into auth.users(id, created_at) values ('${T5}', now())`); await mkId(T5, sub);
    r = await H.user(T5, `insert into public.profiles(id, public_name) values ('${T5}', 'late${T5.slice(0, 4)}') returning strikes`);
    check(`${tag}: an expired tombstone is ignored`, r.ok && r.rows[0].strikes === 0, r);
    // a deletion is never blocked, even if the secret is gone
    const T6 = await H.acct(30); await mkId(T6, "gh-" + nid());
    await H.owner(`update public.profiles set strikes = 2 where id = '${T6}'`);
    await H.owner(`delete from public.tombstone_secret`);
    r = await H.owner(`delete from auth.users where id = '${T6}'`); check(`${tag}: a missing secret never blocks an account deletion`, r.ok, r);
    r = await H.owner(`select tgname from pg_trigger where tgrelid = 'auth.users'::regclass and tgname = 'lienzo_tombstone_on_user_delete'`);
    check(`${tag}: the trigger on auth.users exists`, r.rows.length === 1, r);
  } else {
    check(`${tag}: ATTACK WORKS on 0015: re-registering with the same GitHub account gives a clean profile`, r.ok && r.rows[0].strikes === 0 && r.rows[0].upload_banned_until === null, r);
  }
}
// HMAC against the RFC 4231 test case 2 (and a key longer than the block size: test case 6)
r = await neu.owner(`select encode(legacy.hmac_sha256('Jefe'::bytea, 'what do ya want for nothing?'::bytea), 'hex') as h,
  encode(legacy.hmac_sha256(decode(repeat('aa', 131), 'hex'), 'Test Using Larger Than Block-Size Key - Hash Key First'::bytea), 'hex') as h2`);
check("P6: legacy.hmac_sha256 matches RFC 4231 case 2", r.rows[0].h === "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843", r);
check("P6: ...and case 6 (long key)", r.rows[0].h2 === "60e431591ee0b67f0d8a26aacbf5b77f8e0bc6213728c5140546040f0ee37f54", r);

// ═════════ P10: export without the claimant's free text ═════════
for (const [i, H] of [old, neu].entries()) {
  const fixed = H === neu, tag = `P10 [${H.label}]`;
  const author = await H.acct(30), holder = await H.acct(30);
  const it = await H.item(author);
  await H.user(holder, `select public.file_copyright_claim('${it}', 'Call me on +34 600 123 456 or mail me, this is mine', 'Holder', 'holder@example.org')`);
  r = await H.user(author, `select public.export_my_data() as d`);
  const against = r.rows[0].d.copyright_claims_against_my_items;
  check(`${tag}: claims against my items are exported`, against.length === 1 && against[0].status === "open", against);
  if (fixed) check(`${tag}: ...without the claimant's description, name or contact`, against[0].description === undefined && against[0].claimant === undefined && against[0].claimant_contact === undefined && !JSON.stringify(r.rows[0].d).includes("+34 600"), against[0]);
  else check(`${tag}: baseline: 0015 leaked the description`, against[0].description?.includes("+34 600"), against[0]);
  r = await H.user(holder, `select public.export_my_data() as d`);
  check(`${tag}: the claimant still gets their OWN claim text`, r.rows[0].d.copyright_claims_filed[0]?.description?.includes("+34 600") === true, r.rows[0].d.copyright_claims_filed);
}

// ═════════ Legacy paths still work on 0016 (old clients write through the Spanish views) ═════════
{
  const H = neu, U = await H.acct(30, { terms: false, legacyTerms: true });
  const itemId = nid();
  r = await H.user(U, `insert into public.items(id, autor, tipo, titulo, ruta_cuarentena) values ('${itemId}', '${U}', 'plantilla', 'Legacy upload', '${U}/${itemId}/x.json')`);
  check("legacy: an old client uploads through items", r.ok, r);
  r = await H.user(U, `select nombre_publico, acepto_terminos_en, creado_en from public.perfiles where id = '${U}'`);
  check("legacy: perfiles still readable (6-column view unchanged)", r.ok && r.rows.length === 1, r);
  r = await H.user(U, `update public.perfiles set declara_mayor_18_en = now(), mostrar_18 = true where id = '${U}' returning mostrar_18`);
  check("legacy: perfiles still updatable", r.ok && r.rows[0].mostrar_18 === true, r);
  r = await H.user(U, `select public.record_download('${itemId}')`);
  check("legacy: normal RPCs unaffected", r.ok, r);
}

// ═════════ Rollback and re-apply (twice) ═════════
for (let round = 1; round <= 2; round++) {
  try { await neu.apply("0016_abuse_hardening.down.sql", rb); await neu.apply("0016_abuse_hardening.down.sql", rb); } catch (e) { console.log("FAILED rollback (round " + round + ")", e.message); process.exit(1); }
  r = await neu.owner(`select to_regclass('public.profile_tombstones') is null as gone, to_regprocedure('legacy.can_report()') is null as g2,
    (select count(*) = 0 from pg_trigger where tgname in ('lienzo_tombstone_on_user_delete', 'profiles_tombstone_restore', 'reports_limit')) as g3`);
  check(`rollback ${round}: new objects gone`, r.rows[0].gone && r.rows[0].g2 && r.rows[0].g3, r);
  const fresh = nid(); await neu.owner(`insert into auth.users(id, created_at) values ('${fresh}', now())`);
  r = await neu.user(fresh, `insert into public.profiles(id, public_name, created_at) values ('${fresh}', 'rb${fresh.slice(0, 4)}', '2020-01-01') returning created_at`);
  check(`rollback ${round}: back to 0015 behaviour (created_at client-writable again)`, r.ok && new Date(r.rows[0].created_at) < Date.parse("2021-01-01"), r);
  const victim = await neu.acct(60); const it = await neu.item(victim);
  r = await neu.user(fresh, `insert into public.reports(item, reporter, reason) values ('${it}', '${fresh}', 'other')`);
  check(`rollback ${round}: reports_create is the 0001 rule again`, r.ok, r);
  r = await neu.user(B, `select public.export_my_data() as d`); check(`rollback ${round}: export_my_data works`, r.ok, r);
  try { await neu.apply("0016_abuse_hardening.sql"); await neu.apply("0016_abuse_hardening.sql"); } catch (e) { console.log("FAILED reapply (round " + round + ")", e.message); process.exit(1); }
  r = await neu.user(fresh, `insert into public.reports(item, reporter, reason) values ('${it}', '${fresh}', 'spam')`);
  check(`reapply ${round}: a brand-new profile can not report again`, !r.ok, r);
}

console.log(`${passes} passed, ${fails} failed`); process.exit(fails ? 1 : 0);
