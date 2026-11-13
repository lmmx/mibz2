"""Run unchanged site browser checks through a recorded local asset fixture.

The environment's Chromium lacks the proxy CA; Google Fonts is denied. urllib
uses the inherited verified CA to fetch the site's exact KaTeX pin. Only the
Google Fonts CSS response is replaced, with pinned Fontsource IBM Plex families
at the same styles/weights. This tests real rendering, not public CDN availability.
No TLS verification is disabled; no production site configuration is modified.
"""

import base64
import hashlib
import html
import importlib.util
import json
import mimetypes
import os
from pathlib import Path
import runpy
import sys
import urllib.request

from playwright.sync_api import Browser, sync_playwright

ROOT = Path(__file__).resolve().parents[3]
CACHE = Path("/tmp/mibz2-blog-assets")
CACHE.mkdir(exist_ok=True)
RECORDS = {}
BASE = "https://cdn.jsdelivr.net/npm/"


def fetch(url):
    """Fetch with the environment CA and keep exact bytes for reproducibility."""
    path = CACHE / hashlib.sha256(url.encode()).hexdigest()
    if not path.exists():
        with urllib.request.urlopen(url, timeout=30) as response:
            path.write_bytes(response.read())
    body = path.read_bytes()
    RECORDS[url] = dict(bytes=len(body), sha256=hashlib.sha256(body).hexdigest())
    return body


def font_css():
    """Mirror the configured IBM Plex faces; do not silently use fallback fonts."""
    out = []
    for family, weights in [
        ("ibm-plex-sans", [300, 400, 500, 600, 700]),
        ("ibm-plex-mono", [400, 700]),
    ]:
        base = BASE + "@fontsource/" + family + "@5.2.7/"
        for weight in weights:
            for style in ["", "-italic"]:
                css = fetch(base + f"latin-{weight}{style}.css").decode()
                out.append(css.replace("url(./", "url(" + base))
    return "\n".join(out).encode()


def attach(context):
    """Preserve the page's requests, with explicit asset responses and hashes."""

    def route(request):
        url = request.request.url
        if url.startswith("https://fonts.googleapis.com/"):
            request.fulfill(body=font_css(), content_type="text/css")
        else:
            body = fetch(url)
            mime = mimetypes.guess_type(url.split("?")[0])[0] or "application/octet-stream"
            request.fulfill(body=body, content_type=mime)

    context.route("https://cdn.jsdelivr.net/**", route)
    context.route("https://fonts.googleapis.com/**", route)
    return context


def save_manifest():
    """Record all fixture inputs, including exact fonts and KaTeX assets."""
    path = ROOT / "docs/journal/2026-11-12-rauzy-companion/evidence/browser-assets.json"
    path.write_text(json.dumps(RECORDS, indent=2, sort_keys=True) + "\n")


def social():
    """Use the site's card template with an exact1200x630 Playwright viewport."""
    spec = importlib.util.spec_from_file_location("social", ROOT / "docs/scripts/social_cards.py")
    s = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(s)
    key = "assets/social/blog-2026-11-12-rauzy-technical-companion.png"
    fields = s.cards()[key]
    escaped = {k: html.escape(str(v)) for k, v in fields.items()}
    page_html = s.PAGE.format(
        w=s.W,
        h=s.H,
        title_px=60,
        dek_px=28,
        dek_lines=3,
        hero="data:image/svg+xml;base64," + base64.b64encode(s.HERO.read_bytes()).decode(),
        **escaped,
    )
    path = CACHE / "social.html"
    path.write_text(page_html)
    with sync_playwright() as p:
        browser = p.chromium.launch(executable_path="/usr/bin/chromium")
        context = attach(browser.new_context(viewport={"width": 1200, "height": 630}))
        page = context.new_page()
        page.set_content(page_html, wait_until="networkidle")
        page.evaluate("document.fonts.ready")
        assert page.locator(".hero").evaluate("(e) => e.complete && e.naturalWidth > 0")
        page.screenshot(path=str(CACHE / "social-inspect.png"))
        for selector in ["h1", "p", ".foot"]:
            box = page.locator(selector).bounding_box()
            assert box["y"] + box["height"] <= 630, (selector, box)
            assert page.locator(selector).evaluate("""(e) => {
                const height = e.getBoundingClientRect().height;
                const old = e.style.webkitLineClamp;
                e.style.webkitLineClamp = 'unset';
                const full = e.getBoundingClientRect().height;
                e.style.webkitLineClamp = old;
                return full <= height + 1;
            }"""), selector
        page.screenshot(path=str(s.SITE / key))
        browser.close()
    manifest = json.loads(s.MANIFEST.read_text())
    manifest[key] = fields
    s.MANIFEST.write_text(json.dumps(manifest, indent=2, ensure_ascii=False, sort_keys=True) + "\n")


if __name__ == "__main__":
    os.chdir(ROOT)
    if sys.argv[1:] == ["social"]:
        social()
    else:
        original = Browser.new_context

        def mirrored(self, *args, **kwargs):
            """Attach the explicit fixture to every unmodified checker context."""
            return attach(original(self, *args, **kwargs))

        Browser.new_context = mirrored
        sys.argv[0] = str(ROOT / "docs/scripts/look_pages.py")
        try:
            runpy.run_path(sys.argv[0], run_name="__main__")
        finally:
            save_manifest()
    save_manifest()
