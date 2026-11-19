// SQL test for migration 0014 (my_public_profile view + pending columns hidden) in PGlite.
//   PGLITE=file:///<path to pglite dist/index.js> node supabase/tests/0014_own_profile_view_pglite.mjs
// Replays 0001-0013, then 0014 (twice), checks the new behaviour and every legacy read/write path old clients (<= 1.4.1) use, then the rollback.
// The whole 0012 harness is ALSO run with 0013 + 0014 applied: `PGLITE=... LATER=0013,0014 node supabase/tests/0012_pglite.mjs`.
import fs from "node:fs"; import path from "node:path"; import { fileURLToPath } from "node:url";
const here = path.dirname(fileURLToPath(import.meta.url));
const { PGlite } = await import(process.env.PGLITE ?? "@electric-sql/pglite");
const src = fs.readFileSync(path.join(here, "0012_pglite.mjs"), "utf8");
const boot = src.match(/const BOOT = `([\s\S]*?)`;/)[1];
const A = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa", B = "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb", C = "cccccccc-cccc-4ccc-8ccc-cccccccccccc";
const db = new PGlite(); let fails = 0, passes = 0;
const check = (n, c, x) => { if (c) passes++; else { fails++; console.log("FAIL", n, x === undefined ? "" : JSON.stringify(x)); } };
await db.exec(boot.replaceAll("${A}", A).replaceAll("${B}", B).replaceAll("${C}", C));
const mig = path.join(here, "..", "migrations"), rb = path.join(here, "..", "rollbacks");
const read = (dir, f) => fs.readFileSync(path.join(dir, f), "utf8").replace(/create extension if not exists pgcrypto;/, "");
for (const f of fs.readdirSync(mig).filter(f => /^00(0[1-9]|1[0-3])_/.test(f)).sort()) await db.exec(read(mig, f));
const as = async (uid, role, sql, params) => {
  await db.exec(`reset role; set role ${role}; select set_config('request.jwt.claim.sub', '${uid ?? ""}', false), set_config('request.jwt.claim.role', '${role}', false);`);
  try { const r = await db.query(sql, params); return { ok: true, rows: r.rows }; } catch (e) { return { ok: false, err: String(e.message) }; }
  finally { await db.exec("reset role; select set_config('request.jwt.claim.role', '', false), set_config('request.jwt.claim.sub', '', false);"); }
};
const admin = (sql, p) => as(null, "service_role", sql, p);
const owner = async (sql, p) => { try { const r = await db.query(sql, p); return { ok: true, rows: r.rows }; } catch (e) { return { ok: false, err: String(e.message) }; } };
const PUBLIC_COLS = "id, public_name, bio, github, links, public_avatar, public_banner, followers, hearts_received";

// Data: A has an approved item (so other signed-in people may see A's public profile) and a pending avatar; B and C are plain people.
await db.exec(`insert into public.profiles (id, public_name) values ('${A}', 'alice'), ('${B}', 'bobby'), ('${C}', 'carol');
  update public.profiles set name_reviewed = public_name;`);
const IA = crypto.randomUUID();
await owner(`insert into public.community_items(id, author, kind, title) values ('${IA}', '${A}', 'template', 'Alice template')`);
await admin(`update public.community_items set status = 'approved' where id = '${IA}'`);
await admin(`update public.public_profiles set avatar_pending = true, bio = 'hello' where id = '${A}'`);

// BEFORE 0014: the exposure the audit found
let r = await as(B, "authenticated", `select avatar_pending, updated_at from public.public_profiles where id = '${A}'`);
check("before: another signed-in person can read the pending flag from the table (the problem)", r.ok && r.rows[0]?.avatar_pending === true, r);

const up = () => db.exec(read(mig, "0014_own_profile_view.sql"));
try { await up(); await up(); } catch (e) { console.log("FAILED to apply 0014", e.message); process.exit(1); }
check("0014 applies twice", true);

