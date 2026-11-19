#!/usr/bin/env bash
# Publishes a Lienzo UI version.
#   bash publicar.sh 1.1.0              publish
#   bash publicar.sh 1.1.0 --dry-run    do everything except tag, push, GitHub release and itch.io (builds, packages, checks)
#
# 1. Checks: VERSION equals the version in Cargo.toml, CHANGELOG.md has a non-empty "## [VERSION]" section, the working
#    tree is clean, there are no untracked files in designs/docs/plugins/mcp, and the branch is up to date on the remote
# 2. Runs the tests and builds in release (with path remapping and control-flow guard)
# 3. Checks the executable: it must not contain the build user name, C:\Users, the repository path or ".cargo"
#    (UTF-8 or UTF-16), and it reports control-flow guard / CET compatibility
# 4. Packages dist/LienzoUI-windows-<version>.zip FROM `git ls-files` (untracked files never ship) with forward-slash
#    entries and a THIRD_PARTY_LICENSES.txt (every dependency: name, version, license, repository, license texts; plus the fonts egui bundles)
# 5. Pushes the branch, creates (or reuses) the v<version> tag and the GitHub release with the notes from CHANGELOG.md (requires gh)
#    Every step can be re-run: an existing tag at the same commit or an existing release is reused (assets are replaced).
# 6. If butler is installed and signed in, uploads the package to itch.io (windows channel)
#
# Variables: REMOTE (git remote, default origin; the old REMOTO is still read), ITCH_TARGET (the old ITCH_DESTINO is still read),
#   DIST_DIR (output folder, default dist), CARGO_TARGET_DIR (as for cargo), SKIP_TESTS=1 (skip `cargo test`; never for a real release).
#
# CODE SIGNING IS NOT DONE HERE: the maintainer has to provide a code-signing certificate (an OV/EV certificate, or Azure
# Trusted Signing). Once there is one, sign the exe right after the build step (before the exe check and the packaging), e.g.
#   signtool sign /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 /f certificate.pfx /p "$PASSWORD" "$EXE"
# (or `/a` to pick the certificate installed in the user's store) and verify with `signtool verify /pa "$EXE"`.
# Never commit the certificate or its password.
set -euo pipefail
cd "$(dirname "$0")"

DRY=0
POSITIONAL=()
for a in "$@"; do
  case "$a" in
    --dry-run) DRY=1 ;;
    -*) echo "Unknown option: $a"; exit 2 ;;
    *) POSITIONAL+=("$a") ;;
  esac
done
VERSION="${POSITIONAL[0]:?Usage: bash publicar.sh <version> [--dry-run], e.g. 1.1.0}"
REMOTE="${REMOTE:-${REMOTO:-origin}}"
ITCH_TARGET="${ITCH_TARGET:-${ITCH_DESTINO:-deritu/lienzo-ui:windows}}"
DIST_DIR="${DIST_DIR:-dist}"
PACKAGE="${DIST_DIR}/LienzoUI-${VERSION}"
ZIP="${DIST_DIR}/LienzoUI-windows-${VERSION}.zip"
TARGET_DIR="${CARGO_TARGET_DIR:-target}"
EXE="${TARGET_DIR}/release/lienzo-ui.exe"
LICENSES="${DIST_DIR}/THIRD_PARTY_LICENSES.txt"

PYTHON="$(command -v python3 || command -v python || true)"
[ -n "$PYTHON" ] || { echo "Python 3 is needed (zip, license and executable checks)."; exit 1; }
py() { "$PYTHON" -I - "$@"; }
fail() { echo "ERROR: $*" >&2; exit 1; }
warn() { echo "warning: $*" >&2; }
# In a dry run a problem about the repository state is a warning (so the build can still be tested); in a real run it stops.
state_problem() { if [ "$DRY" = 1 ]; then warn "$* (would stop a real publish)"; else fail "$*"; fi; }

[ "$DRY" = 1 ] && echo "DRY RUN: nothing will be tagged, pushed, released or uploaded."

