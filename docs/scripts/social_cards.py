"""The site's social cards: the 1200 × 630 images link previews show (Open Graph, Twitter).

docs/overrides/main.html puts a page's card in its <head>: the page's front-matter `image`
if it has one, else the site's default card, docs/site/assets/social/default.png.

This script draws the default card (the site's description and how to install mibz2) and a card for each
blog post whose front matter names an `image` under assets/social/ (the post's title, its
description and its date), each beside the home page's illustration. Each card is an HTML
page rendered to PNG by headless Chromium.

Run from the repository root, with a Chromium or Chrome binary:

    python docs/scripts/social_cards.py [--chrome PATH]

The binary is found on PATH (headless_shell, chromium, chromium-browser, google-chrome,
chrome) unless --chrome or the CHROME environment variable names one. Prefer the headless
shell (chrome-headless-shell): full Chrome's headless mode can render a viewport shorter
than --window-size, cutting off the card's foot.
"""

import argparse
import html
import os
import re
import shutil
import subprocess
import sys
import tempfile
import tomllib
from datetime import date
from pathlib import Path

SITE = Path("docs/site")
OUT = SITE / "assets/social"
HERO = SITE / "assets/images/hero.svg"
W, H = 1200, 630

PAGE = """<!doctype html>
<html><head><meta charset="utf-8">
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=IBM+Plex+Sans:wght@400;600&display=block">
<style>
  html, body {{ margin: 0; width: {w}px; height: {h}px; overflow: hidden; }}
  body {{
    position: relative; background: #fbfaf7; color: #1c1c1a;
    font-family: "IBM Plex Sans", "Inter", sans-serif;
  }}
  .text {{
    position: absolute; left: 76px; top: 64px; bottom: 70px; width: 700px;
    display: flex; flex-direction: column;
  }}
  .kicker {{ font-size: 26px; font-weight: 600; color: #0f6e74; letter-spacing: 0.01em; }}
  .kicker span {{ color: #8a8983; font-weight: 400; }}
  h1 {{
    margin: 26px 0 0; font-size: {title_px}px; line-height: 1.12; font-weight: 600;
    letter-spacing: -0.01em;
    display: -webkit-box; -webkit-line-clamp: 3; -webkit-box-orient: vertical; overflow: hidden;
  }}
  p {{
    margin: 24px 0 0; font-size: 27px; line-height: 1.4; color: #5d5c56;
    display: -webkit-box; -webkit-line-clamp: 3; -webkit-box-orient: vertical; overflow: hidden;
  }}
  .foot {{ margin-top: auto; font-size: 22px; color: #8a8983; }}
  .hero {{ position: absolute; right: 30px; top: 50%; width: 400px; transform: translateY(-50%); }}
  .bar {{ position: absolute; left: 0; right: 0; bottom: 0; height: 12px; background: #0f6e74; }}
</style></head>
<body>
  <div class="text">
    <div class="kicker">mibz2<span>{section}</span></div>
    <h1>{title}</h1>
    <p>{description}</p>
    <div class="foot">{foot}</div>
  </div>
  <img class="hero" src="{hero}" alt="">
  <div class="bar"></div>
</body></html>
"""


def front_matter(path):
    """A page's front matter (flat `key: value` lines) and its first H1."""
    text = path.read_text()
    meta = {}
    m = re.match(r"---\n(.*?)\n---\n", text, re.S)
    if m:
        for line in m.group(1).splitlines():
            key, _, value = line.partition(":")
            meta[key.strip()] = value.strip().strip('"')
    h1 = re.search(r"^# (.+)$", text, re.M)
    return meta, h1.group(1).strip() if h1 else None


def find_chrome(arg):
    """The Chromium binary: --chrome, else $CHROME, else the first found on PATH."""
    for candidate in (arg, os.environ.get("CHROME")):
        if candidate:
            return candidate
    for name in (
        "headless_shell",
        "chrome-headless-shell",
        "chromium",
        "chromium-browser",
        "google-chrome",
        "chrome",
    ):
        if found := shutil.which(name):
            return found
    sys.exit("no Chromium found: pass --chrome PATH or set CHROME")


def render(chrome, out, **fields):
    """Render one card (section, title, description, foot) to the PNG `out`."""
    fields = {k: html.escape(str(v)) for k, v in fields.items()}
    title_px = 60 if len(fields["title"]) <= 70 else 52
    page = PAGE.format(w=W, h=H, title_px=title_px, hero=HERO.resolve().as_uri(), **fields)
    with tempfile.TemporaryDirectory() as tmp:
        src = Path(tmp) / "card.html"
        src.write_text(page)
        subprocess.run(
            [
                chrome,
                "--headless",
                "--no-sandbox",
                "--hide-scrollbars",
                "--disable-gpu",
                "--force-device-scale-factor=1",
                f"--window-size={W},{H}",
                "--virtual-time-budget=10000",
                f"--screenshot={out.resolve()}",
                src.as_uri(),
            ],
            check=True,
            capture_output=True,
        )
    print(f"wrote {out}")


def main():
    """Draw the default card and each blog post's card."""
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--chrome", help="the Chromium or Chrome binary")
    chrome = find_chrome(parser.parse_args().chrome)
    OUT.mkdir(parents=True, exist_ok=True)

    config = tomllib.loads(Path("zensical.toml").read_text())["project"]
    host = re.sub(r"^https?://|/$", "", config["site_url"])
    render(
        chrome,
        OUT / "default.png",
        section="",
        title=config["site_description"],
        description="Block-parallel bzip2 decoding, byte for byte with libbzip2, from "
        "Python and Rust: pip install mibz2, cargo add mibz2.",
        foot=host,
    )

    for post in sorted((SITE / "blog").glob("*.md")):
        meta, title = front_matter(post)
        image = meta.get("image")
        if not image:
            continue
        if not image.startswith("assets/social/"):
            sys.exit(f"{post}: the card must be under assets/social/, not {image}")
        day = date.fromisoformat(post.name[:10])
        render(
            chrome,
            SITE / image,
            section=" · Research blog",
            title=title,
            description=meta["description"],
            foot=f"{host} · {day.day} {day:%B %Y}",
        )


if __name__ == "__main__":
    main()
