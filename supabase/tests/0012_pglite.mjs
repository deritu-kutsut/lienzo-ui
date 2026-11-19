// SQL tests for migrations 0001-0012 in PGlite (in-process Postgres, no Supabase, no network).
//   npm i @electric-sql/pglite            (anywhere; e.g. a scratch folder)
//   PGLITE=<path or package name> node supabase/tests/0012_pglite.mjs        (default package name: @electric-sql/pglite)
// Exit code 0 = every check passed. Supabase pieces that the migrations use (auth.uid/role/users, storage.*, the anon / authenticated /
// service_role roles) are stubbed in BOOT below; what is NOT covered: PostgREST, real Storage, RLS on Supabase-owned tables beyond the stubs.
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { QUARANTINE_CASES, PROFILE_QUARANTINE_CASES, PUBLIC_CASES, PROFILE_PUBLIC_CASES, IDS, fill } from "./hostile_paths.mjs";

const here = path.dirname(fileURLToPath(import.meta.url));
const mig = path.join(here, "..", "migrations");
const rb = path.join(here, "..", "rollbacks");
const spec = process.env.PGLITE ?? "@electric-sql/pglite";
const { PGlite } = await import(spec.startsWith("file:") || !/^[A-Za-z]:|^[./\\]/.test(spec) ? spec : pathToFileURL(spec).href);

const A = IDS.A, B = IDS.B, C = "cccccccc-cccc-4ccc-8ccc-cccccccccccc";
const db = new PGlite();
let fails = 0, passes = 0;
const check = (name, cond, extra) => {
  if (cond) passes++; else { fails++; console.log("FAIL", name, extra === undefined ? "" : JSON.stringify(extra)); }
};

const BOOT = `
create role anon nologin; create role authenticated nologin; create role service_role nologin bypassrls;
create schema auth; create table auth.users(id uuid primary key);
create function auth.uid() returns uuid language sql stable as $$ select nullif(current_setting('request.jwt.claim.sub', true),'')::uuid $$;
create function auth.role() returns text language sql stable as $$ select nullif(current_setting('request.jwt.claim.role', true),'') $$;
create schema storage;
create table storage.buckets(id text primary key, name text, public boolean, file_size_limit bigint, allowed_mime_types text[]);
create table storage.objects(id uuid default gen_random_uuid(), bucket_id text, name text, owner uuid);
alter table storage.objects enable row level security;
create function storage.foldername(name text) returns text[] language sql as $$ select string_to_array(name,'/') $$;
create function public.rls_auto_enable() returns void language sql as $$ select 1 $$;
grant usage on schema public, auth, storage to anon, authenticated, service_role;
grant select on all tables in schema storage to anon, authenticated, service_role;
grant insert on storage.objects to authenticated;
insert into auth.users values ('${A}'), ('${B}'), ('${C}');
`;
await db.exec(BOOT);

const read = (dir, f) => fs.readFileSync(path.join(dir, f), "utf8").replace(/create extension if not exists pgcrypto;/, "");
const up = (f) => db.exec(read(mig, f));
const down = (f) => db.exec(read(rb, f));

// Session helpers: `as` runs ONE statement as a role with JWT claims; `admin` as service_role; `owner` is the migration / SQL-editor session.
const as = async (uid, role, sql, params) => {
  await db.exec(`reset role; set role ${role}; select set_config('request.jwt.claim.sub', '${uid ?? ""}', false), set_config('request.jwt.claim.role', '${role}', false);`);
  try { const r = await db.query(sql, params); return { ok: true, rows: r.rows }; } catch (e) { return { ok: false, err: String(e.message) }; }
  finally { await db.exec("reset role; select set_config('request.jwt.claim.role', '', false), set_config('request.jwt.claim.sub', '', false);"); }
};
const admin = (sql, params) => as(null, "service_role", sql, params);
const owner = async (sql, params) => { try { const r = await db.query(sql, params); return { ok: true, rows: r.rows }; } catch (e) { return { ok: false, err: String(e.message) }; } };

// ═════════ 0. Replay 0001 - 0011, then LEGACY data that the 0012 migration must cope with ═════════
const files = fs.readdirSync(mig).filter((f) => /^\d{4}_.*\.sql$/.test(f)).sort();
for (const f of files.filter((x) => x < "0012")) {
  try { await up(f); } catch (e) { console.log("FAILED to apply", f, e.message); process.exit(1); }
}
const LI = "5555555a-5555-4555-8555-55555555555a"; // a legacy item with long tags and a legacy bad path
await db.exec(`
  insert into public.profiles (id, public_name) values ('${A}', 'alice'), ('${B}', 'bobby'), ('${C}', 'lienzo_official');
  insert into public.community_items (id, author, kind, title, tags, status, quarantine_path, public_path, sha256)
    values ('${LI}', '${A}', 'template', 'Legacy one', array['${"x".repeat(40)}', 'ok', 'a\u0001b'], 'approved', '${A}/${LI}/%2e%2e%2fx.json', 'plantilla/${LI}/design.json', repeat('a', 64));
`);

