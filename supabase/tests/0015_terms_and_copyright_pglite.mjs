// SQL test for migration 0015 (terms, declarations, copyright claims, strikes, suspension, export) in PGlite.
//   PGLITE=file:///<path to pglite dist/index.js> node supabase/tests/0015_terms_and_copyright_pglite.mjs
// Replays 0001-0014, then 0015 (twice), checks the new behaviour AND every legacy client path (old clients <= 1.5 write through the Spanish views),
// then the rollback and a re-apply. The whole 0012 harness is also run with 0015 applied: `LATER=0013,0014,0015 node supabase/tests/0012_pglite.mjs`.
import fs from "node:fs"; import path from "node:path"; import { fileURLToPath } from "node:url";
const here = path.dirname(fileURLToPath(import.meta.url));
const { PGlite } = await import(process.env.PGLITE ?? "@electric-sql/pglite");
const src = fs.readFileSync(path.join(here, "0012_pglite.mjs"), "utf8");
const boot = src.match(/const BOOT = `([\s\S]*?)`;/)[1];
const A = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa", B = "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb", C = "cccccccc-cccc-4ccc-8ccc-cccccccccccc";
const D = "dddddddd-dddd-4ddd-8ddd-dddddddddddd", E = "eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee";
const db = new PGlite(); let fails = 0, passes = 0;
const check = (n, c, x) => { if (c) passes++; else { fails++; console.log("FAIL", n, x === undefined ? "" : JSON.stringify(x)); } };
await db.exec(boot.replaceAll("${A}", A).replaceAll("${B}", B).replaceAll("${C}", C)
  + `insert into auth.users values ('${D}'), ('${E}');`);
const mig = path.join(here, "..", "migrations"), rb = path.join(here, "..", "rollbacks");
const read = (dir, f) => fs.readFileSync(path.join(dir, f), "utf8").replace(/create extension if not exists pgcrypto;/, "");
for (const f of fs.readdirSync(mig).filter(f => /^00(0[1-9]|1[0-4])_/.test(f)).sort()) await db.exec(read(mig, f));
const as = async (uid, role, sql, params) => {
  await db.exec(`reset role; set role ${role}; select set_config('request.jwt.claim.sub', '${uid ?? ""}', false), set_config('request.jwt.claim.role', '${role}', false);`);
  try { const r = await db.query(sql, params); return { ok: true, rows: r.rows }; } catch (e) { return { ok: false, err: String(e.message), hint: e.hint, code: e.code }; }
  finally { await db.exec("reset role; select set_config('request.jwt.claim.role', '', false), set_config('request.jwt.claim.sub', '', false);"); }
};
const user = (uid, sql, p) => as(uid, "authenticated", sql, p);
const admin = (sql, p) => as(null, "service_role", sql, p);
const owner = async (sql, p) => { try { const r = await db.query(sql, p); return { ok: true, rows: r.rows }; } catch (e) { return { ok: false, err: String(e.message), hint: e.hint }; } };
const nid = () => crypto.randomUUID();

// ───── Legacy data BEFORE 0015: an old client created profiles (A with the legacy terms timestamp, B without), items, a report ─────
await db.exec(`insert into public.profiles (id, public_name, terms_accepted_at) values ('${A}', 'alice', now()), ('${B}', 'bobby', null), ('${C}', 'carol', now());
  update public.profiles set name_reviewed = public_name;`);
const IA = nid(), IB = nid();
await owner(`insert into public.community_items(id, author, kind, title) values ('${IA}', '${A}', 'template', 'Alice template'), ('${IB}', '${B}', 'template', 'Bobby template')`);
await admin(`update public.community_items set status = 'approved', public_path = 'template/${IA}/f.json', reviewed_at = now() where id = '${IA}'`);
await admin(`update public.community_items set status = 'approved', public_path = 'template/${IB}/f.json', reviewed_at = now() where id = '${IB}'`);
await user(C, `insert into public.reportes(item, reportante, motivo) values ('${IA}', '${C}', 'derechos_de_autor')`);

const up = (f = "0015_terms_and_copyright.sql") => db.exec(read(mig, f));
try { await up(); await up(); } catch (e) { console.log("FAILED to apply 0015", e.message); process.exit(1); }
check("0015 applies twice (idempotent)", true);

// ═════════ 1. Terms versions, settings, current_terms ═════════
let r = await user(B, `select public.current_terms() as t`);
check("current_terms is readable by authenticated and returns version 1.0", r.ok && r.rows[0].t.version === "1.0", r);
r = await as(null, "anon", `select public.current_terms() as t`);
check("current_terms is readable by anon", r.ok && r.rows[0].t.version === "1.0", r);
for (const t of ["terms_versions", "terms_log", "app_settings", "copyright_claims"]) {
  r = await user(A, `select * from public.${t}`); check(`people cannot read ${t}`, !r.ok, r);
  r = await as(null, "anon", `select * from public.${t}`); check(`anon cannot read ${t}`, !r.ok, r);
  r = await user(A, `delete from public.${t}`); check(`people cannot delete from ${t}`, !r.ok, r);
}
r = await owner(`select terms_enforced_from, license_required_from from public.app_settings`);
check("default: nothing enforced", r.rows.length === 1 && r.rows[0].terms_enforced_from === null && r.rows[0].license_required_from === null, r);
r = await owner(`insert into public.app_settings(singleton) values (false)`); check("app_settings is a single row", !r.ok, r);

