"""A browser test of the blog's copy-announcement control (claim blog-announcement-copy).

Serves the built site (site/, from `just docs-build`) on localhost and drives it in Chromium
with Playwright. The expected text for every post is worked out here from the Markdown
sources, independently of the page's script:

    <kind emoji> mibz2 blog ⠶ <the card's title> <site_url>blog/<slug>/

where the emoji is the post's kind, from its card's label through social_cards.py's KINDS (💡 a
result, 🧩 a synthesis, 🔬 research in progress, ❓ an open question, 🧭 meta; 📣 for an unknown
label), and the card is the post's `<a class="post-card" href="<slug>.md">` in
docs/site/blog/index.md and site_url is zensical.toml's. It checks:

- the index has exactly one control per post card, none nested in a card's link, and none
  elsewhere (the home page has none);
- clicking each copies exactly its post's line to the clipboard, without navigating, and
  shows "Copied" only after the write succeeded;
- the sample line given in the claim, byte for byte;
- the control's accessible name and tooltip ("Copy announcement"), a touch target of at
  least 40 by 40 CSS pixels on desktop and phone widths, and keyboard use (Enter, Space);
- with the clipboard refused (no Clipboard API write, no execCommand copy), a selectable
  fallback holding the one line, its text selected, and no "Copied";
- each indexed post's own page has one control copying the same line;
- a post newly added to the index gets a control with no other work, and its title's HTML
  entities are decoded (served with an extra card injected into the index);
- a card still opens its post.

It writes light and dark screenshots of the index and of one post to --shots for a person
to look at. Exits 1 on any failure.

    uv run --no-project -p 3.13 --with playwright==1.56.0 python docs/scripts/check_announce.py \
        [--site site] [--shots site-shots/announce] [--chromium PATH]
"""

from __future__ import annotations

import argparse
import functools
import html
import http.server
import re
import sys
import threading
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(Path(__file__).resolve().parent))
from social_cards import KINDS  # noqa: E402  (the label -> kind table that colours the cards)
INDEX_MD = ROOT / "docs/site/blog/index.md"
PREFIX = " mibz2 blog ⠶ "
EMOJI = {"result": "💡", "synthesis": "🧩", "progress": "🔬", "question": "❓", "meta": "🧭"}
FALLBACK = "📣"
SAMPLE = (
    "💡 mibz2 blog ⠶ The decoder's true floor and the limits of a pooled scan "
    "https://lmmx.github.io/mibz2/blog/2026-11-26-true-floor-and-pooled-scan/"
)
LABEL = "Copy announcement"

CARD = re.compile(
    r'<a class="post-card" href="(?P<href>[^"]+)\.md"[^>]*>.*?'
    r'<span class="post-card-date">(?P<date>.*?)</span>.*?'
    r'<span class="post-card-title">(?P<title>.*?)</span>',
    re.S,
)


def expected() -> dict[str, str]:
    """Slug -> announcement line, from the index's cards and the configured site URL."""
    site_url = tomllib.loads((ROOT / "zensical.toml").read_text())["project"]["site_url"]
    out: dict[str, str] = {}
    for m in CARD.finditer(INDEX_MD.read_text(encoding="utf-8")):
        title = html.unescape(re.sub(r"<[^>]+>", "", m["title"])).strip()
        label = html.unescape(m["date"]).split("·")[-1].strip()
        emoji = EMOJI.get(KINDS.get(label, ""), FALLBACK)
        out[m["href"]] = f"{emoji}{PREFIX}{title} {site_url}blog/{m['href']}/"
    return out


class Quiet(http.server.SimpleHTTPRequestHandler):
    """A static file server that keeps its request log to itself."""

    def log_message(self, *args):
        """Log nothing."""
        pass


def serve(site: Path) -> tuple[http.server.ThreadingHTTPServer, str]:
    """Serve SITE on a free localhost port; return the server and its base URL."""
    handler = functools.partial(Quiet, directory=str(site))
    httpd = http.server.ThreadingHTTPServer(("127.0.0.1", 0), handler)
    threading.Thread(target=httpd.serve_forever, daemon=True).start()
    # The site is built for /mibz2/ but every link in it is relative, so it serves from /.
    return httpd, f"http://localhost:{httpd.server_address[1]}/"


REFUSE_CLIPBOARD = """
(() => {
  const refuse = () => Promise.reject(new DOMException("refused", "NotAllowedError"));
  if (navigator.clipboard) navigator.clipboard.writeText = refuse;
  document.execCommand = () => false;
})();
"""