// ═════════ 1. Apply 0012 (twice: idempotent) ═════════
for (const n of [1, 2]) {
  try { await up("0012_security_fixes_2.sql"); } catch (e) { console.log("FAILED to apply 0012 (run " + n + ")", e.message); process.exit(1); }
}
check("0012 applied twice", true);

let r = await owner(`select tags, name_reviewed is not null as reviewed from public.community_items i, public.profiles p where i.id = '${LI}' and p.id = '${A}'`);
check("legacy long tags were trimmed (30 chars, control chars removed)", r.ok && r.rows[0].tags.length === 3 && r.rows[0].tags[0].length === 30 && r.rows[0].tags[2] === "ab", r);
r = await owner(`select public_name, name_reviewed from public.profiles order by public_name`);
check("existing names grandfathered except the reserved one", r.rows.find((x) => x.public_name === "alice")?.name_reviewed === "alice" && r.rows.find((x) => x.public_name === "lienzo_official")?.name_reviewed === null, r);
r = await owner(`select name_reviewed from public.profiles where id = '${A}'`);
await owner(`update public.profiles set public_name = 'alice2' where id = '${A}'`);
await up("0012_security_fixes_2.sql");
r = await owner(`select name_reviewed from public.profiles where id = '${A}'`);
check("re-applying does not grandfather a name changed after the migration", r.rows[0].name_reviewed === "alice", r);
await owner(`update public.profiles set public_name = 'alice' where id = '${A}'`);
r = await admin(`select * from legacy.bad_storage_paths()`);
check("legacy.bad_storage_paths lists the old bad row", r.ok && r.rows.some((x) => x.item === LI && x.col === "quarantine_path"), r);
r = await admin(`update public.community_items set rejection_reason = 'checked' where id = '${LI}' returning id`);
check("a legacy row with a bad stored path can still be updated by the server (trigger only checks CHANGED paths)", r.ok && r.rows.length === 1, r);
r = await admin(`update public.community_items set quarantine_path = '${A}/${LI}/..%2fy.json' where id = '${LI}'`);
check("but cannot be changed TO another bad path", !r.ok && /Invalid storage path/.test(r.err), r);
r = await admin(`update public.community_items set quarantine_path = null where id = '${LI}'`);
check("and can be cleared", r.ok, r);

// ═════════ 2. Path allowlist: SQL helpers vs the shared hostile list (parity with paths.ts) ═════════
const sqlBool = async (sql, params) => { const x = await owner(sql, params); return x.ok ? x.rows[0].v === true : false; /* an error (e.g. NUL) = rejected */ };
for (const [tpl, ok] of QUARANTINE_CASES) {
  const p = fill(tpl);
  check(`legacy.owns_path ${JSON.stringify(p)} = ${ok}`, (await sqlBool(`select legacy.owns_path($1::uuid, $2::uuid, $3) as v`, [A, IDS.I, p])) === ok);
  // (object_ok does not bind the item id, and also accepts the profile shape: only compare the cases where those do not matter)
  if (!tpl.startsWith("{A}/{J}/") && !tpl.startsWith("{A}/perfil/")) check(`quarantine_object_ok(item shape) ${JSON.stringify(p)}`, (await sqlBool(`select legacy.quarantine_object_ok($1, $2::uuid) as v`, [p, A])) === ok);
}
for (const [tpl, ok] of PROFILE_QUARANTINE_CASES) {
  const p = fill(tpl);
  check(`quarantine_object_ok(profile) ${JSON.stringify(p)} = ${ok}`, (await sqlBool(`select legacy.quarantine_object_ok($1, $2::uuid) as v`, [p, A])) === ok);
}
for (const [tpl, ok] of PUBLIC_CASES) {
  const p = fill(tpl);
  check(`legacy.public_path_ok ${JSON.stringify(p)} = ${ok}`, (await sqlBool(`select legacy.public_path_ok('template'::public.item_kind, $1::uuid, $2) as v`, [IDS.I, p])) === ok);
}
for (const [tpl, ok] of PROFILE_PUBLIC_CASES) {
  const p = fill(tpl);
  check(`legacy.profile_public_path_ok ${JSON.stringify(p)} = ${ok}`, (await sqlBool(`select legacy.profile_public_path_ok($1::uuid, $2) as v`, [A, p])) === ok);
}
check("owns_path(NULL) is true", await sqlBool(`select legacy.owns_path($1::uuid, $2::uuid, null) as v`, [A, IDS.I]));

// ═════════ 3. Inserts by people: the policy + trigger + legacy views ═════════
const nid = () => crypto.randomUUID();
const ins = (uid, id, cols = "", vals = "") =>
  as(uid, "authenticated", `insert into public.community_items(id, author, kind, title ${cols}) values ('${id}', '${uid}', 'template', 'Hello world' ${vals})`);