# ---------- 1. checks ----------
CARGO_VERSION="$(awk -F'"' '/^version[[:space:]]*=/ {print $2; exit}' Cargo.toml)"
[ "$CARGO_VERSION" = "$VERSION" ] || fail "VERSION ($VERSION) differs from the version in Cargo.toml ($CARGO_VERSION). Bump Cargo.toml first."

NOTES="$(awk -v v="$VERSION" 'index($0, "## [" v "]") == 1 {f=1; next} /^## \[/ {f=0} f' CHANGELOG.md)"
grep -q "^## \[$VERSION\]" CHANGELOG.md || fail "CHANGELOG.md has no \"## [$VERSION]\" section."
[ -n "$(printf '%s' "$NOTES" | tr -d '[:space:]')" ] || fail "The \"## [$VERSION]\" section of CHANGELOG.md is empty."

if ! { git diff --quiet && git diff --cached --quiet; }; then state_problem "There are uncommitted changes. Commit before publishing."; fi
UNTRACKED="$(git ls-files --others --exclude-standard -- designs docs plugins mcp assets README.md LICENSE CHANGELOG.md Cargo.toml Cargo.lock)"
[ -z "$UNTRACKED" ] || state_problem "Untracked files in packaged folders (git add them or delete them):
$UNTRACKED"

BRANCH="$(git rev-parse --abbrev-ref HEAD)"
[ "$BRANCH" != "HEAD" ] || state_problem "Detached HEAD: check out a branch before publishing."
HEAD_SHA="$(git rev-parse HEAD)"
if [ "$DRY" = 0 ]; then
  git fetch --quiet "$REMOTE" "$BRANCH" 2>/dev/null || true
fi
REMOTE_SHA="$(git rev-parse -q --verify "refs/remotes/$REMOTE/$BRANCH" || true)"
if [ "$REMOTE_SHA" != "$HEAD_SHA" ]; then
  if [ "$DRY" = 1 ]; then warn "branch $BRANCH is not pushed to $REMOTE yet (a real run pushes it before tagging)"
  elif [ -n "$REMOTE_SHA" ] && ! git merge-base --is-ancestor "$REMOTE_SHA" "$HEAD_SHA"; then fail "$REMOTE/$BRANCH has commits you do not have: pull or rebase first."
  fi
fi

if git rev-parse -q --verify "refs/tags/v$VERSION" >/dev/null; then
  TAG_SHA="$(git rev-list -n1 "v$VERSION")"
  [ "$TAG_SHA" = "$HEAD_SHA" ] || state_problem "Tag v$VERSION already exists and points to another commit ($TAG_SHA)."
fi

# ---------- 2. tests and release build ----------
[ "${SKIP_TESTS:-0}" = 1 ] && [ "$DRY" = 0 ] && fail "SKIP_TESTS is only allowed with --dry-run."
if [ "${SKIP_TESTS:-0}" != 1 ]; then cargo test --quiet; fi

# Keep build-machine paths (user name, project folder, cargo registry) out of the published exe, and turn on control-flow guard.
# CARGO_ENCODED_RUSTFLAGS (separator 0x1f) because the project path may contain spaces; it replaces any rustflags of .cargo/config.toml.
HOME_W="$(cygpath -w "$HOME" 2>/dev/null || echo "$HOME")"
PROJECT_W="$(cygpath -w "$PWD" 2>/dev/null || echo "$PWD")"
SEP=$'\x1f'
CARGO_ENCODED_RUSTFLAGS="--remap-path-prefix=${HOME_W}=/home/user${SEP}--remap-path-prefix=${HOME_W}\.cargo=/cargo${SEP}--remap-path-prefix=${HOME_W}\.rustup=/rustup${SEP}--remap-path-prefix=${PROJECT_W}=/app${SEP}-Ccontrol-flow-guard" cargo build --release --quiet
[ -f "$EXE" ] || fail "$EXE was not produced."
LIENZO_UI="$(cd "$(dirname "$EXE")" && pwd)/$(basename "$EXE")" node mcp/probar-mcp.mjs > /dev/null  # the MCP tests run against the exe just built