// ═════════ 2. Profiles: legacy paths and the guarded columns ═════════
r = await user(D, `insert into public.profiles(id, public_name, terms_version, strikes, upload_banned_until, suspended) values ('${D}', 'dora', '1.0', 2, now() + interval '1 day', true) returning terms_version, strikes, upload_banned_until, suspended`);
check("a new profile cannot write terms_version / strikes / ban / suspended (forced to defaults)", r.ok && r.rows[0].terms_version === null && r.rows[0].strikes === 0 && r.rows[0].upload_banned_until === null && r.rows[0].suspended === false, r);
r = await user(E, `insert into public.perfiles(id, nombre_publico, acepto_terminos_en) values ('${E}', 'erin', now()) returning acepto_terminos_en`);
check("old client: creating a profile with the legacy terms timestamp still works", r.ok && r.rows[0].acepto_terminos_en !== null, r);
r = await user(B, `update public.perfiles set acepto_terminos_en = now() where id = '${B}' returning acepto_terminos_en`);
check("old client: updating acepto_terminos_en still works", r.ok && r.rows[0].acepto_terminos_en !== null, r);
await owner(`update public.profiles set terms_accepted_at = null where id = '${B}'`);
for (const col of ["terms_version = '1.0'", "strikes = 0", "upload_banned_until = null", "suspended = false"]) {
  r = await user(B, `update public.profiles set ${col} where id = '${B}'`); check(`people cannot update (${col.split(" ")[0]})`, !r.ok && /permission denied/.test(r.err), r);
}
r = await user(A, `select terms_version, strikes, suspended from public.profiles where id = '${A}'`);
check("a person reads their own moderation fields", r.ok && r.rows[0].strikes === 0, r);
r = await user(A, `select strikes from public.profiles where id = '${B}'`);
check("but not another person's", r.ok && r.rows.length === 0, r);
r = await user(A, `select * from public.perfiles where id = '${A}'`);
check("legacy view perfiles unchanged (6 columns)", r.ok && Object.keys(r.rows[0]).join() === "id,nombre_publico,mostrar_18,acepto_terminos_en,creado_en,declara_mayor_18_en", r);

// ═════════ 3. accept_terms ═════════
r = await as(null, "anon", `select public.accept_terms('1.0')`); check("anon cannot call accept_terms", !r.ok, r);
r = await user(B, `select public.accept_terms('9.9')`); check("unknown version refused (hint terms_unknown_version)", !r.ok && r.hint === "terms_unknown_version" && /^terms_unknown_version:/.test(r.err), r);
r = await user(B, `select public.accept_terms('v0')`); check("v0 is not an acceptable version", !r.ok && r.hint === "terms_unknown_version", r);
r = await user(B, `select public.accept_terms(null)`); check("null version refused", !r.ok, r);
r = await user(nid(), `select public.accept_terms('1.0')`); check("a user without a profile is refused (profile_required)", !r.ok, r);
r = await user(B, `select public.accept_terms('1.0') as t`); check("accept_terms ok", r.ok && r.rows[0].t.version === "1.0" && r.rows[0].t.current === "1.0", r);
r = await user(B, `select terms_version, terms_accepted_at from public.perfiles p, public.profiles q where q.id = '${B}' and p.id = q.id`);
r = await user(B, `select terms_version, terms_accepted_at is not null as ts from public.profiles where id = '${B}'`);
check("profile now has the version and the legacy timestamp too", r.rows[0].terms_version === "1.0" && r.rows[0].ts === true, r);
r = await user(B, `select public.accept_terms('1.0')`); check("accepting twice is fine", r.ok, r);
r = await owner(`select count(*)::int n from public.terms_log where user_id = '${B}'`); check("and logs only once (audit)", r.rows[0].n === 1, r);
r = await owner(`update public.terms_log set version = '1.0'`); check("terms_log: UPDATE refused even for the owner", !r.ok && /append-only/.test(r.err), r);
r = await owner(`delete from public.terms_log`); check("terms_log: direct DELETE refused even for the owner", !r.ok && /append-only/.test(r.err), r);
r = await owner(`insert into public.terms_versions(version, summary) values ('v0', 'x')`); check("terms_versions refuses the reserved id v0", !r.ok, r);
await owner(`insert into public.terms_versions(version, published_at) values ('2.0', now() + interval '1 second')`);
await owner(`update public.terms_versions set published_at = now() + interval '1 second' where version = '2.0'`);
r = await user(B, `select public.current_terms() as t`); check("a newer version becomes the current one", r.rows[0].t.version === "2.0", r);
r = await user(B, `select public.accept_terms('1.0')`); check("accepting an outdated version is refused (terms_outdated)", !r.ok && r.hint === "terms_outdated", r);
r = await user(B, `select public.accept_terms('2.0')`); check("accepting 2.0 works", r.ok, r);
r = await owner(`select count(*)::int n from public.terms_log where user_id = '${B}'`); check("the log has both acceptances", r.rows[0].n === 2, r);
// back to 1.0 as the current version for the rest of the tests (a test fixture, done as the owner)
await owner(`update public.profiles set terms_version = null where id = '${B}'`);
await owner(`alter table public.terms_log disable trigger terms_log_append_only`);
await owner(`delete from public.terms_log where version = '2.0'`);
await owner(`alter table public.terms_log enable trigger terms_log_append_only`);
await owner(`delete from public.terms_versions where version = '2.0'`);
await owner(`update public.profiles set terms_accepted_at = null where id = '${B}'`);

// Audit: the current version is the LAST INSERTED one, not the greatest text and not the newest published_at (equal inside one transaction).
await owner(`insert into public.terms_versions(version, published_at) values ('9.0', '2030-01-01'), ('10.0', '2030-01-01')`);
r = await user(B, `select public.current_terms() as t`); check("audit 8: current version follows insertion order ('10.0' after '9.0', same published_at)", r.rows[0].t.version === "10.0", r);
r = await owner(`select legacy.current_terms_version() as v`); check("audit 8: legacy.current_terms_version agrees", r.rows[0].v === "10.0", r);
await owner(`delete from public.terms_versions where version in ('9.0', '10.0')`);
r = await user(B, `select public.current_terms() as t`); check("audit 8: back to 1.0 after removing them", r.rows[0].t.version === "1.0", r);