// ── new behaviour ──
r = await as(B, "authenticated", `select avatar_pending from public.public_profiles where id = '${A}'`);
check("after: pending flag of the table is not readable by authenticated", !r.ok && /permission denied/.test(r.err), r);
for (const col of ["banner_pending", "updated_at"]) {
  r = await as(B, "authenticated", `select ${col} from public.public_profiles`);
  check(`after: ${col} not readable by authenticated`, !r.ok, r);
}
r = await as(B, "authenticated", `select * from public.public_profiles`);
check("after: select=* on the table fails for authenticated (clients must name columns)", !r.ok, r);
r = await as(null, "anon", `select * from public.public_profiles`);
check("after: select=* on the table fails for anon", !r.ok, r);
r = await as(B, "authenticated", `select ${PUBLIC_COLS} from public.public_profiles where id = '${A}'`);
check("after: public columns of a creator are readable by authenticated", r.ok && r.rows.length === 1 && r.rows[0].bio === "hello", r);
r = await as(null, "anon", `select ${PUBLIC_COLS} from public.public_profiles where id = '${A}'`);
check("after: public columns readable by anon (creator with an approved item)", r.ok && r.rows.length === 1, r);
r = await as(null, "anon", `select avatar_pending from public.public_profiles`);
check("after: anon cannot read pending", !r.ok, r);
r = await as(B, "authenticated", `update public.public_profiles set bio = 'x' where id = '${A}'`);
check("authenticated cannot write the table", !r.ok, r);

r = await as(A, "authenticated", `select * from public.my_public_profile`);
check("my_public_profile: select * returns the caller's row with the pending flag", r.ok && r.rows.length === 1 && r.rows[0].id === A && r.rows[0].avatar_pending === true && r.rows[0].banner_pending === false && r.rows[0].updated_at !== undefined && r.rows[0].bio === "hello", r);
const cols = Object.keys(r.rows?.[0] ?? {}).sort().join(",");
check("my_public_profile columns", cols === "avatar_pending,banner_pending,bio,followers,github,hearts_received,id,links,public_avatar,public_banner,public_name,updated_at", cols);
r = await as(A, "authenticated", `select id, public_name from public.my_public_profile where id = '${B}'`);
check("my_public_profile: asking for another id returns nothing", r.ok && r.rows.length === 0, r);
r = await as(B, "authenticated", `select * from public.my_public_profile`);
check("my_public_profile: B sees only B's own row (B has no pending)", r.ok && r.rows.length === 1 && r.rows[0].id === B && r.rows[0].avatar_pending === false, r);
r = await as(C, "authenticated", `select count(*)::int n from public.my_public_profile`);
check("my_public_profile: one row per caller", r.ok && r.rows[0].n === 1, r);
r = await as(null, "anon", `select * from public.my_public_profile`);
check("my_public_profile: anon denied", !r.ok, r);
r = await as(A, "authenticated", `update public.my_public_profile set bio = 'hacked' where id = '${A}'`);
check("my_public_profile is read-only for people", !r.ok, r);
r = await as(A, "authenticated", `insert into public.my_public_profile(id) values ('${A}')`);
check("my_public_profile is not insertable", !r.ok, r);
r = await admin(`select count(*)::int n from public.my_public_profile`);
check("service_role may query the view (no rows: it has no uid)", r.ok && r.rows[0].n === 0, r);
await admin(`update public.public_profiles set avatar_pending = false, banner_pending = true where id = '${A}'`);
r = await as(A, "authenticated", `select avatar_pending, banner_pending from public.my_public_profile`);
check("my_public_profile follows the server's flags", r.ok && r.rows[0].avatar_pending === false && r.rows[0].banner_pending === true, r);
await admin(`update public.public_profiles set avatar_pending = true, banner_pending = false where id = '${A}'`);

// ── server paths (service_role + definer triggers) still touch the hidden columns ──
r = await admin(`select id, avatar_pending, banner_pending, updated_at from public.public_profiles where id = '${A}'`);
check("service_role reads every column of the table", r.ok && r.rows[0].avatar_pending === true, r);
r = await admin(`update public.public_profiles set avatar_pending = false, updated_at = now() where id = '${A}'`);
check("service_role can update the hidden columns", r.ok, r);
await admin(`update public.public_profiles set avatar_pending = true where id = '${A}'`);
await owner(`update public.profiles set public_name = 'alice_x' where id = '${A}'`);
await admin(`select public.review_public_name('${A}', 'alice_x') v`);
r = await admin(`select public_name, updated_at is not null u from public.public_profiles where id = '${A}'`);
check("definer trigger propagate_author_name still writes public_profiles.updated_at", r.ok && r.rows[0].public_name === "alice_x" && r.rows[0].u, r);
r = await as(B, "authenticated", `insert into public.follows(follower, followed) values ('${B}', '${A}')`);
check("following a creator (definer trigger updates followers)", r.ok, r);
r = await as(B, "authenticated", `select followers from public.public_profiles where id = '${A}'`);
check("followers counter readable", r.ok && r.rows[0].followers === 1, r);
r = await as(B, "authenticated", `insert into public.hearts(person, item) values ('${B}', '${IA}')`);
check("heart (definer trigger updates hearts_received)", r.ok, r);
r = await as(B, "authenticated", `select hearts_received from public.public_profiles where id = '${A}'`);
check("hearts_received readable", r.ok && r.rows[0].hearts_received === 1, r);

