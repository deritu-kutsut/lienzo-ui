# Lienzo UI Community: security and setup

The community (plugins, images, fonts, templates and elements) uses **Supabase** as its server. This document explains how it is protected and how to set it up. The server code is in `supabase/`.

> Status: **server ready and its logic tested; the Supabase project still has to be created and the app's Community tab connected.** Everything that can be added externally (the `plugins/` folder, opening images and designs by hand) keeps working without an account or internet.

## What data is stored about each person

| Data | Where | What for |
|---|---|---|
| Account identifier and email | `auth.users` (managed by Supabase) | Signing in. **The email is not copied to any table of ours nor shown to anyone.** |
| Public name (3-32 characters) | `profiles` | Signing what you upload |
| 18+ preference | `profiles` | Showing or hiding content for people over 18 |
| Content you upload and its status | `community_items` | Catalog |

**There are no passwords**: access is only through external providers (GitHub, Google…). No IP, device or analytics is stored. Your account and everything of yours is deleted with one button (the `delete-account` function).

## Defenses, layer by layer

1. **Closed by default (RLS).** No table is readable or editable without an explicit policy. A user only sees what is approved and what is theirs; they cannot approve, change paths or remove the 18+ flag (column permissions + policies, `migrations/0001_comunidad.sql`; the tables were renamed to English in `0010_english_names.sql`).
2. **Quarantine.** Every file goes into a private bucket (`cuarentena`, "quarantine") in the user's own folder. It is not public until approved; only the server moves it to the `publico` ("public") bucket. The bucket names stay in Spanish on purpose: they are part of the URLs that older app versions call.
3. **Data formats only.** PNG, JPEG, GIF, WebP, TTF, OTF and JSON. The format is identified by the **actual bytes**, not by the extension: an `.exe` renamed to `.png` is rejected. Executables, installers, system scripts, archives and SVG are forbidden.
4. **Limits.** 2 MB images, 5 MB fonts, 512 KB templates, 256 KB plugins and elements; 4096×4096 px; 10 pending uploads and 20 per day per person.
5. **Plugins without binaries.** A plugin is uploaded as a single JSON with Rhai scripts as text. Paths with `..` or absolute paths, files that are not `.rhai`/`.json`, names that differ only in capitals, `eval`, `import`, links or process calls are rejected, and an exporter may only write `txt`, `csv`, `json`, `xml`, `md`, `cs`, `uxml`, `uss` or `tsv` files. Every `.json` that is a template or looks like a design is validated as a design. In addition, Rhai runs in a sandbox in the app (no files, network or processes; limits on operations, time, memory and printed output), and what a plugin command returns is checked again before it is used.
6. **Antivirus.** The file is checked automatically. A plugin with no track record goes to human review.
7. **Text moderation.** The listing text (title, description, tags and the design's texts) is reviewed automatically. **Your name, email or identifier are not used.**
8. **18+ filter.** The author sets the flag; automatic moderation can raise the flag, never lower it. By default it is **off**: 18+ content is not shown to anyone who has not enabled it in their profile.
9. **Reports.** Anyone can report. With 3 distinct reports the content is hidden until a person reviews it.
10. **Verified downloads.** The app checks the SHA-256 of the downloaded file before installing it, and validates it again with the same rules.
11. **Moderation log.** It records what was decided and why (without personal data). Only the server reads it.

## Honest limits

- **Names and titles** cannot contain control characters or invisible/direction-changing characters (zero-width spaces, left-to-right and right-to-left marks, BOM...), so a listing cannot pretend to be another one.

- **Images:** they go through automatic review and, if in doubt, through a person.
- **No antivirus is infallible.** That is why the content is only data and plugins run isolated; the antivirus is one more layer, not the only one.
- Someone has to review the "pending" queue. Without moderators, anything doubtful simply is not published.

## Setup (step by step)

1. Create a project at [supabase.com](https://supabase.com) (a region close to your users).
2. Install the CLI (`npm i -g supabase`) and link it: `supabase link --project-ref <ref>`.
3. Apply the schema: `supabase db push`.
4. Under **Authentication → Providers**, enable **GitHub** and **Google** with each one's credentials (each provider gives you a *client id* and a *secret*; the callback URL is the one Supabase shows). For **itch.io** and **Unity**, which are not included, configure them as an OIDC/custom provider if the service allows it; they are added later.
5. Store the secrets **in Supabase, never in the repository**:
   ```bash
   # server secrets are stored in Supabase, never in the repository
   ```
6. Deploy the functions: `supabase functions deploy moderate`, `supabase functions deploy review`, `supabase functions deploy profile` and `supabase functions deploy delete-account`.
7. Under **Authentication → URL Configuration → Redirect URLs**, add `http://127.0.0.1:53682/callback` (the app receives the sign-in on that temporary local port).
8. The app only needs two **public** values: the project URL and the publishable (`anon`) key, in `src/community.rs`. With RLS the `anon` key gives no access to anything that is not allowed. **The `service_role` key never goes in the app or in the repository.**

## Tests

```bash
node --test supabase/functions/_shared/validate.test.ts supabase/functions/_shared/decide.test.ts
```

## App notes

- **There is no connection until you open the Community tab.**
- The session token is stored in the Windows Credential Manager (not in a file). On other systems the session is not remembered between runs.
- Every download is verified against the approved SHA-256 and validated again locally before being installed (`src/community_validate.rs`, same rules as the server).
- The base table permissions are in `migrations/0004_permisos_base.sql`: the project was created with "automatically expose tables" turned off, so **every new table needs its explicit `grant` in addition to its RLS policy**.

## Command line

They use the session saved by the app (the same one as the tab's button). **Close the app first**: the session token rotates and the app, left open with the old token, would lose the session.

```bash
lienzo-ui --community-upload <plugin|image|font|template|element|theme> <file|folder> --title "…" [--description "…"] [--labels a,b] [--18]
lienzo-ui --community-retry          # re-moderates your listings that were left pending without a decision
```

A listing can be left **pending** with a reason (for example, an image while there is no image moderator, or a plugin the antivirus does not know): it waits for a person to review it. Rejected ones show the reason and their file is deleted.

> The Edge Functions use the service key, which also needs its `grant`s (`migrations/0005_permisos_servidor.sql`). After changing `validate.ts` or `decide.ts`, `moderate` has to be deployed again.

## Human review (moderators)

Whatever stays **pending** is reviewed by a person. Only accounts whose identifier is in the server's `MODERATORS` secret (comma-separated; the old name `MODERADORES` is still read) can do it. Your identifier is shown by `lienzo-ui --community-me`; it is configured with `supabase secrets set MODERATORS=<id>[,<id>…]`.

```bash
lienzo-ui --community-review list                         # pending items and profile images (text from the server is printed without control characters)
lienzo-ui --community-review approve <id> [--18]          # publishes it (re-validated and fingerprint checked)
lienzo-ui --community-review reject <id> "reason"         # rejects it and deletes the file
```

The `review` function (`supabase/functions/review/`) checks that the caller is a moderator. **The person who approves should not be the author of the content.**

## Icons, previews and creator profiles

- **Icon and preview** (optional, for any content type): PNG up to 512×512 / 256 KB and 1920×1080 / 1 MB. They go through the same moderation: while there is no image moderator, content that carries them waits for human review. On the command line: `--icon icon.png --preview preview.png`.
- **Creator profile** (`public_profiles`): profile photo (PNG ≤ 512×512), banner (PNG ≤ 1920×640), description (≤ 300, no links, reviewed automatically), GitHub username and links to X, YouTube, itch.io, Twitch, Bluesky and Instagram (only `https://` on those domains). It can only be edited through the `profile` function; the table cannot be written from the app. Your own profile, including the pending-review flags, is read from the view `my_public_profile` (only your row); the table itself exposes only the public columns to other people (migration 0014, applied after the 1.5.x release; see `supabase/MIGRATION.md`). A new public name is reviewed by the same text moderation when the profile is created (an empty `save`), saved or first used in an upload; until it is reviewed it is not shown publicly.
- A profile is **public only if the person has at least one approved piece of content**; your own is always visible. It never includes the email or preferences.
- Moderators: `--community-review list` also shows pending profile images; they are approved with `approve-profile <account> avatar|banner` (the `review` function).

## 18+ filter

**The server** decides: an anonymous visitor never receives 18+ content, and with a session it is only received if the account declared being over 18 (`profiles.declared_adult_at`, required to enable `show_18`).

## Following creators and hearts

- **Heart** (the community's "star"): given to approved content; each creator accumulates the ones they receive. **Follow**: on a creator's profile. The **♥ Following** filter shows only the content of the people you follow.
- **Privacy:** who follows whom and who gave each heart **is not public**: each person only sees their own (RLS) and everyone else sees only the totals, which only server triggers change.
- **Limits:** 200 hearts per day and 1000 followed creators per person; you cannot follow yourself.
- **App header:** a button with your profile photo (a default image if you have none) opens your creator profile; without a session, it opens the Community tab. In a preview, one click enlarges it.
- Command line: `--community-heart <content-id>` and `--community-follow <creator-id>` (they toggle).

## Themes

- **Themes** button in the header: five built-in themes (Dark canvas, Light canvas, Midnight, Sunset, Forest), colors editable one by one, corners, light or dark base and **interface font** (any system font or one from your library; community ones are installed from Community › Fonts).
- **Save as theme** puts it in your library (the `temas` folder of the library); **Share in the community** opens the upload form with the theme ready. In Community › Themes, **Install** saves it and puts it in use.
- A theme is a small JSON (≤ 32 KB) that only accepts `#RRGGBB` colors, corners 0-16, light base and the **name** of a font: never files, links or code. Both the server and the app check it.
- For screenshots or tests: `lienzo-ui --theme "Lienzo claro"` (that session only).

## My creator profile

An X-style window: banner with the photo overlaid, camera buttons to change them, labeled fields (name, bio, GitHub and social links) and **Save** at the top. Any image (PNG, JPEG, WebP, GIF or BMP) is cropped and reduced automatically (photo 400×400, banner 1500×500). While a person reviews it, you already see it on your profile and on the header button; everyone else sees it once it is approved.

## Automatic moderation

Every content upload and every profile photo or banner goes through an automatic check in the `moderate` and `profile` functions:

- **Format:** data formats only, checked by their actual content (not by the extension).
- **Antivirus:** all files. Many detections = rejection; few = review by a person.
- **Text:** automatic moderation of the listing.

**Decision rules** (`decide.ts`, with tests): clean content is published on its own; **when in doubt or if a service is down, it is not published** and stays pending for a person. Everything is recorded in `moderation_log` (step, result and reason, without personal data). `lienzo-ui --community-review rescan [id]` re-runs the oldest pending item. `lienzo-ui --community-review cleanup` drains the takedown queue (files of rejected / deleted listings).

Server secrets are stored **in Supabase, never in the repository** (`supabase secrets set ...`).

> **Privacy:** only upload content meant to be public.