// ═════════ 4. Uploads: nothing enforced (default) ═════════
const PNGOK = (uid, id) => `'${uid}/${id}/design.json'`;
const insNew = (uid, id, extra = "", vals = "") => user(uid, `insert into public.community_items(id, author, kind, title, quarantine_path ${extra}) values ('${id}', '${uid}', 'template', 'Hello world', ${PNGOK(uid, id)} ${vals})`);
let id = nid();
r = await user(A, `insert into public.items(id, autor, tipo, titulo, ruta_cuarentena) values ('${id}', '${A}', 'plantilla', 'Old client upload', '${A}/${id}/f.json')`);
check("old client (legacy timestamp = v0), flags off: upload works", r.ok, r);
r = await owner(`select uploaded_under_terms, license, rights_confirmed_at from public.community_items where id = '${id}'`);
check("the effective version is only RECORDED (v0)", r.rows[0].uploaded_under_terms === "v0" && r.rows[0].license === null && r.rows[0].rights_confirmed_at === null, r);
id = nid();
r = await insNew(B, id);   // B accepted nothing
check("an account that accepted nothing can still upload while not enforced", r.ok, r);
r = await owner(`select uploaded_under_terms from public.community_items where id = '${id}'`);
check("... and null is recorded", r.rows[0].uploaded_under_terms === null, r);
id = nid();
r = await insNew(A, id, ", license, source_url, rights_confirmed_at", `, 'cc-by', 'https://github.com/me/my-theme', '2000-01-01'`);
check("new client: license + source + rights accepted", r.ok, r);
r = await owner(`select license, source_url, rights_confirmed_at > now() - interval '1 minute' as fresh from public.community_items where id = '${id}'`);
check("rights_confirmed_at is stamped by the server (a forged old date is replaced)", r.rows[0].fresh === true && r.rows[0].license === "cc-by", r);
for (const lic of ["gpl-3", "CC-BY", "cc by", "all-rights-reserved", "''", "x".repeat(40)]) {
  const i2 = nid();
  r = await insNew(A, i2, ", license, rights_confirmed_at", `, '${lic.replace(/'/g, "''")}', now()`);
  check(`license refused: ${lic.slice(0, 20)} (license_invalid)`, !r.ok && r.hint === "license_invalid" && /^license_invalid:/.test(r.err), r);
}
for (const lic of ["cc0", "public-domain", "cc-by", "cc-by-sa", "cc-by-nc", "mit", "apache-2", "ofl", "proprietary-free-use"]) {
  const i2 = nid();
  r = await insNew(A, i2, ", license, rights_confirmed_at", `, '${lic}', now()`);
  check(`license accepted: ${lic}`, r.ok, r);
  await owner(`delete from public.community_items where id = '${i2}'`);
  await owner(`delete from public.upload_log where item = '${i2}'`);
}
for (const [u, ok] of [["https://github.com/a/b", true], ["http://example.com", true], ["https://x.com/path?q=1#f", true], ["https://example.com:8080/a", true],
  ["javascript:alert(1)", false], ["ftp://example.com/x", false], ["https://user:pw@example.com", false], ["https://exa mple.com", false], ["//example.com", false],
  ["https://example.com/" + "a".repeat(290), false], ["data:text/html,<x>", false], ["https://example.com/\u0007", false]]) {
  const i2 = nid();
  r = await user(A, `insert into public.community_items(id, author, kind, title, quarantine_path, source_url) values ('${i2}', '${A}', 'template', 'Hello world', '${A}/${i2}/f.json', $1)`, [u]);
  check(`source_url ${ok ? "accepted" : "refused"}: ${u.slice(0, 40)}`, r.ok === ok && (ok || r.hint === "source_url_invalid"), r);
  if (r.ok) { await owner(`delete from public.community_items where id = '${i2}'`); await owner(`delete from public.upload_log where item = '${i2}'`); }
}
id = nid();
r = await insNew(A, id, ", license", `, 'cc0'`);
check("a licence without rights confirmation is refused (rights_required)", !r.ok && r.hint === "rights_required", r);
r = await user(A, `update public.community_items set license = 'mit' where id = '${IA}'`); check("people cannot edit license after upload", !r.ok, r);
r = await user(A, `update public.community_items set source_url = 'https://x.com' where id = '${IA}'`); check("people cannot edit source_url after upload", !r.ok, r);
r = await user(A, `update public.community_items set rights_confirmed_at = now() where id = '${IA}'`); check("people cannot edit rights_confirmed_at", !r.ok, r);
r = await admin(`update public.community_items set license = 'mit', source_url = 'https://x.com/y' where id = '${IA}' returning license`);
check("the server (service_role) can set the declarations of a legacy item", r.ok && r.rows[0].license === "mit", r);
r = await admin(`update public.community_items set license = 'nope' where id = '${IA}'`); check("the CHECK constraint backs the trigger (service_role too)", !r.ok, r);
r = await as(null, "anon", `select license, source_url from public.community_items where status = 'approved' limit 1`); check("licence and source are public columns (anon reads them)", r.ok, r);

// ═════════ 5. Enforcement flags ═════════
await owner(`update public.profiles set terms_accepted_at = now() where id = '${C}'`);   // C = legacy timestamp only
await owner(`update public.app_settings set terms_enforced_from = now() + interval '1 hour'`);
id = nid(); r = await insNew(B, id); check("a FUTURE enforcement date does not enforce yet", r.ok, r);
await owner(`update public.app_settings set terms_enforced_from = now() - interval '1 minute'`);
id = nid(); r = await insNew(B, id); check("enforced: an account that accepted nothing is refused (terms_required)", !r.ok && r.hint === "terms_required" && /^terms_required:/.test(r.err), r);
id = nid(); r = await user(C, `insert into public.items(id, autor, tipo, titulo, ruta_cuarentena) values ('${id}', '${C}', 'plantilla', 'Old client', '${C}/${id}/f.json')`);
check("enforced: an OLD client (legacy timestamp = v0) is refused through the legacy view too, with a readable message", !r.ok && /terms_required/.test(r.err), r);
await user(B, `select public.accept_terms('1.0')`);
id = nid(); r = await insNew(B, id); check("enforced: after accept_terms the same account uploads", r.ok, r);
r = await owner(`select uploaded_under_terms from public.community_items where id = '${id}'`); check("the accepted version is recorded", r.rows[0].uploaded_under_terms === "1.0", r);
r = await admin(`insert into public.community_items(id, author, kind, title) values ('${nid()}', '${C}', 'template', 'server import')`);
check("enforced: service_role (imports, functions) is never blocked", r.ok, r);
await owner(`update public.app_settings set terms_enforced_from = null`);
await owner(`update public.app_settings set license_required_from = now() - interval '1 minute'`);
id = nid(); r = await insNew(B, id); check("license_required_from: an upload without a licence is refused (license_invalid)", !r.ok && r.hint === "license_invalid", r);
id = nid(); r = await insNew(B, id, ", license, rights_confirmed_at", `, 'ofl', now()`); check("license_required_from: with licence + rights it works", r.ok, r);
await owner(`update public.app_settings set license_required_from = null`);

