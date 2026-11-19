// Storage path cases shared by the Node tests of _shared/paths.ts and the pglite tests of migration 0012 (so both sides are checked against the
// SAME list and cannot drift). Placeholders: {A} = the author uid, {I} = the item id, {B} = another person's uid, {J} = another item id.
// Each entry is [path template, expected result for «a quarantine path owned by author {A} and item {I}»].
const U = "{A}/{I}/";
const hostileFile = (f) => [U + f, false];

export const QUARANTINE_CASES = [
  // ── what the clients 1.0 - 1.5 really send ──
  [U + "plugin.json", true],
  [U + "mi-diseno_v2.json", true],
  [U + "icon.png", true],
  [U + "preview.png", true],
  [U + "icono.png", true],
  [U + "Font Name.ttf", true],
  [U + "a.b.c.json", true],
  [U + "x", true],
  [U + "_a", true],
  [U + "-a-", true],
  [U + "A".repeat(100), true],
  // ── encoded / literal dot segments ──
  hostileFile(".."), hostileFile("."), hostileFile("..%2f"), hostileFile("%2e%2e"), hostileFile("%2E%2E"), hostileFile("%2e%2e%2f"),
  hostileFile("%2e%2E/x.json"), hostileFile("..%2fx.json"), hostileFile("..%5cx.json"), hostileFile("%252e%252e"), hostileFile("%25%32%65"),
  hostileFile(".%2e"), hostileFile("%2e."), hostileFile("a..b.json"), hostileFile("a/../b.json"), hostileFile("...json"),
  ["{A}/{I}/../{J}/x.json", false], ["{A}/../{B}/{I}/x.json", false], ["{A}/{I}/./x.json", false], ["{A}/./{I}/x.json", false],
  ["../{A}/{I}/x.json", false], ["./{A}/{I}/x.json", false],
  // ── leading dot, trailing dot / space ──
  hostileFile(".hidden"), hostileFile(".json"), hostileFile("x."), hostileFile("x.json."), hostileFile("x "), hostileFile(" x"), hostileFile("x.json "),
  hostileFile("x. "), hostileFile("x .json "), hostileFile(" "), hostileFile(""),
  // ── special characters ──
  hostileFile("x%00.json"), hostileFile("x\u0000.json"), hostileFile("x\u0000"), hostileFile("x\n.json"), hostileFile("x.json\n"), hostileFile("x\r.json"),
  hostileFile("x\t.json"), hostileFile("x?.json"), hostileFile("x#.json"), hostileFile("x%.json"), hostileFile("x%20.json"), hostileFile("x\\y.json"),
  hostileFile("x\\..\\y"), hostileFile("..\\x"), hostileFile("x:y.json"), hostileFile("x*.json"), hostileFile("x|y"), hostileFile("x<y>"), hostileFile("x\"y"),
  hostileFile("x'y"), hostileFile("x;y"), hostileFile("x,y"), hostileFile("x&y"), hostileFile("x=y"), hostileFile("x+y"), hostileFile("x@y"),
  hostileFile("x$y"), hostileFile("x!y"), hostileFile("x(y)"), hostileFile("x[y]"), hostileFile("x{y}"), hostileFile("x~y"), hostileFile("x`y"),
  // ── unicode look-alikes of dots, slashes and letters ──
  hostileFile("．．"), hostileFile("．．/x"), hostileFile("․․"), hostileFile("‥"), hostileFile("…"), hostileFile("··"),
  hostileFile("∕x"), hostileFile("／x"), hostileFile("⁄x"), hostileFile("⧸x"), hostileFile("∖x"), hostileFile("＼x"),
  hostileFile("café.json"), hostileFile("а.json"), hostileFile("x​.json"), hostileFile("x‮.json"), hostileFile("﻿x.json"),
  hostileFile("\u{1F600}.json"),
  // ── double encoding, case ──
  hostileFile("%252e%252e%252f"), hostileFile("%c0%ae%c0%ae"), hostileFile("%c0%af"), hostileFile("%2F"), hostileFile("%5C"), hostileFile("%2f%2e%2e"),
  ["{A}/{I}/X.JSON", true], // upper-case letters in a FILE name are fine
  ["{A_UP}/{I}/x.json", false], ["{A}/{I_UP}/x.json", false], // upper-case UUIDs never match what the database stores
  // ── length ──
  [U + "A".repeat(101), false], [U + "a".repeat(100) + ".", false], [U + "a".repeat(300), false],
  // ── wrong shape ──
  ["{A}/{I}", false], ["{A}/{I}/", false], ["{A}//x.json", false], ["{A}/{I}//x.json", false], ["/{A}/{I}/x.json", false], ["{A}/{I}/x/y.json", false],
  ["{A}/{I}/x.json/", false], ["{A}/x.json", false], ["x.json", false], ["", false], [" {A}/{I}/x.json", false], ["{A}/{I}/x.json ", false],
  ["{A}\\{I}\\x.json", false], ["{A}/perfil/avatar.png", false], // (a profile image path is not an item path)
  // ── somebody else's folder ──
  ["{B}/{I}/x.json", false], ["{A}/{J}/x.json", false], ["{B}/{J}/x.json", false],
];