let id1 = nid();
r = await ins(A, id1, ", quarantine_path, quarantine_icon, tags", `, '${A}/${id1}/design.json', '${A}/${id1}/icon.png', '{one,two}'`);
check("a normal upload still works (base table)", r.ok, r);
const id2 = nid();
r = await as(A, "authenticated", `insert into public.items(id, autor, tipo, titulo, descripcion, etiquetas, es_18, version, ruta_cuarentena, icono_cuarentena)
  values ('${id2}', '${A}', 'plantilla', 'Hola mundo', 'd', '{x}', false, '1.0.0', '${A}/${id2}/Mi Diseno_v2.json', '${A}/${id2}/icono.png')`);
check("a normal upload still works through the legacy view `items` (old clients)", r.ok, r);
for (const [tpl] of QUARANTINE_CASES.filter(([, k]) => !k)) {
  const p = fill(tpl);
  if (p.includes("\u0000")) continue; // Postgres cannot even store NUL
  const id = nid();
  const rr = await as(A, "authenticated", `insert into public.community_items(id, author, kind, title, quarantine_path) values ('${id}', '${A}', 'template', 'Forged', $1)`, [p.replace("{I}", id).replace(IDS.I, id)]);
  // The item id inside the path is replaced with the real id so that ONLY the hostile part can be the reason for the refusal.
  check(`insert refused for the hostile path ${JSON.stringify(p)}`, !rr.ok, rr);
}
r = await as(A, "authenticated", `insert into public.items(id, autor, tipo, titulo, ruta_cuarentena) values ('${nid()}', '${A}', 'plantilla', 'Forged', '${A}/${IDS.J}/..%2f..%2f${B}%2fx.json')`);
check("hostile path through the legacy view refused", !r.ok, r);
const idBad = nid();
r = await as(A, "authenticated", `insert into public.community_items(id, author, kind, title, quarantine_path) values ('${idBad}', '${A}', 'template', 'Other folder', '${B}/${idBad}/x.json')`);
check("another person's folder refused", !r.ok, r);

// tags
for (const [name, tags, ok] of [
  ["8 tags ok", Array.from({ length: 8 }, (_, i) => `t${i}`), true],
  ["9 tags refused", Array.from({ length: 9 }, (_, i) => `t${i}`), false],
  ["30 chars ok", ["x".repeat(30)], true],
  ["31 chars refused", ["x".repeat(31)], false],
  ["200 total ok", ["x".repeat(30), "x".repeat(30), "x".repeat(30), "x".repeat(30), "x".repeat(30), "x".repeat(30), "x".repeat(20)], true],
  ["201 total refused", ["x".repeat(30), "x".repeat(30), "x".repeat(30), "x".repeat(30), "x".repeat(30), "x".repeat(30), "x".repeat(21)], false],
  ["control char refused", ["a\u0007b"], false],
]) {
  const id = nid();
  const rr = await as(A, "authenticated", `insert into public.community_items(id, author, kind, title, tags) values ('${id}', '${A}', 'template', 'Tags test', $1)`, [tags]);
  check(`tags: ${name}`, rr.ok === ok, rr);
  if (rr.ok) await admin(`delete from public.community_items where id = '${id}'`);
}
r = await as(A, "authenticated", `insert into public.items(id, autor, tipo, titulo, etiquetas) values ('${nid()}', '${A}', 'plantilla', 'Tags legacy', '{${"y".repeat(31)}}')`);
check("tags: legacy view refuses a 31-char tag too", !r.ok, r);
r = await as(A, "authenticated", `update public.community_items set tags = $1 where id = '${id1}'`, [["z".repeat(31)]]);
check("tags: editing an existing item to a long tag refused", !r.ok, r);
r = await as(A, "authenticated", `update public.community_items set tags = '{fine}' where id = '${id1}' returning tags`);
check("tags: a valid edit works", r.ok && r.rows[0].tags[0] === "fine", r);

// storage policy (the quarantine bucket insert)
const so = (uid, name, bucket = "cuarentena") => as(uid, "authenticated", `insert into storage.objects(bucket_id, name, owner) values ($1, $2, $3)`, [bucket, name, uid]);
check("storage: own item file accepted", (await so(A, `${A}/${id1}/design.json`)).ok);
check("storage: own profile image accepted", (await so(A, `${A}/perfil/avatar.png`)).ok);
check("storage: own banner accepted", (await so(A, `${A}/perfil/banner.png`)).ok);
for (const bad of [`${A}/${id1}/%2e%2e%2fx.json`, `${A}/${id1}/..`, `${A}/${id1}/.x`, `${A}/${id1}/x.`, `${A}/${id1}/x `, `${A}/${id1}/a\\b`, `${B}/${id1}/x.json`,
  `${A}/perfil/%2e%2e/avatar.png`, `${A}/perfil/other.png`, `${A}/x.json`, `${A}/${id1}/x/y.json`, `${A}/${id1}/x%00.json`, `${A.toUpperCase()}/${id1}/x.json`, `${A}/../${B}/${id1}/x.json`]) {
  check(`storage: refused ${JSON.stringify(bad)}`, !(await so(A, bad)).ok);
}
check("storage: refused in another bucket without a policy", !(await so(A, `${A}/${id1}/x.json`, "publico")).ok);

