// SQL test for migration 0013 (name exemptions) in PGlite. PGLITE=file:///<path to pglite dist/index.js> node supabase/tests/0013_names_pglite.mjs
import fs from "node:fs"; import path from "node:path"; import { fileURLToPath } from "node:url";
const here = path.dirname(fileURLToPath(import.meta.url));
const { PGlite } = await import(process.env.PGLITE ?? "@electric-sql/pglite");
const src = fs.readFileSync(path.join(here, "0012_pglite.mjs"), "utf8");
const boot = src.match(/const BOOT = `([\s\S]*?)`;/)[1];
const A = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa", B = "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb", C = "cccccccc-cccc-4ccc-8ccc-cccccccccccc";
const db = new PGlite(); let fails = 0, passes = 0;
const check = (n, c, x) => { if (c) passes++; else { fails++; console.log("FAIL", n, x === undefined ? "" : JSON.stringify(x)); } };
await db.exec(boot.replaceAll("${A}", A).replaceAll("${B}", B).replaceAll("${C}", C));
const dir = path.join(here, "..", "migrations");
for (const f of fs.readdirSync(dir).filter(f => /^00(0[1-9]|1[0-2])_/.test(f)).sort()) await db.exec(fs.readFileSync(path.join(dir, f), "utf8").replace(/create extension if not exists pgcrypto;/, ""));
// BEFORE 0013: the owner's name is reserved -> never reviewed
await db.exec(`insert into public.profiles (id, public_name) values ('${A}', 'Alice'), ('${B}', 'Deritu_Kutsut') on conflict do nothing;`);
const before = (await db.query(`select id, name_reviewed from public.profiles order by id`)).rows;
const apply = async () => db.exec(fs.readFileSync(path.join(dir, "0013_name_exemptions.sql"), "utf8"));
await apply(); await apply(); // idempotent
const p = Object.fromEntries((await db.query(`select id, name_reviewed from public.profiles`)).rows.map(r => [r.id, r.name_reviewed]));
check("owner name is now reviewed", p[B] === "Deritu_Kutsut", p);
check("other people untouched", p[A] === (before.find(r => r.id === A)?.name_reviewed ?? null), p);
check("owner is exempt", (await db.query(`select count(*)::int n from public.name_exemptions`)).rows[0].n === 1);
await db.exec(`set role service_role`);
check("review_public_name accepts the owner", (await db.query(`select public.review_public_name('${B}', 'Deritu_Kutsut') ok`)).rows[0].ok === true);
await db.exec(`insert into public.profiles (id, public_name) values ('${C}', 'tmp') on conflict do nothing; update public.profiles set public_name = 'Lienzo_Official' where id = '${C}'`);
check("service role can set but review refuses a reserved name for others", (await db.query(`select public.review_public_name('${C}', 'Lienzo_Official') ok`)).rows[0].ok === false);
await db.exec(`reset role`);
// people cannot pick a reserved name, but the exempt owner can keep theirs
await db.exec(`set role authenticated; select set_config('request.jwt.claim.role','authenticated',false), set_config('request.jwt.claim.sub','${A}',false)`);
let blocked = false; try { await db.exec(`update public.profiles set public_name = 'Deritu_Fake' where id = '${A}'`); } catch (e) { blocked = /not available/.test(String(e.message)); }
check("a normal user still cannot take a reserved name", blocked);
await db.exec(`select set_config('request.jwt.claim.sub','${B}',false)`);
let ok = true; try { await db.exec(`update public.profiles set public_name = 'Deritu_Kutsut' where id = '${B}'`); } catch (e) { ok = false; }
check("the exempt owner can keep/rename within their name", ok);
await db.exec(`reset role`);
// rollback restores the old rule
await db.exec(fs.readFileSync(path.join(here, "..", "rollbacks", "0013_name_exemptions.down.sql"), "utf8"));
check("rollback drops the table", (await db.query(`select to_regclass('public.name_exemptions') t`)).rows[0].t === null);
await apply();
check("re-apply after rollback", (await db.query(`select count(*)::int n from public.name_exemptions`)).rows[0].n === 1);
console.log(`${passes} passed, ${fails} failed`); process.exit(fails ? 1 : 0);
