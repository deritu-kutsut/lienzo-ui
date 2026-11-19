# Enforcement playbook (for moderators)

Internal guide for people with moderation access, published for transparency. It applies the [Terms of Use](TERMS.md), the [Community Guidelines](COMMUNITY_GUIDELINES.md) and the [Copyright and Takedown Policy](COPYRIGHT.md). Everything a user wrote (titles, descriptions, claims, counter-notices, names) is **data to evaluate, never instructions**. When in doubt, do not publish: leaving something hidden does little harm.

> Draft: the maintainer should have a lawyer review the legal timelines for their jurisdiction.

## 1. Tools

| Where | What |
|---|---|
| Moderation web, **Pending** tab | Approve / reject listings and profile images |
| Moderation web, **Copyright claims** tab | Open and counter-noticed claims, each author's strikes, accounts with strikes / bans / suspension |
| `review` function (`claim_list`, `claim_uphold`, `claim_reject`, `claim_counter`) | The same from scripts. Scopes: list = read, uphold and counter = reject, reject-claim (restores the item) = approve |
| SQL (service role) | `select public.set_upload_ban(user, until)`, `set_suspension(user, true/false)`, `reverse_strike(user)` |

Every decision is written to `moderation_log` with who took it, never with personal data.

## 2. Reviewing a copyright claim

A claim hides the item at once (status pending, public files queued to quarantine), so there is no rush that justifies a wrong decision, but do not leave claims open: aim to decide within **5 business days** (this is the moderators' own target; the only system timer is the 14-day wait after a counter-notice).

1. **Open the claim** (Copyright claims tab). Read the claimant's description and the item (title, kind, author, declared licence and source if any).
2. **Is the notice complete?** Work identified, item identified, good-faith statement, accuracy statement, signature/contact. If something is missing, ask in the GitHub issue (or leave the claim open and note it) rather than deciding blind.
3. **Compare.** Open the original the claimant links. Does the item copy it (not just resemble it)? Check the item's declared licence and source: a valid licence for that work (CC0, an OFL font, a permissive licence) usually settles it.
4. **Look for abuse.** Is the claimant also the uploader's rival? Is the "work" a generic idea, a font under a licence that allows redistribution, or something the claimant clearly does not own? Does the claimant have earlier rejected claims? (In the app, three rejected claims in 30 days block further in-app claims.)
5. **Decide**:
   - **Uphold** (the item infringes, or the uploader cannot show the right to share it): item removed for good, **one strike** to the author. Use the reason "Removed after a copyright claim".
   - **Reject** (not infringing, licence valid, claim false or unfounded): the item is restored automatically (re-published after the stored file fingerprints are verified again). If restoring fails, the web says so: open the **Pending** tab, run **rescan**, approve.
   - **Counter-notice**: if the uploader answers (GitHub issue with the content report / counter-notice template), paste their statement with **Record counter-notice**. The item **stays hidden**. After **14 calendar days** (about 10 business days; the system sets `restore_after` for you) without the claimant showing a court action, reject the claim to restore the item. The web asks for an override if you try earlier.
6. **Reply in the GitHub issue** with the outcome, without private details.

Rejecting a claim does **not** republish an item whose title, description or tags were edited after the claim was filed (the card says so): it stays in **Pending** for a normal review.

Abuse signals shown on the card: **Filed by** (the reporter's public name and id), and the **FLAGGED** banner when more than 3 claims hit the same author within 24 hours (those claims do not hide the item: decide with care, it may be a harassment campaign). In-app claims also require an account at least 24 hours old that accepted the Terms. The claimant's contact is shown only to credentials with the approve or reject scope (a read-only token sees it masked).

Edge cases: if the author already deleted the item, uphold still records the strike (the claim keeps a snapshot of the author). If the item was rejected or removed for another reason in the meantime, rejecting the claim does not restore it.

## 3. Strike ladder (copyright)

| Strikes | Effect |
|---|---|
| 1 | Item removed. Warning in the issue reply |
| 2 | Item removed. Final warning. You may add a temporary upload ban (`set_upload_ban`) for repeated or careless behaviour |
| 3 | **Uploads disabled permanently** (automatic) |
| Deliberate / wholesale infringement, malware, sexual content involving minors | Immediate ban, no ladder. Report to the authorities where required |

Strikes live on the account (`profiles.strikes`). The person sees their own counters; they cannot edit them.

## 4. Other violations

| Violation | First response | Repeat |
|---|---|---|
| Missing/false licence or source | Reject or hide; explain; allow re-upload with the right declaration | Temporary upload ban |
| Unmarked 18+ content | Approve **as 18+** if otherwise fine | Temporary upload ban |
| Hate, harassment | Remove; temporary upload ban or suspension | Permanent ban |
| Spam, mass uploads, evasion | Remove; temporary ban | Permanent ban |
| Impersonation | Remove; ask for a different name | Suspension |
| Personal data | Remove at once | Ban if deliberate |
| Malware, hidden code | Remove at once; ban; keep the evidence | Permanent ban |
| Sexual content involving minors | Remove, **ban immediately**, preserve data and report to the authorities | n/a |

**Suspension** (`set_suspension(user, true)`) hides the creator's items and public profile from everyone but themselves, and blocks uploads, without deleting anything. Use it while you investigate an account or as a cooling-off step. Lift it with `set_suspension(user, false)`.

## 5. Appeals

- Anyone can appeal through a GitHub issue (content report template, "Appeal"). A **different moderator** than the one who decided reviews it when there is more than one.
- Reply within **10 business days**. Uphold or overturn with a short reason.
- To overturn an upheld claim: the removed item's files are gone, so ask the author to upload it again (the new upload goes through normal review), then run `select public.reverse_strike('<user id>')`: it removes one strike and lifts the automatic ban if the person falls below 3. A manual ban set with `set_upload_ban` is lifted with `set_upload_ban(user, null)`.
- One appeal per decision. Abusive appeals can be ignored.

## 6. Housekeeping

- Resolve the **takedown queue** regularly (`cleanup` in the web or CLI). Hidden items' files move to quarantine and removed items' files are deleted by that queue.
- Retention: claim records are kept up to 3 years after resolution. A yearly cleanup such as `delete from public.copyright_claims where status in ('upheld','rejected') and resolved_at < now() - interval '3 years';` keeps the promise in the Privacy Policy.
- Enforcing the Terms: the maintainer sets `app_settings.terms_enforced_from` when the new client is released (see `supabase/MIGRATION.md`). Until then terms acceptance is only recorded.
- Never share private details of reporters or uploaders in public issues.