// ═════════ 4. The path trigger on server writes (public paths, profiles) ═════════
const id3 = nid();
await ins(A, id3, ", quarantine_path", `, '${A}/${id3}/f.json'`);
r = await admin(`update public.community_items set public_path = 'template/${id3}/..%2fx' where id = '${id3}'`);
check("server cannot store a hostile public path", !r.ok && /Invalid storage path/.test(r.err), r);
r = await admin(`update public.community_items set public_path = 'template/${id3}/f.json', quarantine_path = null where id = '${id3}'`);
check("server stores a correct public path", r.ok, r);
r = await admin(`update public.community_items set public_icon = 'template/${nid()}/icon.png' where id = '${id3}'`);
check("a public path under ANOTHER item id refused", !r.ok, r);
r = await admin(`update public.public_profiles set public_avatar = 'perfiles/${A}/../x.png' where id = '${A}'`);
check("hostile avatar path refused", !r.ok, r);
r = await admin(`update public.public_profiles set public_avatar = 'perfiles/${A}/avatar-0123456789abcdef.png' where id = '${A}'`);
check("correct avatar path accepted", r.ok, r);
r = await admin(`update public.public_profiles set public_avatar = 'perfiles/${B}/avatar-0123456789abcdef.png' where id = '${A}'`);
check("another person's avatar path refused", !r.ok, r);

// ═════════ 5. updated_at (optimistic lock) ═════════
r = await owner(`select updated_at as u from public.community_items where id = '${id1}'`);
const u0 = r.rows[0].u;
await admin(`update public.community_items set downloads = downloads + 1 where id = '${id1}'`);
r = await owner(`select updated_at as u from public.community_items where id = '${id1}'`);
check("a counter change does not move updated_at", +new Date(r.rows[0].u) === +new Date(u0), r);
await admin(`update public.community_items set title = 'New title here' where id = '${id1}'`);
r = await owner(`select updated_at as u from public.community_items where id = '${id1}'`);
check("a title change moves updated_at", +new Date(r.rows[0].u) > +new Date(u0), r);
r = await admin(`update public.community_items set updated_at = '2000-01-01' where id = '${id1}' returning updated_at`);
check("nobody can rewrite updated_at directly", r.ok && new Date(r.rows[0].updated_at).getFullYear() > 2000, r);

// ═════════ 6. Names ═════════
const pf = (uid, name) => as(uid, "authenticated", `insert into public.profiles(id, public_name) values ('${uid}', $1)`, [name]);
await owner(`delete from public.profiles where id = '${C}'`);
for (const n of ["lienzo", "Lienzo_UI", "L1enzo.fan", "deritu", "xx_deritu_xx", "admin", "sysadmin", "admin_joe", "official", "Official-Team", "moderator", "mod", "root", "system", "owner"]) {
  check(`reserved name refused: ${n}`, !(await pf(C, n)).ok);
}
const vp = await as(C, "authenticated", `insert into public.perfiles(id, nombre_publico) values ('${C}', 'lienzo_ui')`);
check("reserved name refused through the legacy view `perfiles`", !vp.ok, vp);
r = await as(C, "authenticated", `insert into public.profiles(id, public_name, name_reviewed) values ('${C}', 'carol', 'forged') returning name_reviewed`);
check("a new profile cannot write name_reviewed (forced to null)", r.ok && r.rows[0].name_reviewed === null, r);
r = await owner(`select public_name from public.public_profiles where id = '${C}'`);
check("the public profile of an unreviewed name is created with an EMPTY name", r.rows[0]?.public_name === "", r);
r = await as(C, "authenticated", `update public.profiles set name_reviewed = 'forged' where id = '${C}'`);
check("people cannot update name_reviewed (no column grant)", !r.ok, r);
r = await as(A, "authenticated", `update public.perfiles set nombre_publico = 'deritu' where id = '${A}'`);
check("renaming to a reserved name through the legacy view refused", !r.ok, r);
r = await as(A, "authenticated", `update public.perfiles set nombre_publico = 'alice_new' where id = '${A}'`);
check("a normal rename works (legacy view)", r.ok, r);
r = await owner(`select i.author_name, pp.public_name, p.public_name as private_name, p.name_reviewed from public.community_items i join public.public_profiles pp on pp.id = i.author join public.profiles p on p.id = i.author where i.id = '${id1}'`);
check("a NEW name is NOT copied to items / public profile until reviewed", r.rows[0].author_name === "alice" && r.rows[0].public_name === "alice" && r.rows[0].private_name === "alice_new", r);
r = await admin(`select public.review_public_name('${A}', 'alice_new') as v`);
check("review_public_name marks the CURRENT name", r.ok && r.rows[0].v === true, r);
r = await owner(`select i.author_name, pp.public_name from public.community_items i join public.public_profiles pp on pp.id = i.author where i.id = '${id1}'`);
check("after the review the reviewed name is propagated", r.rows[0].author_name === "alice_new" && r.rows[0].public_name === "alice_new", r);
r = await admin(`select public.review_public_name('${A}', 'someone_else') as v`);
check("review_public_name refuses a name that is not the current one", r.ok && r.rows[0].v === false, r);
r = await as(A, "authenticated", `select public.review_public_name('${A}', 'alice_new')`);
check("people cannot call review_public_name", !r.ok, r);
r = await as(A, "authenticated", `select * from public.reserved_names`);
check("people cannot read reserved_names", !r.ok, r);
const idn = nid();
await owner(`update public.profiles set name_reviewed = null where id = '${B}'`);
r = await as(B, "authenticated", `insert into public.community_items(id, author, kind, title) values ('${idn}', '${B}', 'template', 'New author item') returning author_name`);
check("an item by an author with an unreviewed name gets an EMPTY author_name", r.ok && r.rows[0].author_name === "", r);
await owner(`update public.profiles set name_reviewed = public_name where id = '${B}'`);
await admin(`delete from public.community_items where id = '${idn}'`);

