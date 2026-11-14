// Maths: pymdownx.arithmatex (generic mode) marks each formula with the class `arithmatex`,
// as \(...\) inline or \[...\] displayed; KaTeX (loaded before this, pinned in zensical.toml)
// renders those elements only, so a dollar sign elsewhere on a page stays text. Re-run on each
// instant navigation (document$).
(function () {
  function render() {
    if (!window.katex) return;
    document.querySelectorAll(".arithmatex").forEach(function (el) {
      if (el.dataset.katex) return;
      var tex = el.textContent.trim();
      var display = tex.slice(0, 2) === "\\[";
      tex = tex.replace(/^\\[[(]/, "").replace(/\\[\])]$/, "");
      window.katex.render(tex, el, { displayMode: display, throwOnError: false });
      el.dataset.katex = "1";
    });
  }
  if (window.document$ && typeof window.document$.subscribe === "function") window.document$.subscribe(render);
  else document.addEventListener("DOMContentLoaded", render);
})();
