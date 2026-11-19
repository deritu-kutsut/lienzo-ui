# Lienzo UI Community: Privacy Policy

**Version 1.0** · Last updated: 2026-11-19 · Part of the [Terms of Use](TERMS.md).

> **Draft, not legal advice.** The maintainer should have a lawyer review this policy for their jurisdiction (for example GDPR, UK GDPR, CCPA, children's privacy rules), and fill in the legal details that apply to them (data controller identity, representative, supervisory authority).

This policy covers the **online Community service**. The desktop app itself has **no telemetry, no analytics and no tracking**. It works fully offline. It contacts the Community service **only when you open the Community tab** (and only to the addresses of that service).

## 1. Who is responsible

The maintainer of Lienzo UI is the data controller for the Community service. Contact: open an issue at <https://github.com/deritu-kutsut/lienzo-ui/issues> (do not put personal data in a public issue; ask for a private channel and we will say how). There is no personal email address.

## 2. What we store, and why

| Data | When | Why | Visible to others? |
|---|---|---|---|
| **Account identifier** from GitHub or Google sign-in (an opaque id) | When you sign in | To recognise your account. **Your email is held by the sign-in system only; we do not copy it into our tables** and never show it. No password is ever stored. | No |
| **Public name** | When you create a profile | To sign what you share | Yes (after review) |
| **Optional profile data**: bio, GitHub username, social links, avatar, banner | If you add them | Your creator profile | Yes, once you have at least one approved item |
| **18+ settings**: your 18+ declaration and whether you show 18+ content | If you enable it | To show adult content only to adults who chose it | No |
| **Terms acceptance**: the version you accepted and when, plus an audit log of acceptances | When you accept | To show we asked you and what you agreed to | No |
| **Uploads**: the files, title, description, tags, licence, source link, rights confirmation, review status | When you share content | To run the catalog | Yes once approved |
| **Reports** you file (item, reason, short note) | When you report | To moderate | No |
| **Copyright claims** you file or that concern your content (claimant name and contact as entered, description, outcome) | When a claim is made | To handle the claim fairly and apply the repeat-infringer policy | The uploader is told the substance of a claim; claimant details are not shown publicly |
| **Hearts and follows** | When you use them | Totals are shown. **Who gave a heart or who follows whom is private**; only counts are public. | Counts only |
| **Download record** (which items you installed) | When you install | To count each download once | No |
| **Moderation records**: what was decided on an item and why, strikes, upload bans, suspensions | During moderation | To enforce the rules | No |
| **Technical records** of our hosting provider (such as request logs) | Always, while you use the service | Operation and security | No |

We do **not** collect: your location, device identifiers, contacts, browsing history, or advertising data. We do not sell personal data and there are no ads.

## 3. How we use it

To run the Community (sign-in, profiles, catalog, downloads), to moderate content, to enforce the Terms (strikes, bans), to answer copyright claims, to protect the service against abuse and security problems, and to meet legal obligations. If you are in a place with legal bases for processing (such as the EU/UK), we rely on **performing the agreement** with you (running your account and uploads), **our legitimate interest** in keeping the service safe and lawful, and **legal obligation** where it applies.

## 4. Who processes data for us

- **Hosting provider.** A managed platform provides the database, file storage, sign-in and server functions. It processes the data above on our behalf.
- **Sign-in providers.** GitHub or Google authenticate you. They have their own privacy policies; they tell us only that you signed in and give us an account id.
- **Automatic moderation and malware scanning.** To keep the catalog safe, uploaded files (for malware scanning) and text (titles, descriptions, tags, text inside designs, public names and bios, and images for adult-content checks) are sent to **third-party scanning and moderation services**. They receive the content, **not your email or account identifier**. They may keep a copy of scanned files or their fingerprints under their own terms.
- **Authorities.** We may disclose data if the law requires it, or to protect people from serious harm.

We do not share personal data with anyone else.

## 5. Where it is processed

Our providers may process data in several countries. Where the law requires it, we rely on appropriate safeguards for transfers outside your region. The maintainer should state here the region of the hosting project once it is fixed.

## 6. How long we keep it

| Data | Retention |
|---|---|
| Account, profile, uploads and files, hearts, follows, reports you filed, download records, terms acceptance log | Until you delete your account (they are deleted with it) |
| Upload counters used for rate limits | 7 days |
| Moderation log (decisions about items; no personal data) | Kept as a record of decisions; it holds no profile data and is detached from the item if the item is deleted |
| Copyright claim records, including claimant details as entered | Up to 3 years after the claim is resolved, to apply the repeat-infringer policy and defend against disputes. If you delete your account, claims you filed lose the link to your account; claims about your content keep only what is needed to apply the policy. |
| Strikes and bans | While your account exists. Deleting an account does not erase the reason it was banned from our moderation records |
| Hosting provider technical logs | The period set by the provider (typically days to weeks) |

Items that were rejected, withdrawn or deleted have their public files removed; files may stay in provider backups for a short time.

## 7. Your rights

Depending on where you live, you may have the right to **access** your data, **correct** it, **delete** it, **export** it, **restrict or object to** processing, and to complain to a data protection authority. In the app you can:

- **Delete your account**: one button; it deletes your account, profile, uploads and their files.
- **Export your data**: ask the app to export your data. It returns your own profile, uploads, reports, hearts, follows, download records, terms log and claims as a JSON file. It never includes other people's data.
- **Edit** your public name, bio and links at any time (a changed name or bio is reviewed again).
- **Withdraw your 18+ declaration** by turning the 18+ setting off.

For anything the app cannot do, open a GitHub issue. We will answer within a reasonable time (within one month where the law sets that period).

## 8. Children

The Service is **not for children under 13**. We do not knowingly collect data from them; if we learn that an account belongs to a child under 13 we will delete it. If the law where you live sets a higher age for online accounts or consent (some places use 14 or 16), you must meet that age. **Adult content is only available to accounts that declare they are 18 or older** and turn it on; the declaration is self-declared, and we do not verify age. Parents and guardians who think a child has an account can open a GitHub issue.

## 9. Security

Everything is closed by default: people can only read what is published and their own data. Uploaded files wait in a private quarantine area until checked. Sessions use industry-standard sign-in providers; we store no passwords. No system is perfect: to report a security problem use GitHub's private vulnerability reporting (the repository's Security tab) if enabled, or open an issue that says only that you have a security report, without details.

## 10. Cookies and tracking

The desktop app has no cookies. The moderation web page used by moderators keeps its session only in the browser tab until it is closed. There is no analytics and no advertising anywhere.

## 11. Changes

Material changes are published as a new version of this policy with a new date; where required we will ask you to accept the new terms again. Earlier versions stay in the repository history.