def main() -> int:
    """Run every check; return the exit status."""
    ap = argparse.ArgumentParser()
    ap.add_argument("--site", default=str(ROOT / "site"))
    ap.add_argument("--shots", default=str(ROOT / "site-shots/announce"))
    ap.add_argument("--chromium", help="a Chromium binary, instead of Playwright's own")
    args = ap.parse_args()
    site, shots = Path(args.site), Path(args.shots)
    shots.mkdir(parents=True, exist_ok=True)

    from playwright.sync_api import sync_playwright

    want = expected()
    problems: list[str] = []

    def check(ok: bool, what: str) -> None:
        if not ok:
            problems.append(what)

    check(want.get("2026-11-26-true-floor-and-pooled-scan") == SAMPLE, "the sample line differs")
    check(len(want) >= 40, f"only {len(want)} cards parsed from the index")

    httpd, base = serve(site)
    with sync_playwright() as p:
        browser = p.chromium.launch(executable_path=args.chromium)
        ctx = browser.new_context(viewport={"width": 1280, "height": 900})
        ctx.grant_permissions(["clipboard-read", "clipboard-write"], origin=base.rstrip("/"))
        page = ctx.new_page()
        errors: list[str] = []
        page.on("pageerror", lambda e: errors.append(str(e)))

        def clip() -> str:
            return page.evaluate("navigator.clipboard.readText()")

        def reset_clip() -> None:
            page.evaluate("navigator.clipboard.writeText('')")

        # --- the index ---
        index = base + "blog/"
        page.goto(index)
        page.wait_for_selector(".post-announce")
        cards = page.locator(".post-list a.post-card")
        buttons = page.locator(".post-announce")
        check(
            cards.count() == len(want),
            f"{cards.count()} cards on the page, {len(want)} in the source",
        )
        check(buttons.count() == len(want), f"{buttons.count()} controls for {len(want)} posts")
        check(
            page.locator("a .post-announce, a.post-announce").count() == 0,
            "a control inside a link",
        )
        for i in range(buttons.count()):
            b = buttons.nth(i)
            slug = b.get_attribute("data-post") or ""
            check(slug in want, f"control {i} names an unknown post {slug!r}")
            check(b.get_attribute("aria-label") == LABEL, f"{slug}: accessible name")
            check(b.get_attribute("title") == LABEL, f"{slug}: tooltip")
            check(
                b.evaluate("e => e.tagName") == "BUTTON" and b.get_attribute("type") == "button",
                f"{slug}: not a button",
            )
            check(b.inner_text().strip() == "", f"{slug}: visible label {b.inner_text()!r}")
            reset_clip()
            b.click()
            page.wait_for_function(
                "e => e.dataset.state === 'copied'", arg=b.element_handle(), timeout=3000
            )
            got = clip()
            check(got == want.get(slug), f"{slug}: copied {got!r}, want {want.get(slug)!r}")
            check(page.url == index, f"{slug}: click navigated to {page.url}")
        check(
            len(set(buttons.evaluate_all("bs => bs.map(b => b.dataset.post)"))) == buttons.count(),
            "duplicate controls",
        )
        status = page.locator(".post-announce-status")
        check(
            status.count() == 1 and "Copied" in status.inner_text(), "no live Copied announcement"
        )

        # keyboard: Enter and Space
        for key in ("Enter", " "):
            reset_clip()
            first = buttons.nth(0)
            first.evaluate("e => { delete e.dataset.state; }")
            first.focus()
            page.keyboard.press(key if key != " " else "Space")
            page.wait_for_function(
                "e => e.dataset.state === 'copied'", arg=first.element_handle(), timeout=3000
            )
            check(
                clip() == want[first.get_attribute("data-post")], f"keyboard {key!r} did not copy"
            )

        # touch targets, desktop and phone
        for w, h in ((1280, 900), (390, 844)):
            page.set_viewport_size({"width": w, "height": h})
            for i in range(buttons.count()):
                box = buttons.nth(i).bounding_box()
                check(
                    box is not None and box["width"] >= 40 and box["height"] >= 40,
                    f"{w}px: control {i} box {box}",
                )
            overflow = page.evaluate(
                "document.documentElement.scrollWidth - document.documentElement.clientWidth"
            )
            check(overflow <= 0, f"{w}px: page scrolls sideways by {overflow}px")
        page.set_viewport_size({"width": 1280, "height": 900})

        for scheme in ("light", "dark"):
            page.emulate_media(color_scheme=scheme)
            page.evaluate(
                "s => { document.body.setAttribute('data-md-color-scheme', s === 'dark' ? 'slate' : 'default'); }",
                scheme,
            )
            page.locator(".post-list").screenshot(path=str(shots / f"index-{scheme}.png"))
            page.set_viewport_size({"width": 390, "height": 844})
            page.screenshot(path=str(shots / f"index-phone-{scheme}.png"))
            page.set_viewport_size({"width": 1280, "height": 900})

        # a card still opens its post
        page.locator(".post-list a.post-card .post-card-title").nth(0).click()
        page.wait_for_load_state()
        first_slug = list(want)[0]
        check(page.url.rstrip("/").endswith(first_slug), f"card opened {page.url}")

        # --- each post's own page ---
        for slug, line in want.items():
            page.goto(f"{base}blog/{slug}/")
            bs = page.locator(".post-announce")
            check(bs.count() == 1, f"{slug} page: {bs.count()} controls")
            if bs.count() != 1:
                continue
            reset_clip()
            bs.click()
            page.wait_for_function(
                "e => e.dataset.state === 'copied'", arg=bs.element_handle(), timeout=3000
            )
            got = clip()
            check(got == line, f"{slug} page: copied {got!r}, want {line!r}")
            check(page.url.rstrip("/").endswith(slug), f"{slug} page: click navigated")
            if slug == "2026-11-26-true-floor-and-pooled-scan":
                for scheme in ("light", "dark"):
                    page.evaluate(
                        "s => document.body.setAttribute('data-md-color-scheme', s)",
                        "slate" if scheme == "dark" else "default",
                    )
                    page.locator("article h1").first.evaluate("e => e.scrollIntoView()")
                    page.screenshot(path=str(shots / f"post-{scheme}.png"))

        page.goto(base)
        check(page.locator(".post-announce").count() == 0, "a control on the home page")

        # --- a newly indexed post, with entities in its title ---
        new_card = (
            '<a class="post-card" href="2026-12-01-new-post/"><span class="post-card-text">'
            '<span class="post-card-date">1 December 2026 · Meta</span>'
            '<span class="post-card-title">Q&amp;A: &lt;tags&gt; &amp; caf&eacute; &#x2014; &quot;quotes&quot;</span>'
            "</span></a>"
        )

        def inject(route):
            resp = route.fetch()
            body = resp.text().replace(
                '<div class="post-list">', '<div class="post-list">' + new_card, 1
            )
            route.fulfill(response=resp, body=body)

        page.route(re.compile(r".*/blog/(index\.html)?$"), inject)
        page.goto(index)
        page.wait_for_selector(".post-announce")
        nb = page.locator('.post-announce[data-post="2026-12-01-new-post"]')
        check(nb.count() == 1, "the newly indexed post has no control")
        if nb.count() == 1:
            reset_clip()
            nb.click()
            page.wait_for_function(
                "e => e.dataset.state === 'copied'", arg=nb.element_handle(), timeout=3000
            )
            want_new = (
                EMOJI["meta"]
                + PREFIX
                + 'Q&A: <tags> & café — "quotes" https://lmmx.github.io/mibz2/blog/2026-12-01-new-post/'
            )
            check(clip() == want_new, f"new post: copied {clip()!r}")
        page.unroute(re.compile(r".*/blog/(index\.html)?$"))

        # --- the clipboard refused: the selectable fallback ---
        ctx2 = browser.new_context(viewport={"width": 390, "height": 844})
        ctx2.add_init_script(REFUSE_CLIPBOARD)
        p2 = ctx2.new_page()
        p2.on("pageerror", lambda e: errors.append(str(e)))
        p2.goto(index)
        p2.wait_for_selector(".post-announce")
        b = p2.locator(".post-announce").nth(4)
        slug = b.get_attribute("data-post")
        b.click()
        fb = p2.locator(".post-announce-fallback")
        fb.wait_for(state="visible", timeout=3000)
        field = fb.locator("textarea, input")
        check(field.count() == 1, "fallback has no text field")
        if field.count() == 1:
            check(field.input_value() == want[slug], f"fallback holds {field.input_value()!r}")
            sel = field.evaluate(
                "e => [e.selectionStart, e.selectionEnd, e.value.length, document.activeElement === e]"
            )
            check(
                sel[0] == 0 and sel[1] == sel[2] and sel[3],
                f"fallback text not selected and focused: {sel}",
            )
        check(b.get_attribute("data-state") != "copied", "Copied shown although the write failed")
        check(p2.url == index, "fallback navigated")
        p2.screenshot(path=str(shots / "fallback-phone.png"))
        p2.keyboard.press("Escape")
        check(not fb.is_visible(), "Escape did not close the fallback")
        ctx2.close()

        check(not errors, f"script errors: {errors}")
        browser.close()
    httpd.shutdown()

    for p_ in problems:
        print("FAIL:", p_)
    print(f"{len(want)} posts; {len(problems)} problems; screenshots in {shots}")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