// ═════════ 7. Upload cap that deleting does not dodge ═════════
await owner(`delete from public.upload_log where author = '${B}'`);
await owner(`delete from public.community_items where author = '${B}'`);
let accepted = 0, refused = "";
for (let i = 0; i < 25; i++) {
  const id = nid();
  const rr = await as(B, "authenticated", `insert into public.items(id, autor, tipo, titulo) values ('${id}', '${B}', 'plantilla', 'Cap test ${i}')`);
  if (rr.ok) {
    accepted++;
    // the author deletes it right away through the legacy view (this used to free a slot)
    await as(B, "authenticated", `delete from public.items where id = '${id}'`);
  } else { refused = rr.err; break; }
}
check("the daily cap (20) holds although the author deletes every item", accepted === 20 && /Daily upload limit/.test(refused), { accepted, refused });
r = await owner(`select count(*)::int as n from public.upload_log where author = '${B}'`);
check("upload_log kept the 20 uploads after the deletes", r.rows[0].n === 20, r);
await owner(`update public.upload_log set created_at = now() - interval '8 days' where author = '${B}'`);
r = await as(B, "authenticated", `insert into public.items(id, autor, tipo, titulo) values ('${nid()}', '${B}', 'plantilla', 'After retention')`);
check("entries older than the retention window stop counting (and are purged)", r.ok, r);
r = await owner(`select count(*)::int as n from public.upload_log where author = '${B}' and created_at < now() - interval '7 days'`);
check("old upload_log rows purged", r.rows[0].n === 0, r);
r = await as(B, "authenticated", `select * from public.upload_log`);
check("people cannot read upload_log", !r.ok, r);
r = await as(B, "authenticated", `delete from public.upload_log`);
check("people cannot delete from upload_log", !r.ok, r);
await owner(`delete from public.community_items where author = '${B}'`);

// ═════════ 8. Claims: lease, heartbeat, profile claims ═════════
const II = nid();
await ins(A, II, "", "");
r = await admin(`select public.claim_moderation('${II}', 5, 3, '${A}', 30) as v`);
check("claim ok", r.rows?.[0]?.v === "ok", r);
await admin(`update public.moderation_runs set processing_at = now() - interval '400 seconds' where item = '${II}'`);
r = await admin(`select public.claim_moderation('${II}', 5, 3, '${A}', 30) as v`);
check("a claim 400 s old is NOT taken over (lease 10 min > 5 min edit lock)", r.rows?.[0]?.v === "busy", r);
await admin(`update public.moderation_runs set processing_at = now() - interval '700 seconds' where item = '${II}'`);
r = await admin(`select public.claim_moderation('${II}', 50, 3, '${A}', 30) as v`);
check("a claim 700 s old IS taken over", r.rows?.[0]?.v === "ok", r);
await admin(`update public.moderation_runs set processing_at = now() - interval '500 seconds' where item = '${II}'`);
r = await admin(`select public.heartbeat_moderation('${II}')`);
r = await owner(`select processing_at > now() - interval '5 seconds' as fresh from public.moderation_runs where item = '${II}'`);
check("heartbeat extends the lease", r.rows[0].fresh === true, r);
r = await as(A, "authenticated", `update public.community_items set title = 'edit during a live run' where id = '${II}'`);
check("author edit refused during a live run (heartbeat keeps the edit lock alive)", !r.ok && /being reviewed/.test(r.err), r);
r = await as(A, "authenticated", `select public.heartbeat_moderation('${II}')`);
check("people cannot call heartbeat_moderation", !r.ok, r);
await admin(`select public.release_moderation('${II}', false)`);