// ═════════ 6. Bans and suspension ═════════
await owner(`update public.profiles set upload_banned_until = now() + interval '2 days' where id = '${B}'`);
id = nid(); r = await insNew(B, id); check("banned: new upload refused (upload_banned)", !r.ok && r.hint === "upload_banned" && /UTC/.test(r.err), r);
id = nid(); r = await user(B, `insert into public.items(id, autor, tipo, titulo, ruta_cuarentena) values ('${id}', '${B}', 'plantilla', 'Old client', '${B}/${id}/f.json')`);
check("banned: refused through the legacy view as well", !r.ok && /upload_banned/.test(r.err), r);
await owner(`update public.profiles set upload_banned_until = '9999-12-31 00:00:00+00' where id = '${B}'`);
id = nid(); r = await insNew(B, id); check("permanent ban message", !r.ok && r.hint === "upload_banned" && /disabled for this account\./.test(r.err), r);
await owner(`update public.profiles set upload_banned_until = now() - interval '1 second' where id = '${B}'`);
id = nid(); r = await insNew(B, id); check("an expired ban lets uploads through", r.ok, r);

const ID_S = nid();
await owner(`insert into public.community_items(id, author, kind, title) values ('${ID_S}', '${A}', 'template', 'Suspended-creator item')`);
await admin(`update public.community_items set status = 'approved', public_path = 'template/${ID_S}/f.json', reviewed_at = now() where id = '${ID_S}'`);
await admin(`update public.public_profiles set bio = 'hi' where id = '${A}'`);
const visible = async (uid, role) => (await as(uid, role, `select count(*)::int n from public.community_items where id = '${ID_S}'`)).rows?.[0]?.n;
check("before: the item is public", (await visible(null, "anon")) === 1 && (await visible(B, "authenticated")) === 1);
r = await admin(`select public.set_suspension('${A}', true)`); check("service_role can suspend", r.ok, r);
check("suspended: hidden from anon", (await visible(null, "anon")) === 0);
check("suspended: hidden from other people", (await visible(B, "authenticated")) === 0);
check("suspended: still visible to the owner", (await visible(A, "authenticated")) === 1);
check("suspended: service_role still sees it", (await visible(null, "service_role")) === 1);
r = await as(null, "anon", `select count(*)::int n from public.items where id = '${ID_S}'`); check("suspended: hidden through the legacy view `items`", r.rows[0].n === 0, r);
r = await user(B, `select count(*)::int n from public.public_profiles where id = '${A}'`); check("suspended: public profile hidden", r.rows[0].n === 0, r);
r = await user(A, `select count(*)::int n from public.my_public_profile`).catch?.(() => null);
r = await user(A, `select count(*)::int n from public.public_profiles where id = '${A}'`); check("suspended: the owner still reads their public profile", r.rows[0].n === 1, r);
await user(B, `select record_download('${ID_S}')`);
r = await owner(`select downloads from public.community_items where id = '${ID_S}'`); check("suspended: downloads are not counted", r.rows[0].downloads === 0, r);
id = nid(); r = await insNew(A, id); check("suspended: uploads refused (account_suspended)", !r.ok && r.hint === "account_suspended", r);
r = await user(A, `select public.set_suspension('${A}', false)`); check("people cannot call set_suspension", !r.ok, r);
await admin(`select public.set_suspension('${A}', false)`);
check("unsuspended: visible again", (await visible(null, "anon")) === 1);