# ---------- 3. the executable must not leak the build machine ----------
BUILD_USER="${USERNAME:-${USER:-$(basename "$HOME")}}"
py "$EXE" "$BUILD_USER" "$(basename "$HOME")" 'C:\Users' 'C:/Users' "$HOME_W" "$PROJECT_W" "$PWD" '.cargo' <<'PY' || fail "The executable contains build-machine data (see above). Fix the remaps and rebuild."
import sys
exe, needles = sys.argv[1], sys.argv[2:]
data = open(exe, "rb").read()
low = data.lower()
bad = 0
seen = set()
for n in needles:
    if len(n) < 4 or n.lower() in seen:
        continue
    seen.add(n.lower())
    for label, enc in (("utf-8", "utf-8"), ("utf-16", "utf-16-le")):
        pat = n.lower().encode(enc)
        i = low.find(pat)
        if i >= 0:
            bad += 1
            ctx = data[max(0, i - 24): i + len(pat) + 24]
            shown = ctx.decode(enc, "replace") if enc == "utf-8" else ctx.decode("latin-1")
            shown = "".join(c if 32 <= ord(c) < 127 else "." for c in shown)
            print(f"  FOUND ({label}) {n!r} at offset {i}: ...{shown}...")
if bad:
    sys.exit(1)
print("  exe check: no user name, C:\\Users, repository path or .cargo inside the executable")
PY

# Windows hardening report: control-flow guard must be on; CET shadow-stack compatibility is reported.
py "$EXE" <<'PY' || fail "The executable is not built with control-flow guard."
import struct, sys
d = open(sys.argv[1], "rb").read()
pe = struct.unpack_from("<I", d, 0x3C)[0]
assert d[pe:pe + 4] == b"PE\0\0", "not a PE file"
nsec = struct.unpack_from("<H", d, pe + 6)[0]
optsize = struct.unpack_from("<H", d, pe + 20)[0]
opt = pe + 24
magic = struct.unpack_from("<H", d, opt)[0]
dllchar = struct.unpack_from("<H", d, opt + 70)[0]
cfg = bool(dllchar & 0x4000)
# CET compatibility lives in the debug directory (type 20 = IMAGE_DEBUG_TYPE_EX_DLLCHARACTERISTICS, bit 0 = CETCOMPAT)
dd_off = opt + (112 if magic == 0x20B else 96)
dbg_rva, dbg_size = struct.unpack_from("<II", d, dd_off + 6 * 8)
secs = []
s0 = opt + optsize
for i in range(nsec):
    vs, va, rs, rp = struct.unpack_from("<IIII", d, s0 + i * 40 + 8)
    secs.append((va, max(vs, rs), rp))
def rva2off(r):
    for va, sz, rp in secs:
        if va <= r < va + sz:
            return r - va + rp
    return None