// ── every legacy path old clients use ──
r = await as(null, "anon", `select id, nombre_publico, bio, github, enlaces, avatar_publico, banner_publico, seguidores, corazones_recibidos from public.perfiles_publicos`);
check("legacy (anon): perfiles_publicos public columns", r.ok && r.rows.length >= 1, r);
r = await as(null, "anon", `select * from public.perfiles_publicos`);
check("legacy (anon): perfiles_publicos select * fails exactly as before (column grants)", !r.ok, r);
r = await as(A, "authenticated", `select * from public.perfiles_publicos where id = '${A}'`);
check("legacy (authenticated): perfiles_publicos select * works (own flags)", r.ok && r.rows.length === 1 && r.rows[0].avatar_pendiente === true && r.rows[0].actualizado_en !== null, r);
r = await as(B, "authenticated", `select * from public.perfiles_publicos where id = '${A}'`);
check("legacy: another person's pending flags / actualizado_en stay hidden", r.ok && r.rows.length === 1 && r.rows[0].avatar_pendiente === false && r.rows[0].actualizado_en === null, r);
r = await as(B, "authenticated", `select nombre_publico, seguidores, corazones_recibidos from public.perfiles_publicos where id = '${A}'`);
check("legacy: authenticated reads a creator", r.ok && r.rows.length === 1, r);
r = await as(A, "authenticated", `select * from public.perfiles where id = '${A}'`);
check("legacy: perfiles select *", r.ok && r.rows.length === 1, r);
r = await as(A, "authenticated", `update public.perfiles set nombre_publico = 'alice_legacy', mostrar_18 = false where id = '${A}'`);
check("legacy: perfiles update", r.ok, r);
const II = crypto.randomUUID();
r = await as(A, "authenticated", `insert into public.items(id, autor, tipo, titulo, descripcion, etiquetas, es_18, version, ruta_cuarentena) values ('${II}', '${A}', 'plantilla', 'Legacy upload', 'd', '{a}', false, '1.0.0', '${A}/${II}/x.json')`);
check("legacy: items insert", r.ok, r);
r = await as(A, "authenticated", `update public.items set titulo = 'Legacy edit' where id = '${II}'`);
check("legacy: items update", r.ok, r);
r = await as(A, "authenticated", `update public.items set etiquetas = '{${"z".repeat(31)}}' where id = '${II}'`);
check("legacy: editing tags through items refuses a 31-char tag (tags cap on the update path)", !r.ok, r);
r = await as(A, "authenticated", `update public.items set etiquetas = '{1,2,3,4,5,6,7,8,9}' where id = '${II}'`);
check("legacy: editing tags through items refuses 9 tags", !r.ok, r);
r = await as(null, "anon", `select id, titulo, autor_nombre from public.items where estado = 'aprobado'`);
check("legacy (anon): items select", r.ok && r.rows.length >= 1, r);
r = await as(C, "authenticated", `insert into public.seguidores(seguidor, seguido) values ('${C}', '${A}')`);
check("legacy: seguidores insert", r.ok, r);
r = await as(C, "authenticated", `delete from public.seguidores where seguidor = '${C}' and seguido = '${A}'`);
check("legacy: seguidores delete", r.ok, r);
r = await as(C, "authenticated", `insert into public.corazones(persona, item) values ('${C}', '${IA}')`);
check("legacy: corazones insert", r.ok, r);
r = await as(A, "authenticated", `delete from public.items where id = '${II}'`);
check("legacy: items delete", r.ok, r);
r = await admin(`update public.perfiles_publicos set avatar_pendiente = false, actualizado_en = now(), bio = 'by server' where id = '${A}' returning id`);
check("legacy: service_role updates perfiles_publicos (INSTEAD OF trigger writes the hidden columns)", r.ok && r.rows.length === 1, r);
r = await admin(`select avatar_pending, bio from public.public_profiles where id = '${A}'`);
check("... and the change reached the table", r.ok && r.rows[0].avatar_pending === false && r.rows[0].bio === "by server", r);
r = await as(A, "authenticated", `update public.perfiles_publicos set bio = 'mine' where id = '${A}'`);
check("legacy: people still cannot update perfiles_publicos", !r.ok, r);
r = await as(C, "authenticated", `select public.record_download('${IA}')`);
check("record_download works", r.ok, r);