// ═════════ 7. Copyright claims ═════════
// Claims need a profile >= 24 h old that accepted the terms (audit 3): age the fixtures and give D and E the legacy timestamp.
await owner(`update public.profiles set created_at = now() - interval '3 days', terms_accepted_at = coalesce(terms_accepted_at, now()) where id in ('${A}', '${B}', '${C}', '${D}', '${E}')`);
const ICL = nid();
await owner(`insert into public.community_items(id, author, kind, title) values ('${ICL}', '${A}', 'template', 'Claimed item')`);
await admin(`update public.community_items set status = 'approved', public_path = 'template/${ICL}/f.json', public_icon = 'template/${ICL}/icon.png', reviewed_at = now() where id = '${ICL}'`);
r = await as(null, "anon", `select public.file_copyright_claim('${ICL}', 'This is my work, see the original', 'Jane')`); check("anon cannot file a claim", !r.ok, r);
r = await user(C, `select public.file_copyright_claim('${ICL}', 'short', 'Jane')`); check("a too short description is refused (claim_invalid)", !r.ok && r.hint === "claim_invalid", r);
r = await user(C, `select public.file_copyright_claim('${ICL}', $1, 'Jane')`, ["x".repeat(2001)]); check("a description over 2000 is refused", !r.ok && r.hint === "claim_invalid", r);
r = await user(C, `select public.file_copyright_claim('${ICL}', 'This is my original artwork, see my site', $1)`, ["n".repeat(201)]); check("a claimant name over 200 is refused", !r.ok && r.hint === "claim_invalid", r);
r = await user(A, `select public.file_copyright_claim('${ICL}', 'I claim my own item, which is silly', 'Me')`); check("you cannot claim your own item", !r.ok && r.hint === "claim_invalid", r);
r = await user(C, `select public.file_copyright_claim('${nid()}', 'This is my original artwork, see my site', 'Jane')`); check("unknown item (item_not_found)", !r.ok && r.hint === "item_not_found", r);
r = await user(nid(), `select public.file_copyright_claim('${ICL}', 'This is my original artwork, see my site', 'Jane')`); check("no profile: refused", !r.ok, r);
const q0 = (await owner(`select count(*)::int n from public.storage_cleanup`)).rows[0].n;
r = await user(C, `select public.file_copyright_claim('${ICL}', 'This is my original artwork, see my site', 'Jane Artist', 'jane@example.org') as c`);
check("a claim is filed", r.ok && r.rows[0].c.status === "open" && /^[0-9a-f-]{36}$/.test(r.rows[0].c.claim), r);
const CL1 = r.rows?.[0]?.c?.claim;
r = await owner(`select status, rejection_reason from public.community_items where id = '${ICL}'`); check("the item is hidden at once (pending)", r.rows[0].status === "pending" && /copyright claim/.test(r.rows[0].rejection_reason), r);
r = await owner(`select action, path from public.storage_cleanup where item = '${ICL}' order by path`);
check("its public files are queued to_quarantine", r.rows.length === 2 && r.rows.every((x) => x.action === "to_quarantine"), r);
check("catalog no longer shows it", (await as(null, "anon", `select count(*)::int n from public.community_items where id = '${ICL}'`)).rows[0].n === 0);
r = await user(D, `select public.file_copyright_claim('${ICL}', 'Second claimant says the same thing here', 'Bob')`); check("a hidden item cannot be claimed again (item_not_found)", !r.ok && r.hint === "item_not_found", r);
r = await owner(`select claimant, claimant_contact, reporter, item_author, status from public.copyright_claims where id = '${CL1}'`);
check("the claim row stores claimant, contact and the author snapshot", r.rows[0].claimant === "Jane Artist" && r.rows[0].claimant_contact === "jane@example.org" && r.rows[0].reporter === C && r.rows[0].item_author === A && r.rows[0].status === "open", r);
r = await admin(`update public.community_items set status = 'approved' where id = '${ICL}'`); check("an item with an OPEN claim cannot be approved, not even by service_role", !r.ok && /copyright_claim_open/.test(r.err), r);
r = await user(A, `select public.resolve_claim('${CL1}', 'rejected')`); check("people cannot resolve claims", !r.ok, r);
r = await user(A, `select * from public.copyright_claims`); check("people cannot read claims", !r.ok, r);
r = await admin(`select public.mark_claim_counter('${CL1}', 'It is my work: I made it in 2024 and here is the project file.') as c`);
check("a counter-notice is recorded", r.ok && r.rows[0].c.status === "counter", r);
r = await owner(`select status, restore_after > now() + interval '13 days' as later from public.copyright_claims where id = '${CL1}'`); check("restore_after is set (about 14 days)", r.rows[0].status === "counter" && r.rows[0].later === true, r);
r = await admin(`update public.community_items set status = 'approved' where id = '${ICL}'`); check("a counter-noticed claim still blocks approval", !r.ok, r);
r = await admin(`select public.resolve_claim('${CL1}', 'rejected', 'person') as c`);
check("rejecting a claim: no strike", r.ok && r.rows[0].c.status === "rejected" && r.rows[0].c.strikes === null, r);
r = await owner(`select strikes from public.profiles where id = '${A}'`); check("the author has 0 strikes", r.rows[0].strikes === 0, r);
r = await admin(`update public.community_items set status = 'approved' where id = '${ICL}' returning status`); check("after rejecting the claim the item can be approved again", r.ok && r.rows[0].status === "approved", r);
r = await admin(`select public.resolve_claim('${CL1}', 'upheld')`); check("a resolved claim cannot be resolved twice (claim_not_open)", !r.ok && /claim_not_open/.test(r.err), r);
r = await admin(`select public.resolve_claim('${nid()}', 'upheld')`); check("unknown claim (claim_not_found)", !r.ok && /claim_not_found/.test(r.err), r);
r = await admin(`select public.resolve_claim('${CL1}', 'maybe')`); check("bad outcome refused", !r.ok, r);
r = await admin(`select public.mark_claim_counter('${CL1}', 'late')`); check("a counter-notice on a resolved claim is refused", !r.ok, r);

// strike ladder: 3 upheld claims = permanent upload ban
const fileAndUphold = async (n) => {
  const it = nid();
  await owner(`insert into public.community_items(id, author, kind, title) values ('${it}', '${A}', 'template', 'Infringing ${n}')`);
  await admin(`update public.community_items set status = 'approved', public_path = 'template/${it}/f.json', reviewed_at = now() where id = '${it}'`);
  const f = await user(D, `select public.file_copyright_claim('${it}', 'This copies my published artwork exactly', 'Dee') as c`);
  if (!f.ok) return { f };
  const u = await admin(`select public.resolve_claim('${f.rows[0].c.claim}', 'upheld', 'person') as c`);
  return { f, u, it };
};
let s = await fileAndUphold(1);
check("upheld #1: strike 1, no ban", s.u?.ok && s.u.rows[0].c.strikes === 1 && s.u.rows[0].c.upload_banned_until === null, s);
id = nid(); r = await insNew(A, id); check("after 1 strike uploads still work", r.ok, r);
s = await fileAndUphold(2);
check("upheld #2: strike 2, no ban", s.u?.ok && s.u.rows[0].c.strikes === 2 && s.u.rows[0].c.upload_banned_until === null, s);
s = await fileAndUphold(3);
check("upheld #3: strike 3 and a permanent upload ban", s.u?.ok && s.u.rows[0].c.strikes === 3 && /^9999-12-31/.test(s.u.rows[0].c.upload_banned_until), s);
id = nid(); r = await insNew(A, id); check("at 3 strikes uploads are refused (upload_banned)", !r.ok && r.hint === "upload_banned", r);
r = await admin(`select public.reverse_strike('${A}') as c`); check("an appeal: reverse_strike lifts the automatic ban below 3", r.ok && r.rows[0].c.strikes === 2 && r.rows[0].c.upload_banned_until === null, r);
id = nid(); r = await insNew(A, id); check("uploads work again after the appeal", r.ok, r);
r = await admin(`select public.reverse_strike('${A}') as c`); await admin(`select public.reverse_strike('${A}')`);
r = await admin(`select public.reverse_strike('${A}') as c`); check("strikes never go below 0", r.ok && r.rows[0].c.strikes === 0, r);
r = await user(A, `select public.reverse_strike('${A}')`); check("people cannot call reverse_strike", !r.ok, r);
r = await admin(`select public.set_upload_ban('${A}', now() + interval '1 day')`); r = await owner(`select upload_banned_until > now() as b from public.profiles where id = '${A}'`); check("set_upload_ban sets a manual ban", r.rows[0].b === true, r);
await admin(`select public.set_upload_ban('${A}', null)`);