cet = False
off = rva2off(dbg_rva) if dbg_rva else None
if off is not None:
    for k in range(dbg_size // 28):
        typ, sz, _, ptr = struct.unpack_from("<IIII", d, off + k * 28 + 12)
        if typ == 20 and ptr + 4 <= len(d):
            cet = bool(struct.unpack_from("<H", d, ptr)[0] & 1)
print(f"  hardening: control-flow guard {'ON' if cfg else 'OFF'}, CET shadow stack compatible {'ON' if cet else 'OFF'}, ASLR/DEP flags 0x{dllchar:04x}")
if not cet:
    print("  warning: /CETCOMPAT is not set in this build (build.rs asks for it with the MSVC linker)", file=sys.stderr)
sys.exit(0 if cfg else 1)
PY

# ---------- 4. third-party licenses ----------
mkdir -p "$DIST_DIR"
py "$LICENSES" <<'PY'
import hashlib, json, os, subprocess, sys
out = sys.argv[1]
meta = json.loads(subprocess.check_output(["cargo", "metadata", "--format-version", "1", "--locked", "--offline", "--filter-platform", "x86_64-pc-windows-msvc"], text=True, encoding="utf-8"))
root = meta["resolve"]["root"]
by_id = {p["id"]: p for p in meta["packages"]}
# only the crates that are in the dependency tree of the application (normal and build dependencies for Windows)
node = {n["id"]: n for n in meta["resolve"]["nodes"]}
seen, stack = set(), [root]
while stack:
    i = stack.pop()
    if i in seen:
        continue
    seen.add(i)
    for dep in node[i]["deps"]:
        if any(k.get("kind") in (None, "build") for k in dep["dep_kinds"]):
            stack.append(dep["pkg"])
pkgs = sorted((by_id[i] for i in seen if i != root), key=lambda p: (p["name"], p["version"]))
lines = ["Lienzo UI third-party software notices", "=" * 40, "",
         "Lienzo UI itself is released under the MIT license (see LICENSE).",
         "It is built with the open-source crates listed below. Each entry gives the crate, its version, its license",
         "expression and its repository; the license texts shipped by the crates follow the list.", ""]
texts = {}
for p in pkgs:
    lines.append(f"{p['name']} {p['version']} - {p.get('license') or p.get('license_file') or 'see repository'} - {p.get('repository') or p.get('homepage') or '(no repository listed)'}")
    d = os.path.dirname(p["manifest_path"])
    try:
        names = sorted(f for f in os.listdir(d) if f.upper().startswith(("LICENSE", "LICENCE", "COPYING", "UNLICENSE", "NOTICE")) and os.path.isfile(os.path.join(d, f)))
    except OSError:
        names = []
    for f in names:
        t = open(os.path.join(d, f), "rb").read().decode("utf-8", "replace").strip()
        if t:
            texts.setdefault(hashlib.sha256(t.encode()).hexdigest(), [t, []])[1].append(f"{p['name']} {p['version']} ({f})")
lines += ["", "", f"{len(pkgs)} crates.", ""]
# fonts bundled by egui (epaint_default_fonts)
fonts = next((p for p in pkgs if p["name"] == "epaint_default_fonts"), None)
lines += ["", "=" * 40, "Fonts bundled with the egui user-interface library (epaint_default_fonts)", "=" * 40, "",
          "Hack (MIT-style license with the Bitstream Vera license), Ubuntu-Light (Ubuntu Font Licence 1.0),",
          "Noto Emoji (SIL Open Font License 1.1), emoji-icon-font (MIT) and egui-icons.", ""]
if fonts:
    fd = os.path.join(os.path.dirname(fonts["manifest_path"]), "fonts")
    for f in ("Hack-Regular.txt", "UFL.txt", "OFL.txt", "emoji-icon-font-mit-license.txt", "egui-icons.txt"):
        fp = os.path.join(fd, f)
        if os.path.isfile(fp):
            lines += ["-" * 40, f, "-" * 40, open(fp, "rb").read().decode("utf-8", "replace").strip(), ""]
        else:
            print(f"  warning: font license file {f} not found", file=sys.stderr)
else:
    print("  warning: epaint_default_fonts is not in the dependency tree: font notices are missing", file=sys.stderr)
lines += ["", "=" * 40, "License texts shipped by the crates", "=" * 40, ""]
for t, users in sorted(texts.values(), key=lambda v: v[1][0]):
    lines += ["-" * 40, "Used by: " + ", ".join(users), "-" * 40, t, ""]
open(out, "w", encoding="utf-8", newline="\n").write("\n".join(lines) + "\n")
print(f"  {os.path.basename(out)}: {len(pkgs)} crates, {len(texts)} distinct license texts")
PY

# ---------- 5. package (from git ls-files only) ----------
rm -rf "$PACKAGE" "$ZIP"
mkdir -p "$PACKAGE"
py "$VERSION" "$EXE" "$LICENSES" "$PACKAGE" "$ZIP" <<'PY'
import json, os, shutil, subprocess, sys, zipfile
version, exe, licenses, package, zip_path = sys.argv[1:6]
tracked = subprocess.check_output(["git", "ls-files", "-z"]).decode("utf-8").split("\0")
tracked = [t for t in tracked if t]
def wanted(p):
    if p in ("README.md", "LICENSE", "CHANGELOG.md", "mcp/lienzo-ui-mcp.mjs"):
        return True
    if p.startswith(("docs/", "plugins/", "designs/pixel/")):
        return True
    return p.startswith("designs/") and p.count("/") == 1 and p.endswith(".editorgui.json")
files = sorted(p for p in tracked if wanted(p))
missing = [p for p in files if not os.path.isfile(p)]
if missing:
    sys.exit("tracked files missing from the working tree: " + ", ".join(missing))
for p in files:
    dest = os.path.join(package, p)
    os.makedirs(os.path.dirname(dest), exist_ok=True)
    shutil.copyfile(p, dest)
shutil.copyfile(exe, os.path.join(package, "lienzo-ui.exe"))
shutil.copyfile(licenses, os.path.join(package, "THIRD_PARTY_LICENSES.txt"))
# the MCP server reads its version from here when it runs outside the repository
os.makedirs(os.path.join(package, "mcp"), exist_ok=True)
with open(os.path.join(package, "mcp", "package.json"), "w", encoding="utf-8", newline="\n") as f:
    json.dump({"name": "lienzo-ui-mcp", "version": version, "private": True}, f, indent=2)
    f.write("\n")
top = os.path.basename(os.path.normpath(package))
count = 0
with zipfile.ZipFile(zip_path, "w", zipfile.ZIP_DEFLATED, compresslevel=9) as z:
    for dirpath, dirnames, filenames in os.walk(package):
        dirnames.sort()
        for name in sorted(filenames):
            full = os.path.join(dirpath, name)
            rel = os.path.relpath(full, os.path.dirname(os.path.normpath(package))).replace(os.sep, "/")  # forward slashes only
            assert "\\" not in rel and not rel.startswith("/")
            z.write(full, rel)
            count += 1
with zipfile.ZipFile(zip_path) as z:
    bad = [n for n in z.namelist() if "\\" in n or n.startswith("/") or ".." in n.split("/")]
    assert not bad, bad
    assert z.testzip() is None
print(f"  {count} files from git ls-files + exe, licenses and mcp/package.json")
PY
echo "Package: $ZIP ($(du -h "$ZIP" | cut -f1))"

if [ "$DRY" = 1 ]; then
  echo "DRY RUN finished: package and checks are fine. Nothing was tagged, pushed, released or uploaded."
  exit 0
fi

# ---------- 6. publish: branch, tag, release, itch.io ----------
command -v gh >/dev/null 2>&1 || fail "gh (GitHub CLI) is needed to create the release."
REPO_URL="$(git remote get-url "$REMOTE")"
git push "$REMOTE" "$BRANCH"
if ! git rev-parse -q --verify "refs/tags/v$VERSION" >/dev/null; then
  git tag -a "v$VERSION" -m "Lienzo UI $VERSION"
fi
REMOTE_TAG="$(git ls-remote --tags "$REMOTE" "refs/tags/v$VERSION^{}" "refs/tags/v$VERSION" | awk '{print $1}' | tail -n1)"
if [ -z "$REMOTE_TAG" ]; then
  git push "$REMOTE" "v$VERSION"
else
  git ls-remote --tags "$REMOTE" "refs/tags/v$VERSION^{}" | grep -q "$HEAD_SHA" || git ls-remote --tags "$REMOTE" "refs/tags/v$VERSION" | grep -q "$HEAD_SHA" || fail "The tag v$VERSION on $REMOTE points to another commit."
  echo "Tag v$VERSION is already on $REMOTE (same commit): reusing it."
fi
if gh release view "v$VERSION" --repo "$REPO_URL" >/dev/null 2>&1; then
  echo "Release v$VERSION already exists: replacing its package."
  gh release upload "v$VERSION" "$ZIP" --clobber --repo "$REPO_URL"
  gh release edit "v$VERSION" --repo "$REPO_URL" --title "Lienzo UI $VERSION" --notes "$NOTES"
else
  gh release create "v$VERSION" "$ZIP" --repo "$REPO_URL" --title "Lienzo UI $VERSION" --notes "$NOTES"
fi

if command -v butler >/dev/null 2>&1; then
  butler push "$PACKAGE" "$ITCH_TARGET" --userversion "$VERSION"
else
  echo "butler is not installed: update the itch.io download by hand (link to the GitHub release)."
fi
echo "Version $VERSION published."