// rate limit per kind
await owner(`delete from public.moderation_calls where author = '${A}'`);
r = await admin(`select public.claim_profile('${A}', 'image', 2, 600) as v`); check("claim_profile image #1 ok", r.rows?.[0]?.v === "ok", r);
r = await admin(`select public.claim_profile('${A}', 'image', 2, 600) as v`); check("claim_profile image while running = busy", r.rows?.[0]?.v === "busy", r);
await admin(`select public.release_profile('${A}', 'image')`);
r = await admin(`select public.claim_profile('${A}', 'image', 2, 600) as v`); check("claim_profile image #2 ok", r.rows?.[0]?.v === "ok", r);
await admin(`select public.release_profile('${A}', 'image')`);
r = await admin(`select public.claim_profile('${A}', 'image', 2, 600) as v`); check("claim_profile image #3 = rate (cap 2)", r.rows?.[0]?.v === "rate", r);
r = await admin(`select public.claim_profile('${A}', 'save', 2, 600) as v`); check("save has its own counter", r.rows?.[0]?.v === "ok", r);
await admin(`select public.release_profile('${A}', 'save')`);
r = await admin(`select public.claim_moderation('${II}', 5, 3, '${A}', 3) as v`);
check("the item rate counter ignores profile calls (2 image + 1 save calls, item cap 3)", r.rows?.[0]?.v === "ok", r);
await admin(`select public.release_moderation('${II}', false)`);
r = await admin(`select public.claim_profile('${A}', 'bogus', 2, 600) as v`); check("unknown kind refused", r.rows?.[0]?.v === "busy", r);
r = await as(A, "authenticated", `select public.claim_profile('${A}', 'image', 2, 600)`); check("people cannot call claim_profile", !r.ok, r);
r = await as(A, "authenticated", `select * from public.profile_image_pins`); check("people cannot read profile_image_pins", !r.ok, r);
r = await admin(`insert into public.profile_image_pins(person, class, sha256) values ('${A}', 'avatar', repeat('a', 64)) on conflict (person, class) do update set sha256 = excluded.sha256 returning sha256`);
check("service role can pin a profile image hash", r.ok, r);
r = await admin(`insert into public.profile_image_pins(person, class, sha256) values ('${A}', 'avatar', 'xyz') on conflict (person, class) do update set sha256 = excluded.sha256`);
check("a pin must be a sha256", !r.ok, r);

