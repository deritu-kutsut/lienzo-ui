// Anti-framing. `frame-ancestors` cannot be set from a <meta> CSP, so this script does the job: the page stays hidden
// (see the `html:not(.top-level)` rule in moderation.css) unless it is the top-level document. If it is framed, it tries to
// break out and, failing that, it stays blank. It is an external file because the CSP forbids inline scripts.
(function () {
  var top = null;
  try { top = window.top; } catch (e) { top = null; }
  if (top === window.self) {
    document.documentElement.classList.add("top-level");
    return;
  }
  try { top.location = window.self.location; } catch (e) { /* cross-origin parent: it stays hidden */ }
})();