// ── names at creation (0012 + 0013 + the profile function's review_public_name call) ──
const D = crypto.randomUUID(), O = crypto.randomUUID();
await owner(`insert into auth.users values ('${D}'), ('${O}')`);
r = await as(D, "authenticated", `insert into public.profiles(id, public_name, terms_accepted_at) values ('${D}', 'dave_maker', now()) returning name_reviewed`);
check("a brand-new profile starts unreviewed", r.ok && r.rows[0].name_reviewed === null, r);
r = await owner(`select public_name from public.public_profiles where id = '${D}'`);
check("... and its public name is empty until reviewed", r.rows[0]?.public_name === "", r);
r = await admin(`select public.review_public_name('${D}', 'dave_maker') as v`);
check("the text review marks the new name", r.ok && r.rows[0].v === true, r);
r = await as(null, "anon", `select public_name from public.public_profiles where id = '${D}'`);
r = await owner(`select public_name from public.public_profiles where id = '${D}'`);
check("... the public profile now shows it", r.rows[0].public_name === "dave_maker", r);
r = await as(D, "authenticated", `select public_name, avatar_pending from public.my_public_profile`);
check("... and my_public_profile too", r.ok && r.rows[0].public_name === "dave_maker", r);
await owner(`insert into public.name_exemptions(user_id, note) values ('${D}', 'test') on conflict do nothing`);
r = await as(O, "authenticated", `insert into public.profiles(id, public_name, terms_accepted_at) values ('${O}', 'lienzo_team', now())`);
check("a normal new profile cannot take a reserved name", !r.ok, r);
await owner(`insert into public.profiles(id, public_name) values ('${O}', 'tmp_owner')`);
await owner(`insert into public.name_exemptions(user_id, note) values ('${O}', 'owner')`);
await owner(`update public.profiles set public_name = 'Deritu_Kutsut' where id = '${O}'`);
r = await admin(`select public.review_public_name('${O}', 'Deritu_Kutsut') as v`);
check("an exempt (owner) account's reserved-looking name is reviewed", r.ok && r.rows[0].v === true, r);
r = await owner(`select public_name from public.public_profiles where id = '${O}'`);
check("... and shown", r.rows[0].public_name === "Deritu_Kutsut", r);

// ── rollback and re-apply ──
for (let n = 1; n <= 2; n++) {
  try { await db.exec(read(rb, "0014_own_profile_view.down.sql")); await db.exec(read(rb, "0014_own_profile_view.down.sql")); } catch (e) { console.log("FAILED rollback", e.message); process.exit(1); }
  r = await owner(`select to_regclass('public.my_public_profile') is null as gone`);
  check(`rollback ${n}: view gone`, r.rows[0].gone, r);
  r = await as(B, "authenticated", `select * from public.public_profiles where id = '${A}'`);
  check(`rollback ${n}: authenticated reads the whole table again (pre-0014 behaviour)`, r.ok && r.rows.length === 1, r);
  r = await as(null, "anon", `select avatar_pending from public.public_profiles`);
  check(`rollback ${n}: anon still limited to public columns`, !r.ok, r);
  r = await as(null, "anon", `select ${PUBLIC_COLS} from public.public_profiles`);
  check(`rollback ${n}: anon public columns`, r.ok, r);
  try { await up(); } catch (e) { console.log("FAILED re-apply", e.message); process.exit(1); }
  r = await as(B, "authenticated", `select avatar_pending from public.public_profiles`);
  check(`re-apply ${n}: pending hidden again`, !r.ok, r);
  r = await as(A, "authenticated", `select * from public.my_public_profile`);
  check(`re-apply ${n}: view works`, r.ok && r.rows.length === 1, r);
}
console.log(`${passes} passed, ${fails} failed`); process.exit(fails ? 1 : 0);