// ═════════ 9. Takedown queue ═════════
const IT = nid();
await ins(A, IT, ", quarantine_path, quarantine_icon", `, '${A}/${IT}/f.json', '${A}/${IT}/icon.png'`);
await admin(`update public.community_items set status = 'approved', public_path = 'template/${IT}/f.json', public_icon = 'template/${IT}/icon.png', quarantine_path = null, quarantine_icon = null, reviewed_at = now() where id = '${IT}'`);
r = await admin(`select count(*)::int as n from public.storage_cleanup`);
const q0 = r.rows[0].n;
// a. the author deletes an approved item (legacy view)
r = await as(A, "authenticated", `delete from public.items where id = '${IT}'`);
check("the author can still delete through the legacy view", r.ok, r);
r = await admin(`select bucket, path, action from public.storage_cleanup where item = '${IT}' order by path`);
check("author delete queued both public files (delete)", r.rows.length === 2 && r.rows.every((x) => x.bucket === "publico" && x.action === "delete"), r);
// b. a rejected item that kept its public files
const IR = nid();
await ins(A, IR, "", "");
await admin(`update public.community_items set status = 'approved', public_path = 'template/${IR}/f.json' where id = '${IR}'`);
await admin(`update public.community_items set status = 'pending' where id = '${IR}'`);
await admin(`update public.community_items set status = 'rejected' where id = '${IR}'`);
r = await admin(`select action from public.storage_cleanup where item = '${IR}'`);
check("rejecting an item that still had public files queues them for deletion", r.rows.length === 1 && r.rows[0].action === "delete", r);
await admin(`update public.community_items set status = 'withdrawn' where id = '${IR}'`);
r = await admin(`select count(*)::int as n from public.storage_cleanup where item = '${IR}'`);
check("the queue has no duplicates (open rows are unique)", r.rows[0].n === 1, r);
// c. hidden by malware / copyright reports → to_quarantine
const IH = nid();
await ins(B, IH, "", "");
await admin(`update public.community_items set status = 'approved', public_path = 'template/${IH}/f.json', public_icon = 'template/${IH}/icon.png', reviewed_at = now() - interval '1 hour' where id = '${IH}'`);
const rep = async (uid, reason, ago = "0 seconds") => as(uid, "authenticated", `insert into public.reports(item, reporter, reason, created_at) values ('${IH}', '${uid}', '${reason}', now() - interval '${ago}')`);
await owner(`insert into auth.users values ('${nid()}') `);
// old reports (before the approval) must not count
const olds = [nid(), nid(), nid()];
for (const o of olds) await owner(`insert into auth.users values ('${o}') on conflict do nothing`);
for (const o of olds) await owner(`insert into public.profiles(id, public_name) values ('${o}', 'u${o.slice(0, 6)}') on conflict do nothing`);
for (const o of olds) await owner(`insert into public.reports(item, reporter, reason, created_at) values ('${IH}', '${o}', 'malware', now() - interval '2 hours')`);
r = await owner(`select status from public.community_items where id = '${IH}'`);
check("3 reports from BEFORE the last approval do not hide the item", r.rows[0].status === "approved", r);
const news = [nid(), nid(), nid()];
for (const o of news) { await owner(`insert into auth.users values ('${o}')`); await owner(`insert into public.profiles(id, public_name) values ('${o}', 'n${o.slice(0, 6)}')`); }
await owner(`insert into public.reports(item, reporter, reason) values ('${IH}', '${news[0]}', 'other')`);
await owner(`insert into public.reports(item, reporter, reason) values ('${IH}', '${news[1]}', 'malware')`);
r = await owner(`select status from public.community_items where id = '${IH}'`);
check("2 reports since the approval (+3 old) do not hide it", r.rows[0].status === "approved", r);
await owner(`insert into public.reports(item, reporter, reason) values ('${IH}', '${news[2]}', 'malware')`);
r = await owner(`select status from public.community_items where id = '${IH}'`);
check("3 reports since the approval hide it", r.rows[0].status === "pending", r);
r = await admin(`select path, action from public.storage_cleanup where item = '${IH}' order by path`);
check("two malware reports queue the public files to be MOVED to quarantine (not deleted)", r.rows.length === 2 && r.rows.every((x) => x.action === "to_quarantine"), r);
// d. generic reports (no malware / copyright) hide but keep the files
const IG = nid();
await ins(B, IG, "", "");
await admin(`update public.community_items set status = 'approved', public_path = 'template/${IG}/f.json', reviewed_at = now() - interval '1 hour' where id = '${IG}'`);
for (const o of news) await owner(`insert into public.reports(item, reporter, reason) values ('${IG}', '${o}', 'other')`);
r = await owner(`select (select status::text from public.community_items where id = '${IG}') as s, (select count(*)::int from public.storage_cleanup where item = '${IG}') as q`);
check("3 generic reports hide the item and keep its public files (re-approvable)", r.rows[0].s === "pending" && r.rows[0].q === 0, r);
// e. claim / finish
r = await admin(`select id, attempts from public.claim_cleanup(2)`);
check("claim_cleanup hands out at most the limit", r.ok && r.rows.length === 2 && r.rows.every((x) => x.attempts === 1), r);
const claimed = r.rows.map((x) => x.id);
r = await admin(`select count(*)::int as n from public.claim_cleanup(100)`);
const rest = r.rows[0].n;
check("claimed rows are locked for other drainers", rest === (await owner(`select count(*)::int as n from public.storage_cleanup where done_at is null`)).rows[0].n - 2, rest);
await admin(`select public.finish_cleanup(array[${claimed.join(",")}]::bigint[], true)`);
r = await admin(`select count(*)::int as n from public.storage_cleanup where id = any(array[${claimed.join(",")}]::bigint[]) and done_at is not null`);
check("finish_cleanup marks rows done", r.rows[0].n === 2, r);
await admin(`select public.finish_cleanup(array[${claimed.join(",")}]::bigint[], false)`);
r = await admin(`select count(*)::int as n from public.storage_cleanup where id = any(array[${claimed.join(",")}]::bigint[]) and done_at is not null`);
check("a closed row is never reopened", r.rows[0].n === 2, r);
r = await as(A, "authenticated", `select * from public.storage_cleanup`); check("people cannot read storage_cleanup", !r.ok, r);
r = await as(A, "authenticated", `select * from public.claim_cleanup(5)`); check("people cannot call claim_cleanup", !r.ok, r);
r = await as(null, "anon", `select * from public.storage_cleanup`); check("anon cannot read storage_cleanup", !r.ok, r);

// ═════════ 10. Downloads: public items only; 18+ only for people who opted in ═════════
const ID_ = nid(), ID18 = nid(), IDP = nid();
for (const [id, extra] of [[ID_, ""], [ID18, ", is_18 = true"], [IDP, ""]]) {
  await owner(`insert into public.community_items(id, author, kind, title) values ('${id}', '${A}', 'template', 'dl test')`);
}
await admin(`update public.community_items set status = 'approved' where id in ('${ID_}', '${ID18}')`);
await admin(`update public.community_items set is_18 = true where id = '${ID18}'`);
await owner(`update public.profiles set show_18 = false where id = '${B}'`);
await as(B, "authenticated", `select registrar_descarga('${ID_}')`);
await as(B, "authenticated", `select registrar_descarga('${ID18}')`);
await as(B, "authenticated", `select public.record_download('${IDP}')`);
r = await owner(`select (select downloads from public.community_items where id = '${ID_}') as a, (select downloads from public.community_items where id = '${ID18}') as b, (select downloads from public.community_items where id = '${IDP}') as c`);
check("download counted for a public item; ignored for 18+ (not opted in) and for a pending item", r.rows[0].a === 1 && r.rows[0].b === 0 && r.rows[0].c === 0, r);
await owner(`update public.profiles set declared_adult_at = now(), show_18 = true where id = '${B}'`);
await as(B, "authenticated", `select public.record_download('${ID18}')`);
r = await owner(`select downloads from public.community_items where id = '${ID18}'`);
check("18+ counts for a person who opted in", r.rows[0].downloads === 1, r);
await as(B, "authenticated", `select public.record_download('${ID_}')`);
r = await owner(`select downloads from public.community_items where id = '${ID_}'`);
check("still once per (person, item)", r.rows[0].downloads === 1, r);
r = await as(null, "anon", `select registrar_descarga('${ID_}')`); check("anon still cannot call registrar_descarga", !r.ok, r);