// per-user rate limit (5 / day) and the false-claim block (3 rejected in 30 days)
const rl = [];
for (let i = 0; i < 6; i++) {
  const it = nid();
  await owner(`insert into public.community_items(id, author, kind, title) values ('${it}', '${B}', 'template', 'Rate ${i}')`);
  await admin(`update public.community_items set status = 'approved', public_path = 'template/${it}/f.json', reviewed_at = now() where id = '${it}'`);
  rl.push(await user(E, `select public.file_copyright_claim('${it}', 'This is a bogus claim to test the limit', 'Eve') as c`));
}
check("the 6th claim of the day is refused (claim_rate_limited)", rl.slice(0, 5).every((x) => x.ok) && !rl[5].ok && rl[5].hint === "claim_rate_limited", rl.map((x) => x.ok || x.hint));
r = await owner(`select id from public.copyright_claims where reporter = '${E}' limit 3`);
for (const row of r.rows) await admin(`select public.resolve_claim('${row.id}', 'rejected')`);
await owner(`update public.copyright_claims set created_at = now() - interval '2 days' where reporter = '${E}'`);
const itn = nid();
await owner(`insert into public.community_items(id, author, kind, title) values ('${itn}', '${C}', 'template', 'After rejects')`);
await admin(`update public.community_items set status = 'approved', public_path = 'template/${itn}/f.json', reviewed_at = now() where id = '${itn}'`);
r = await user(E, `select public.file_copyright_claim('${itn}', 'Yet another claim after being rejected', 'Eve')`);
check("3 rejected claims in 30 days block further in-app claims (claims_blocked)", !r.ok && r.hint === "claims_blocked", r);
r = await user(D, `select public.file_copyright_claim('${itn}', 'A different person can still file one', 'Dee')`); check("another person is unaffected", r.ok, r);

// ═════════ 7b. Audit hardening ═════════
// New helper: a profile that is old enough and accepted the terms (the claim preconditions), plus approved items.
const mkUser = async (name, { days = 3, terms = true } = {}) => {
  const u = nid();
  await owner(`insert into auth.users values ('${u}')`);
  await owner(`insert into public.profiles(id, public_name${terms ? ", terms_accepted_at" : ""}) values ('${u}', '${name}'${terms ? ", now()" : ""})`);
  await owner(`update public.profiles set name_reviewed = public_name, created_at = now() - interval '${days} days' where id = '${u}'`);
  return u;
};
const mkItems = async (author, n, status = "approved") => {
  const out = [];
  for (let i = 0; i < n; i++) {
    const x = nid(); out.push(x);
    await owner(`insert into public.community_items(id, author, kind, title) values ('${x}', '${author}', 'template', 'Audit item ${i}')`);
    if (status === "approved") await admin(`update public.community_items set status = 'approved', public_path = 'template/${x}/f.json', reviewed_at = now() where id = '${x}'`);
  }
  return out;
};
const RLS = /new row violates row-level security policy/;

// 1. Author-spoof oracle: inserting with ANOTHER account's id must not reveal that account's ban / suspension / terms state.
const V_SUSP = await mkUser("victim_susp"), V_BAN = await mkUser("victim_ban"), V_CLEAN = await mkUser("victim_clean"), V_NOTERMS = await mkUser("victim_noterms", { terms: false });
await admin(`select public.set_suspension('${V_SUSP}', true)`);
await admin(`select public.set_upload_ban('${V_BAN}', now() + interval '3 days')`);
await owner(`update public.app_settings set terms_enforced_from = now() - interval '1 minute'`);
const spoof = [];
for (const [n, v] of [["suspended", V_SUSP], ["banned", V_BAN], ["clean", V_CLEAN], ["no terms (enforced)", V_NOTERMS]]) {
  const x = nid();
  const res = await user(A, `insert into public.community_items(id, author, kind, title, quarantine_path) values ('${x}', '${v}', 'template', 'Spoofed', '${v}/${x}/f.json')`);
  spoof.push(res);
  check(`audit 1: spoofing author = ${n} account gives the plain RLS error, no code`, !res.ok && RLS.test(res.err) && res.code === "42501" && !res.hint && !/account_suspended|upload_banned|terms_required|UTC/.test(res.err), res);
}
check("audit 1: the answer is identical whatever the victim's state", new Set(spoof.map((x) => x.err)).size === 1, spoof.map((x) => x.err));
{
  const x = nid();
  const res = await user(A, `insert into public.items(id, autor, tipo, titulo, ruta_cuarentena) values ('${x}', '${V_BAN}', 'plantilla', 'Spoofed legacy', '${V_BAN}/${x}/f.json')`);
  check("audit 1: same through the legacy view items", !res.ok && RLS.test(res.err) && !/upload_banned|UTC/.test(res.err), res);
}
{
  const x = nid();
  const res = await as(null, "anon", `insert into public.community_items(id, author, kind, title, quarantine_path) values ('${x}', '${V_SUSP}', 'template', 'Spoofed anon', '${V_SUSP}/${x}/f.json')`);
  check("audit 1: anon gets the plain error too", !res.ok && !/account_suspended/.test(res.err), res);
}
check("audit 1: nothing was inserted", (await owner(`select count(*)::int n from public.community_items where title like 'Spoofed%'`)).rows[0].n === 0);
r = await owner(`select count(*)::int n from public.upload_log where author in ('${V_SUSP}', '${V_BAN}', '${V_CLEAN}', '${V_NOTERMS}')`); check("audit 1: no upload_log noise", r.rows[0].n === 0, r);
await owner(`update public.app_settings set terms_enforced_from = null`);
{
  const x = nid();
  const res = await insNew(V_CLEAN, x);
  check("audit 1: the real owner still uploads", res.ok, res);
  const res2 = await insNew(V_BAN, nid());
  check("audit 1: the real owner who is banned still gets upload_banned", !res2.ok && res2.hint === "upload_banned", res2);
}

