# Client contract for migration 0015 (terms, declarations, claims, export)

For whoever wires the desktop client. Everything here is **v2 / English names**. Old clients (<= 1.5) never call any of it and keep working (they are only refused with `upload_banned` / `account_suspended` / `terms_required` when the server flags say so).

Base: `<SUPABASE_URL>` and the publishable key. Every call sends `apikey: <publishable key>` and `Authorization: Bearer <user JWT>` (anon calls may send the publishable key as bearer). `Content-Type: application/json`.

## How to read an error

PostgREST answers `{"code":"<sqlstate>","details":null,"hint":"<machine code>","message":"<machine code>: <readable text>"}`. **Match on `hint`** (fallback: the `message` prefix before the first `: `), never on the HTTP status or SQLSTATE. Edge functions answer `{"v":2,"error":"<text>","code":"<machine code>"}`.

| code (`hint`) | Where | Meaning | Client should |
|---|---|---|---|
| `terms_required` | insert community_items | Enforcement is on and the account has not accepted the CURRENT version | Show the terms screen, call `accept_terms`, retry the upload |
| `upload_banned` | insert community_items | Strikes or a ban. The message says until when (`...until 2026-12-01 10:00 UTC.`) or `disabled for this account.` (permanent) | Show the message; hide the upload button (see "Own moderation state") |
| `account_suspended` | insert community_items, `file_copyright_claim` | Account suspended | Show the message |
| `license_invalid` | insert community_items | Licence not in the allowlist, or missing when the server requires one | Highlight the licence picker |
| `source_url_invalid` | insert community_items | Not http(s), has user:pass@, spaces, or > 300 characters | Highlight the source field |
| `rights_required` | insert community_items | A licence was sent without `rights_confirmed_at` (or it is required) | The "I own it / may share it" checkbox must be ticked |
| `terms_unknown_version` | `accept_terms` | Version not known by the server | Re-read `current_terms()` |
| `terms_outdated` | `accept_terms` | Version known but no longer current | Re-read `current_terms()`, re-show the new text |
| `profile_required` | `accept_terms`, `file_copyright_claim` | No profile yet | Create the profile first |
| `sign_in_required` | RPCs | No user JWT | Sign in |
| `claim_invalid` | `file_copyright_claim` | Description not 10-2000 chars, claimant > 200, contact > 200, or claiming your own item | Fix the form (the text says which) |
| `profile_too_new` | `file_copyright_claim` | Profile younger than 24 h | Say so; point to the GitHub copyright template |
| `terms_required` | `file_copyright_claim` (also uploads once enforced) | The account never accepted the terms | Show the terms, call `accept_terms` |
| `claim_rate_limited` | `file_copyright_claim` | 5 claims in 24 h | Say so; point to the GitHub copyright template |
| `claims_blocked` | `file_copyright_claim` | 3 rejected claims in 30 days | Point to the GitHub copyright template |
| `item_not_found` | `file_copyright_claim` | Item is not public (already hidden, pending, deleted) | Say "already under review" |

Note the daily upload limits (`Daily upload limit reached (20).`, `Too many uploads pending review (maximum 10).`) are plain errors with no code, as before.

## 1. Which terms version is current

```
POST /rest/v1/rpc/current_terms          body {}        (anon or signed in)
200 {"version":"1.0","published_at":"2026-11-19T00:00:00+00:00","summary":"First version: ..."}
```
The TEXTS are not served by the API: embed `assets/legal/*.txt` (short versions, each starts with its version) and link the full docs in the repository (`docs/TERMS.md`, `docs/COMMUNITY_GUIDELINES.md`, `docs/PRIVACY.md`, `docs/COPYRIGHT.md`). Bump the embedded texts whenever the maintainer publishes a new version row.

Does this account need to accept? Read your own profile:
```
GET /rest/v1/profiles?id=eq.<uid>&select=terms_version,terms_accepted_at,strikes,upload_banned_until,suspended
200 [{"terms_version":null,"terms_accepted_at":"2026-05-01T10:00:00+00:00","strikes":0,"upload_banned_until":null,"suspended":false}]
```
`needs_accept = profile.terms_version !== current.version`. A non-null `terms_accepted_at` with a null `terms_version` is a LEGACY acceptance ("v0"): ask again (the first time it only matters once the maintainer turns enforcement on, but ask anyway: it is cheap and the version is recorded).

## 2. Accept the terms

Either:
```
POST /rest/v1/rpc/accept_terms           body {"p_version":"1.0"}          (signed in)
200 {"version":"1.0","accepted_at":"2026-11-19T10:00:00+00:00","current":"1.0"}
```
or through the edge function (same effect; v2 only):
```
POST /functions/v1/profile               body {"v":2,"action":"accept_terms","version":"1.0"}
200 {"v":2,"status":"saved","version":"1.0","accepted_at":"...","current":"1.0"}
```
Call it right AFTER creating the profile (the profile must exist) and again whenever `needs_accept`. It is idempotent. The client must not write `terms_version` directly (it is ignored). You may keep sending the legacy `terms_accepted_at` on profile creation if you want old and new behaviour to agree; it is not needed.

