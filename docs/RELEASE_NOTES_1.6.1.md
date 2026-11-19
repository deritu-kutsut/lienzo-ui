# Lienzo UI 1.6.1

Stability and security fixes found in a full audit of 1.6.0.

## Fixed
- **Saving and recovery:** Save only writes to the file the design is bound to; Save As is offered otherwise. Closing, opening and creating with unsaved changes now ask first. Autosave retries and warns when it cannot write; recovery files record the owning process so a second window never restores another window's work; 64 MB recovery files are handled.
- **Scripts:** deeply nested data returned by a script can no longer crash the app (scripts run on a large stack; results deeper than 64 levels are rejected). `on_click` of downloaded content is validated.
- **Review:** «Fix all» works in batches (up to 3000 fixes per click) and keeps the original name on the first duplicate.
- **Community:** an invalid session token is reported as an expired session; a request with `?error=` no longer cancels the sign-in; the busy state is always released.
- **Images:** decoding is limited to 8192 px, three images per pass, a 256 MB cache and 192 px thumbnails.
- **MCP:** designs without `next_id` are no longer corrupted and added elements never copy another element's `unity_name`.
- **Command line:** wrong usage exits with code 2 and prints to the console when started from a terminal.
- Files are written atomically with retries (antivirus / OneDrive locks); the built-in pack never overwrites a different file of the same name.

## Security
- Plugins are no longer loaded from the working directory; only the ones next to the executable and in the data folder.
- Manual JSON export no longer writes absolute paths; the reserved name `COM¹` is rejected.
- Server (migration 0016): profile dates and terms acceptance are stamped by the server; reports need an aged account and are limited to 10 per day; three young accounts can no longer hide an item; suspending an account hides its items; deleting an account no longer clears strikes or bans (hashed identity only, see [docs/PRIVACY.md](docs/PRIVACY.md)); copyright claims require accepted terms.