// 3. Claim abuse
const R_NEW = await mkUser("reporter_new", { days: 0 }), R_NOTERMS = await mkUser("reporter_noterms", { terms: false });
const [T1] = await mkItems(V_CLEAN, 1);
r = await user(R_NEW, `select public.file_copyright_claim('${T1}', 'This is my own work, see my portfolio', 'Rita')`);
check("audit 3: a profile younger than 24 h cannot file a claim (profile_too_new)", !r.ok && r.hint === "profile_too_new", r);
r = await user(R_NOTERMS, `select public.file_copyright_claim('${T1}', 'This is my own work, see my portfolio', 'Rita')`);
check("audit 3: a profile that never accepted the terms cannot file a claim (terms_required)", !r.ok && r.hint === "terms_required", r);
check("audit 3: both left the item public", (await owner(`select status from public.community_items where id = '${T1}'`)).rows[0].status === "approved");
await owner(`update public.profiles set created_at = now() - interval '23 hours' where id = '${R_NEW}'`);
r = await user(R_NEW, `select public.file_copyright_claim('${T1}', 'This is my own work, see my portfolio', 'Rita')`);
check("audit 3: 23 h is still too new", !r.ok && r.hint === "profile_too_new", r);
await owner(`update public.profiles set created_at = now() - interval '25 hours' where id = '${R_NEW}'`);
r = await user(R_NEW, `select public.file_copyright_claim('${T1}', 'This is my own work, see my portfolio', 'Rita') as c`);
check("audit 3: 25 h is fine and the item is hidden", r.ok && r.rows[0].c.hidden === true, r);

// per-victim limit: the 4th and later open claims against the same author in 24 h are recorded and flagged, the item stays public
const VICTIM = await mkUser("victim_flood");
const vitems = await mkItems(VICTIM, 6);
const reps = []; for (let i = 0; i < 6; i++) reps.push(await mkUser(`rep_flood_${i}`));
const flood = [];
for (let i = 0; i < 6; i++) flood.push(await user(reps[i], `select public.file_copyright_claim('${vitems[i]}', 'Coordinated claim number ${i}, my work', 'Mob') as c`));
check("audit 3: every claim in the flood is accepted (recorded)", flood.every((x) => x.ok), flood.map((x) => x.ok || x.hint));
check("audit 3: the first 3 hide their items, the rest do not", flood.map((x) => x.rows?.[0]?.c?.hidden).join() === "true,true,true,false,false,false", flood.map((x) => x.rows?.[0]?.c));
r = await owner(`select i.status, c.flag from public.community_items i join public.copyright_claims c on c.item = i.id where i.author = '${VICTIM}' order by c.created_at, c.id`);
check("audit 3: items 4-6 stay approved and their claims are flagged author_flood", r.rows.filter((x) => x.status === "approved").length === 3 && r.rows.filter((x) => x.flag === "author_flood").length === 3
  && r.rows.filter((x) => x.status === "pending").every((x) => x.flag === null), r.rows);
r = await owner(`select count(*)::int n from public.moderation_log m join public.community_items i on i.id = m.item where i.author = '${VICTIM}' and m.detail like '%FLAGGED (author_flood)%'`); check("audit 3: the flag is written to moderation_log", r.rows[0].n === 3, r);
// a flagged claim can still be decided
{
  const fc = await owner(`select id from public.copyright_claims where flag = 'author_flood' and status = 'open' and item_author = '${VICTIM}' limit 1`);
  r = await admin(`select public.resolve_claim('${fc.rows[0].id}', 'rejected') as c`); check("audit 3: a flagged claim can be resolved", r.ok, r);
}

// claim-hidden items do not count in the 10-pending upload cap
const CAPPED = await mkUser("capped_author");
const pend = [];
for (let i = 0; i < 10; i++) { const x = nid(); pend.push(x); const o = await owner(`insert into public.community_items(id, author, kind, title) values ('${x}', '${CAPPED}', 'template', 'Pending ${i}')`); if (!o.ok) console.log("PENDING INSERT", o); }
r = await insNew(CAPPED, nid()); check("audit 3: control, 10 pending items block a new upload", !r.ok && /Too many uploads pending/.test(r.err), r);
for (let i = 0; i < 4; i++) await owner(`insert into public.copyright_claims(item, item_author, claimant, description) values ('${pend[i]}', '${CAPPED}', 'Troll', 'hidden by a claim, not by the author')`);
r = await insNew(CAPPED, nid()); check("audit 3: 4 of them are hidden by open claims: they no longer count, the upload works", r.ok, r);
await owner(`update public.copyright_claims set status = 'rejected', resolved_at = now() where item = '${pend[0]}'`);
r = await owner(`select count(*)::int n from public.community_items where author = '${CAPPED}' and status = 'pending'`);