/** Profile image names in quarantine: [path template, expected ok for author {A}]. */
export const PROFILE_QUARANTINE_CASES = [
  ["{A}/perfil/avatar.png", true], ["{A}/perfil/banner.png", true],
  ["{A}/perfil/avatar.PNG", false], ["{A}/perfil/avatar.png ", false], ["{A}/perfil/avatar.png.", false], ["{A}/perfil/other.png", false],
  ["{A}/perfil/../avatar.png", false], ["{A}/perfil/%2e%2e/avatar.png", false], ["{A}/perfil/avatar%2epng", false], ["{A}/perfil/avatar.png\n", false],
  ["{B}/perfil/avatar.png", false], ["{A}/perfil//avatar.png", false], ["{A}/Perfil/avatar.png", false], ["{A}/perfiles/avatar.png", false],
  ["{A}/perfil/avatar.png/x", false], ["{A}/perfil/avatar\u0000.png", false],
];

/** Public object names: [path template, expected ok for an item of kind {K} and id {I}]. */
export const PUBLIC_CASES = [
  ["template/{I}/design.json", true], ["template/{I}/icon.png", true], ["plantilla/{I}/design.json", true], // pre-0010 Spanish directory of a template
  ["template/{I}/..", false], ["template/{I}/%2e%2e", false], ["template/{I}/.x", false], ["template/{I}/x.", false], ["template/{I}/x ", false],
  ["template/{I}/x%00", false], ["template/{I}/a..b", false], ["template/../{I}/x", false], ["template/{J}/x.json", false], ["font/{I}/x.json", false],
  ["fuente/{I}/x.json", false], ["Template/{I}/x.json", false], ["template/{I}//x.json", false], ["/template/{I}/x.json", false],
  ["template/{I}/x\\y", false], ["template/{I}/．．", false], ["template/{I}/x?y", false], ["template/{I}/x#y", false],
  ["perfiles/{A}/avatar.png", false], ["{A}/{I}/x.json", false],
];

/** Public profile image names: [path template, expected ok for profile {A}]. */
export const PROFILE_PUBLIC_CASES = [
  ["perfiles/{A}/avatar-0123456789abcdef.png", true], ["perfiles/{A}/banner-0123456789abcdef.png", true], ["perfiles/{A}/avatar.png", true],
  ["perfiles/{A}/avatar-0123456789abcde.png", false], ["perfiles/{A}/avatar-0123456789ABCDEF.png", false], ["perfiles/{A}/avatar-0123456789abcdef.png ", false],
  ["perfiles/{A}/../x.png", false], ["perfiles/{A}/%2e%2e/avatar.png", false], ["perfiles/{B}/avatar.png", false], ["perfiles/{A}/other.png", false],
  ["perfiles/{A}/avatar-0123456789abcdef.png.", false], ["perfiles/{A}/avatar-0123456789abcdef.png\n", false], ["perfil/{A}/avatar.png", false],
];

export const IDS = {
  A: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
  B: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
  I: "1111111a-1111-4111-8111-11111111111a",
  J: "2222222b-2222-4222-8222-22222222222b",
};
export const fill = (tpl, extra = {}) => {
  const v = { ...IDS, A_UP: IDS.A.toUpperCase(), I_UP: IDS.I.toUpperCase(), K: "template", ...extra };
  return tpl.replace(/\{(\w+)\}/g, (m, k) => (k in v ? v[k] : m));
};
