#!/usr/bin/env python3
"""Open pages of the built site in Chromium, screenshot them, and check what only a browser shows.

    uv run --no-project -p 3.13 --with playwright==1.56.0 python docs/scripts/look_pages.py \
        [--changed BASE | --blog | PAGE ...] [--base-url URL] [--out DIR]

Pages are paths in the site (`blog/2026-11-04-autonomous-agent-team/`). `--changed BASE`
takes the pages whose sources differ from git revision BASE (every blog post, if the
blog's stylesheet or script changed); `--blog` takes every post. Without `--base-url`
it serves site/ itself, so it looks at the build in hand, not a deployed copy.

Each page is opened at a desktop and a phone width, in the light and the dark theme. Every
section (from one H2 to the next) is saved as its own screenshot under DIR (default
site-shots/), with a report.md listing them and the problems found:

- a console error or uncaught exception, and a request that failed or returned an error;
- an image that did not load;
- the page scrolling sideways, or an element reaching past the content column outside
  a scrolling block (code, a wide table);
- an SVG label drawn outside its figure (the blog's SVGs have overflow: visible, so a
  label past the viewBox still shows, over whatever is next to it);
- maths KaTeX rejected, or left unrendered.

Exits with 1 if any problem was found. The screenshots are for a person to read.
"""

import argparse
import functools
import http.server
import re
import subprocess
import sys
import threading
from pathlib import Path

from playwright.sync_api import sync_playwright

ROOT = Path(__file__).resolve().parents[2]
DOCS = ROOT / "docs" / "site"
VIEWPORTS = {"desktop": (1280, 900), "phone": (390, 844)}
THEMES = ("light", "dark")
SLICE = 1600  # the tallest screenshot, in CSS pixels, so that text stays legible

# Runs in the page: returns a list of problems, each a short string.
CHECKS = r"""
() => {
  const problems = [];
  const doc = document.documentElement;
  if (doc.scrollWidth > doc.clientWidth + 1)
    problems.push(`the page scrolls sideways: ${doc.scrollWidth}px wide in a ${doc.clientWidth}px window`);
  const article = document.querySelector('article');
  if (!article) return ['no <article> on the page'];
  const column = article.getBoundingClientRect();
  const clipped = new Map();  // element -> whether it or an ancestor clips or scrolls sideways
  const scrolls = el => {
    const e = el.parentElement;
    if (!e || e === article) return false;
    if (!clipped.has(e)) {
      const o = getComputedStyle(e).overflowX;
      clipped.set(e, o === 'auto' || o === 'scroll' || o === 'hidden' || scrolls(e));
    }
    return clipped.get(e);
  };
  const name = el => el.tagName.toLowerCase() + (el.id ? '#' + el.id : '') +
    ' "' + (el.textContent || '').trim().replace(/\s+/g, ' ').slice(0, 50) + '"';
  const blocks = 'p, li, h1, h2, h3, h4, h5, h6, figure, table, img, svg, pre, blockquote, details, .admonition, div';
  for (const el of article.querySelectorAll(blocks)) {
    if (el.parentElement.closest('svg')) continue;
    const r = el.getBoundingClientRect();
    if (r.width === 0) continue;
    if (el.matches('pre, .highlight')) continue;  // code blocks go full-bleed on a phone, by design
    const own = getComputedStyle(el).overflowX;  // as may a scrolling table
    if ((r.right > column.right + 2 || r.left < column.left - 2) && own !== 'auto' && own !== 'scroll' && !scrolls(el))
      problems.push(`${name(el)} reaches past the content column (${Math.round(r.left)}–${Math.round(r.right)} against ${Math.round(column.left)}–${Math.round(column.right)})`);
  }
  for (const img of article.querySelectorAll('img'))
    if (img.complete && img.naturalWidth === 0) problems.push(`image did not load: ${img.getAttribute('src')}`);
  for (const svg of article.querySelectorAll('svg')) {
    const box = (svg.closest('figure') || svg).getBoundingClientRect();
    if (box.width === 0) continue;
    for (const t of svg.querySelectorAll('text')) {
      const r = t.getBoundingClientRect();
      if (r.width === 0) continue;
      if (r.left < box.left - 1 || r.right > box.right + 1 || r.top < box.top - 1 || r.bottom > box.bottom + 1)
        problems.push(`SVG label outside its figure: "${t.textContent.trim().slice(0, 50)}"`);
    }
  }
  for (const e of article.querySelectorAll('.katex-error'))
    problems.push(`KaTeX rejected: ${e.getAttribute('title') || e.textContent.slice(0, 60)}`);
  for (const e of article.querySelectorAll('.arithmatex'))
    if (!e.querySelector('.katex')) problems.push(`maths left unrendered: ${e.textContent.trim().slice(0, 60)}`);
  return [...new Set(problems)];
}
"""

# Runs in the page: the top of each H2 section, in document coordinates, with its anchor.
SECTIONS = r"""
() => [...document.querySelectorAll('article h2')].map(h => ({
  top: Math.floor(h.getBoundingClientRect().top + window.scrollY) - 16,
  id: h.id || 'section',
}))
"""