// 4. Rate-limit race: PGlite is single-session, so the race itself cannot run; assert the serialization is in the function (a row lock BEFORE the counts)
r = await owner(`select pg_get_functiondef('public.file_copyright_claim(uuid,text,text,text)'::regprocedure) as d`);
const def = r.rows[0].d;
const lockAt = def.search(/from public\.profiles where id = v_uid for update/), countAt = def.search(/select count\(\*\) from public\.copyright_claims where reporter = v_uid/);
check("audit 4: file_copyright_claim locks the reporter's profile row before counting their claims", lockAt > 0 && countAt > lockAt, { lockAt, countAt });
check("audit 4: and serializes per victim author (advisory lock) before counting claims against them", /pg_advisory_xact_lock/.test(def) && def.search(/pg_advisory_xact_lock/) < def.search(/item_author = it\.author/));

// ═════════ 8. Reports reasons ═════════
for (const reason of ["copyright", "adult", "malware", "hate", "spam", "impersonation", "personal_data", "other", "adult_content", "hate_or_harassment"]) {
  const it = nid();
  await owner(`insert into public.community_items(id, author, kind, title) values ('${it}', '${A}', 'template', 'R ${reason}')`);
  await admin(`update public.community_items set status = 'approved' where id = '${it}'`);
  r = await user(D, `insert into public.reports(item, reporter, reason) values ('${it}', '${D}', '${reason}')`); check(`report reason accepted: ${reason}`, r.ok, r);
}
r = await user(D, `insert into public.reports(item, reporter, reason) values ('${IB}', '${D}', 'because')`); check("an unknown reason is refused", !r.ok, r);
r = await user(E, `insert into public.reportes(item, reportante, motivo) values ('${IB}', '${E}', 'odio_o_acoso')`); check("old client: Spanish reason through reportes still works", r.ok, r);
r = await owner(`select count(*)::int n from public.reports where item = '${IA}' and reason = 'copyright'`); check("the old report row stayed valid", r.rows[0].n === 1, r);

// ═════════ 9. Export ═════════
r = await as(null, "anon", `select public.export_my_data()`); check("anon cannot export", !r.ok, r);
r = await user(D, `select public.export_my_data() as d`);
const ex = r.rows?.[0]?.d;
check("export returns the caller's rows only", r.ok && ex.profile.id === D && ex.reports_filed.length >= 1 && ex.reports_filed.every((x) => x.reporter === D)
  && ex.copyright_claims_filed.length >= 1 && ex.items.every((x) => x.author === D) && ex.terms_log.every((x) => x.user_id === undefined), ex && Object.keys(ex));
r = await user(A, `select public.export_my_data() as d`);
const exA = r.rows[0].d;
check("export: items and claims against my items, without the claimant", exA.items.length >= 3 && exA.copyright_claims_against_my_items.length >= 3
  && exA.copyright_claims_against_my_items.every((x) => x.claimant === undefined && x.claimant_contact === undefined), exA.copyright_claims_against_my_items?.[0]);
check("export: contains no other person's id in the claims filed", exA.copyright_claims_filed.length === 0, exA.copyright_claims_filed);

// ═════════ 10. Account deletion keeps working ═════════
await user(E, `select public.accept_terms('1.0')`);
r = await owner(`select count(*)::int n from public.terms_log where user_id = '${E}'`); check("E has an acceptance logged", r.rows[0].n === 1, r);
r = await owner(`delete from auth.users where id = '${E}'`);
check("deleting an account (cascade) still works with terms_log and claims in place", r.ok, r);
r = await owner(`select (select count(*)::int from public.terms_log where user_id = '${E}') as l, (select count(*)::int from public.copyright_claims where reporter is null) as orphan`);
check("terms_log rows left with the account; their claims stay with reporter = null", r.rows[0].l === 0 && r.rows[0].orphan >= 5, r);
r = await owner(`delete from auth.users where id = '${A}'`); check("deleting an account with strikes and claims against its items works", r.ok, r);
r = await owner(`select count(*)::int n from public.copyright_claims where item_author = '${A}'`); check("claims about a deleted author's items are kept (snapshot)", r.rows[0].n >= 3, r);

// ═════════ 11. Rollback and re-apply (twice) ═════════
for (let round = 1; round <= 2; round++) {
  try { await db.exec(read(rb, "0015_terms_and_copyright.down.sql")); await db.exec(read(rb, "0015_terms_and_copyright.down.sql")); } catch (e) { console.log("FAILED rollback (round " + round + ")", e.message); process.exit(1); }
  r = await owner(`select to_regclass('public.terms_log') is null as gone, not exists (select 1 from information_schema.columns where table_name = 'profiles' and column_name = 'strikes') as nocol,
      not exists (select 1 from information_schema.columns where table_name = 'community_items' and column_name = 'license') as nolic`);
  check(`rollback ${round}: new objects gone`, r.rows[0].gone && r.rows[0].nocol && r.rows[0].nolic, r);
  const pid = nid();
  r = await user(C, `insert into public.items(id, autor, tipo, titulo, ruta_cuarentena) values ('${pid}', '${C}', 'plantilla', 'After rollback', '${C}/${pid}/x.json')`);
  check(`rollback ${round}: old client uploads (0014 behaviour)`, r.ok, r);
  const ir = nid(); await owner(`insert into public.community_items(id, author, kind, title) values ('${ir}', '${D}', 'template', 'Report target ${round}')`);
  r = await user(C, `insert into public.reportes(item, reportante, motivo) values ('${ir}', '${C}', 'otro')`); check(`rollback ${round}: legacy report works`, r.ok, r);
  r = await user(C, `insert into public.reports(item, reporter, reason) values ('${nid()}', '${C}', 'spam')`); check(`rollback ${round}: new reasons refused again`, !r.ok, r);
  try { await up(); await up(); } catch (e) { console.log("FAILED reapply (round " + round + ")", e.message); process.exit(1); }
  r = await user(C, `select public.current_terms() as t`); check(`reapply ${round}: current_terms back`, r.ok && r.rows[0].t.version === "1.0", r);
}

console.log(`${passes} passed, ${fails} failed`); process.exit(fails ? 1 : 0);