## 3. Upload fields

Same insert as today (table `community_items`), with three extra optional columns:

```
POST /rest/v1/community_items
{ "id":"<uuid>", "author":"<uid>", "kind":"template", "title":"Neon HUD", "description":"...", "tags":["hud"],
  "quarantine_path":"<uid>/<id>/design.json", "...": "(everything you send today)",
  "license":"cc-by", "source_url":"https://github.com/me/neon-hud", "rights_confirmed_at":"2026-11-19T10:00:00Z" }
```
- `license` is one of: `cc0`, `public-domain`, `cc-by`, `cc-by-sa`, `cc-by-nc`, `mit`, `apache-2`, `ofl`, `proprietary-free-use`. (Show human labels; fonts default to `ofl`, plugins to `mit`.) "All rights reserved" does not exist on purpose.
- `source_url`: optional, `http(s)://host/...`, at most 300 characters, no `user:pass@`.
- `rights_confirmed_at`: send any timestamp when the user ticked "I own this or have the right to share it under this licence". **The server overwrites it with its own time.** Send `license` only together with it.
- Until the maintainer sets `license_required_from`, these are optional (null is accepted). Send them anyway from the new client.
- Author cannot edit them later (they are not in the update grant): to change a licence the user re-uploads.
- Rows read back include `license`, `source_url` (public, show them on the item page; render `source_url` as plain text, open only on click and only http/https).

## 4. File a copyright claim (Report > Copyright)

```
POST /rest/v1/rpc/file_copyright_claim
{ "p_item":"<item uuid>", "p_description":"<10-2000 chars: the work and why it is infringed>", "p_claimant":"<name of the rights holder, <= 200; omit to use the public name>", "p_contact":"<optional, <= 200>" }
200 {"claim":"<uuid>","status":"open","hidden":true}
```
Requires a profile at least 24 h old that accepted the terms. `hidden` is `false` when the author already has more than 3 open claims in 24 h: the claim is recorded and flagged for a moderator but the item stays public (show the same thanks message). The item is HIDDEN at once (it disappears from catalog queries; the claimant also loses sight of it: refresh the list). Show "Thanks: the item is hidden while a moderator reviews it. Details: docs/COPYRIGHT.md". Errors: `claim_invalid`, `claim_rate_limited`, `claims_blocked`, `item_not_found`, `account_suspended`, `profile_required`, `profile_too_new`, `terms_required`.

## 5. Report with a reason

```
POST /rest/v1/reports
{ "item":"<uuid>", "reporter":"<uid>", "reason":"hate", "detail":"<= 500 chars, optional>" }
```
`reason` is one of: `copyright`, `adult`, `malware`, `hate`, `spam`, `impersonation`, `personal_data`, `other`. (`adult_content` and `hate_or_harassment` are the old spellings: still valid, do not send them from the new client.) One report per person per item (a second one is a unique-violation error). For **copyright**, prefer `file_copyright_claim`: it also hides the item immediately; a plain `copyright` report only counts towards the 3-reports auto-hide.

## 6. Export my data

```
POST /rest/v1/rpc/export_my_data        body {}        (signed in)
200 { "exported_at":"...", "profile":{...}, "public_profile":{...}, "items":[...], "reports_filed":[...], "hearts":[...], "follows":[...],
      "downloads":[...], "terms_log":[{"id":1,"version":"1.0","accepted_at":"..."}], "copyright_claims_filed":[...], "copyright_claims_against_my_items":[...] }
```
Save the whole JSON as a file the user chooses (e.g. `lienzo-my-data.json`). Own rows only. Items are capped at 2000.

## 7. Own moderation state (to hide the Upload button, show a banner)

From the profile read above: `suspended === true` → "account suspended"; `upload_banned_until` in the future → "uploads disabled until ..." ; a value starting with `9999-` means **permanent**; `strikes` 0-3 (3 means banned). These are read-only for the user.

## 8. What stays the same

- Browsing, installing, hearts, follows, profile edits, delete account: unchanged.
- Old clients: still insert through the Spanish views and `perfiles.acepto_terminos_en`; nothing is refused unless a ban/suspension applies, or the maintainer turned `terms_enforced_from` on.
- Client-side order for first sign-in: sign in → create profile → `current_terms` → show terms → `accept_terms` → continue.

## 9. Things the client side still has to build

1. Terms/guidelines/privacy/copyright screen with the embedded short texts and the Accept button (at account creation and when `needs_accept`).
2. Licence picker + source field + "I own this or may share it" checkbox in the upload dialog, wired to the new columns, with the error highlights above.
3. Report dialog with the eight reasons, and the **Copyright** flow that calls `file_copyright_claim`.
4. Item page: show licence and source link.
5. Banner for strikes / ban / suspension; hide Upload when banned or suspended.
6. "Export my data" button in settings.
7. Handling of the codes in the table above.