def changed_pages(base):
    """The site paths of the pages whose sources differ from git revision base."""
    names = subprocess.run(
        ["git", "diff", "--name-only", f"{base}...HEAD", "--", "docs/site", "zensical.toml"],
        cwd=ROOT, capture_output=True, text=True, check=True,
    ).stdout.split()
    pages = set()
    for name in names:
        path = ROOT / name
        if name.endswith(".md") and path.exists() and path.is_relative_to(DOCS):
            pages.add(page_path(path))
        elif re.search(r"assets/(stylesheets|javascripts)/blog\.|zensical\.toml", name):
            pages.update(blog_pages())
    return sorted(pages)


def blog_pages():
    return [page_path(p) for p in sorted((DOCS / "blog").glob("*.md"))]


def page_path(source):
    rel = source.relative_to(DOCS).with_suffix("")
    if rel.name == "index":
        rel = rel.parent
    return "" if str(rel) == "." else f"{rel}/"


def serve(site):
    class Quiet(http.server.SimpleHTTPRequestHandler):
        def log_message(self, *args):
            pass

    handler = functools.partial(Quiet, directory=str(site))
    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), handler)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    return server, f"http://127.0.0.1:{server.server_address[1]}/"


def look(browser, url, out, viewport, theme):
    width, height = VIEWPORTS[viewport]
    context = browser.new_context(
        viewport={"width": width, "height": height}, color_scheme=theme, device_scale_factor=1
    )
    page = context.new_page()
    problems = []
    # A failed load is reported once, from its response; the theme's repository widget
    # calls api.github.com, which rate-limits and is not the page. A rate-limited call can
    # also come back without CORS headers, so it shows only as a console error and a failed
    # request: those are not the page either.
    page.on("console", lambda m: m.type == "error" and not m.text.startswith("Failed to load resource")
            and "api.github.com" not in m.text
            and problems.append(f"console error: {m.text[:200]}"))
    page.on("pageerror", lambda e: problems.append(f"uncaught exception: {str(e)[:200]}"))
    page.on("requestfailed", lambda r: "api.github.com" not in r.url
            and problems.append(f"request failed: {r.url} ({r.failure})"))
    page.on("response", lambda r: r.status >= 400 and "api.github.com" not in r.url
            and problems.append(f"HTTP {r.status}: {r.url}"))
    page.goto(url, wait_until="networkidle")
    page.evaluate("document.fonts.ready")
    problems += page.evaluate(CHECKS)
    full = page.evaluate("document.documentElement.scrollHeight")
    cuts = [0] + [s["top"] for s in page.evaluate(SECTIONS) if s["top"] > 0] + [full]
    names = ["top"] + [s["id"] for s in page.evaluate(SECTIONS) if s["top"] > 0]
    out.mkdir(parents=True, exist_ok=True)
    shots = []
    for i, (start, end) in enumerate(zip(cuts, cuts[1:])):
        for j, y in enumerate(range(start, end, SLICE)):
            shot = out / f"{i:02d}{'abcdefghij'[j] if end - start > SLICE else ''}-{names[i][:48]}.png"
            page.screenshot(
                path=shot, full_page=True,
                clip={"x": 0, "y": y, "width": width, "height": min(SLICE, end - y)},
            )
            shots.append(shot)
    context.close()
    return problems, shots


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("pages", nargs="*")
    parser.add_argument("--changed", metavar="BASE")
    parser.add_argument("--blog", action="store_true")
    parser.add_argument("--base-url")
    parser.add_argument("--site", default=str(ROOT / "site"))
    parser.add_argument("--out", default=str(ROOT / "site-shots"))
    parser.add_argument("--chromium", help="a Chromium binary, instead of Playwright's own")
    args = parser.parse_args()
    pages = list(args.pages)
    if args.changed:
        pages += changed_pages(args.changed)
    if args.blog:
        pages += blog_pages()
    pages = sorted(set(pages))
    out = Path(args.out)
    report = [f"# Page screenshots\n\nPages: {len(pages)}.\n"]
    if not pages:
        print("no pages to look at", file=sys.stderr)
        out.mkdir(parents=True, exist_ok=True)
        (out / "report.md").write_text("".join(report) + "\nNo page changed.\n")
        return
    server, base = (None, args.base_url) if args.base_url else serve(Path(args.site))
    if not base.endswith("/"):
        base += "/"
    failed = 0
    with sync_playwright() as p:
        browser = p.chromium.launch(executable_path=args.chromium)
        for path in pages:
            report.append(f"\n## {path or '/'}\n")
            for viewport in VIEWPORTS:
                for theme in THEMES:
                    slug = (path.strip("/").replace("/", "_") or "home")
                    problems, shots = look(
                        browser, base + path, out / slug / f"{viewport}-{theme}", viewport, theme
                    )
                    failed += len(problems)
                    report.append(
                        f"\n**{viewport}, {theme}:** {len(shots)} screenshots, "
                        f"{len(problems)} problems\n"
                    )
                    report += [f"- {p}\n" for p in problems]
                    for problem in problems:
                        print(f"{path} ({viewport}, {theme}): {problem}")
        browser.close()
    if server:
        server.shutdown()
    (out / "report.md").write_text("".join(report))
    print(f"{len(pages)} pages looked at; screenshots and report in {out}", file=sys.stderr)
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