// ═════════ 11. Old-client flows still work ═════════
const IL = nid();
r = await as(A, "authenticated", `insert into public.items(id, autor, tipo, titulo, descripcion, etiquetas, es_18, version, ruta_cuarentena, icono_cuarentena, vista_cuarentena)
  values ('${IL}', '${A}', 'imagen', 'Old client upload', 'desc', '{a,b}', false, '1.0.0', '${A}/${IL}/pic.png', '${A}/${IL}/icono.png', '${A}/${IL}/vista.png')`);
check("old client: insert through items with Spanish values", r.ok, r);
r = await as(A, "authenticated", `update public.items set titulo = 'Old client edit', etiquetas = '{c}' where id = '${IL}' returning titulo, estado`);
check("old client: edit through items", r.ok && r.rows[0].titulo === 'Old client edit' && r.rows[0].estado === 'pendiente', r);
r = await as(A, "authenticated", `select id, tipo, estado, titulo, autor_nombre from public.items where id = '${IL}'`);
check("old client: select through items", r.ok && r.rows.length === 1, r);
r = await as(A, "authenticated", `select nombre_publico, mostrar_18 from public.perfiles where id = '${A}'`); check("old client: read perfiles", r.ok && r.rows.length === 1, r);
r = await as(null, "anon", `select id, nombre_publico from public.perfiles_publicos`); check("old client (anon): select perfiles_publicos columns", r.ok, r);
r = await as(A, "authenticated", `select avatar_pendiente, actualizado_en from public.perfiles_publicos where id = '${A}'`); check("old client: own pending flags through the view", r.ok, r);
r = await as(B, "authenticated", `select avatar_pendiente, actualizado_en from public.perfiles_publicos where id = '${A}'`);
check("pending flags / updated_at of ANOTHER person are hidden in the legacy view", r.ok && (r.rows.length === 0 || (r.rows[0].avatar_pendiente === false && r.rows[0].actualizado_en === null)), r);
r = await as(A, "authenticated", `insert into public.corazones(persona, item) values ('${A}', '${ID_}')`); check("old client: heart through corazones", r.ok, r);

// ═════════ 12. Rollback + reapply (twice) ═════════
for (let round = 1; round <= 2; round++) {
  await admin(`delete from public.storage_cleanup`);
  try { await down("0012_security_fixes_2.down.sql"); await down("0012_security_fixes_2.down.sql"); } catch (e) { console.log("FAILED rollback (round " + round + ")", e.message); process.exit(1); }
  r = await owner(`select to_regclass('public.storage_cleanup') is null as gone, not exists (select 1 from information_schema.columns where table_name = 'community_items' and column_name = 'updated_at') as nocol`);
  check(`rollback ${round}: new objects gone`, r.rows[0].gone && r.rows[0].nocol, r);
  const pid = nid();
  r = await as(A, "authenticated", `insert into public.items(id, autor, tipo, titulo, ruta_cuarentena) values ('${pid}', '${A}', 'plantilla', 'After rollback', '${A}/${pid}/x.json')`);
  check(`rollback ${round}: the app still uploads (0011 behaviour)`, r.ok, r);
  r = await as(A, "authenticated", `insert into public.items(id, autor, tipo, titulo, ruta_cuarentena) values ('${nid()}', '${A}', 'plantilla', 'Traversal', '${A}/../x.json')`);
  check(`rollback ${round}: a literal «..» is still refused (0011 helper restored)`, !r.ok, r);
  await owner(`delete from public.community_items where id = '${pid}'`);
  try { await up("0012_security_fixes_2.sql"); await up("0012_security_fixes_2.sql"); } catch (e) { console.log("FAILED reapply (round " + round + ")", e.message); process.exit(1); }
  r = await owner(`select to_regclass('public.storage_cleanup') is not null as there`);
  check(`reapply ${round}: new objects back`, r.rows[0].there, r);
  r = await as(A, "authenticated", `insert into public.items(id, autor, tipo, titulo, ruta_cuarentena) values ('${nid()}', '${A}', 'plantilla', 'Traversal 2', '${A}/${IDS.I}/..%2fx.json')`);
  check(`reapply ${round}: encoded dot-segments refused again`, !r.ok, r);
}

console.log(`${passes} passed, ${fails} failed`);
process.exit(fails ? 1 : 0);
