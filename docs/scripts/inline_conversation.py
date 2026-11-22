"""Inline a standalone conversation page into its blog post.

The conversation pages (docs/site/blog/conversations/*.html) are built as whole documents. A
post shows one inline, in the blog's own page, with no frame and no link: this script takes the
document's body, fonts, styles and script, scopes every style rule to a `.convo` wrapper so it
cannot touch the site around it (and the site's own `.md-typeset` rules are reset inside it),
maps the page's light and dark themes onto the site's palette toggle, and writes one HTML
fragment with no blank lines, which the post includes with pymdownx.snippets. The heavy parts
go to asset files, not the page: the scoped stylesheet, the script, and every embedded
(data URI) image, written once each under the asset directory.

    uv run --no-project -p 3.13 --with tinycss2 python docs/scripts/inline_conversation.py \
        docs/site/blog/conversations/2026-11-22-thomas-murphy.html \
        docs/conversations/2026-11-22-thomas-murphy.inline.html \
        conversation-thomas-murphy

Run it again whenever the standalone page is rebuilt.
"""

import base64
import hashlib
import os
import re
import sys

import tinycss2

SCOPE = ".convo"
DARK = '[data-md-color-scheme="slate"] .convo'


def scope_selector(sel: str) -> str:
    """One selector (no top-level commas) scoped to the wrapper."""
    s = sel.strip()
    if not s:
        return s
    # the page's explicit dark theme, and its OS-dark default, both become the site's slate
    for pat in (r':root\[data-theme="dark"\]', r':root:not\(\[data-theme="light"\]\)'):
        if re.match(pat, s):
            rest = re.sub("^" + pat, "", s)
            return DARK + rest if rest.startswith((" ", ".", ":", "[", ">")) or not rest else DARK + " " + rest
    if re.match(r':root\[data-theme="light"\]', s):
        rest = re.sub(r'^:root\[data-theme="light"\]', "", s)
        return '[data-md-color-scheme="default"] .convo' + rest
    m = re.match(r"(:root|html|body)(?![\w-])", s)
    if m:
        return SCOPE + s[m.end():]
    if s.startswith("*"):
        return f"{SCOPE} {s}, {SCOPE}"
    return f"{SCOPE} {s}"


def split_selectors(prelude: str) -> list[str]:
    out, depth, cur = [], 0, ""
    for ch in prelude:
        if ch in "([":
            depth += 1
        elif ch in ")]":
            depth -= 1
        if ch == "," and depth == 0:
            out.append(cur)
            cur = ""
        else:
            cur += ch
    out.append(cur)
    return out


def scope_rules(rules, in_dark_media=False) -> str:
    out = []
    for r in rules:
        if r.type == "qualified-rule":
            prelude = tinycss2.serialize(r.prelude)
            sels = [scope_selector(x) for x in split_selectors(prelude)]
            if in_dark_media:
                # inside @media (prefers-color-scheme: dark), only the :root:not(light) form
                # carries meaning, and it is already mapped to the site's dark palette
                sels = [x for x in sels if x.startswith(DARK)]
                if not sels:
                    continue
            out.append(", ".join(sels) + "{" + tinycss2.serialize(r.content) + "}")
        elif r.type == "at-rule":
            kw = r.at_keyword.lower()
            prelude = tinycss2.serialize(r.prelude).strip()
            if kw == "media" and r.content is not None:
                inner = tinycss2.parse_rule_list(r.content, skip_comments=True, skip_whitespace=True)
                if "prefers-color-scheme" in prelude and "dark" in prelude:
                    out.append(scope_rules(inner, in_dark_media=True))
                else:
                    out.append(f"@media {prelude}{{{scope_rules(inner)}}}")
            elif kw in ("supports", "layer", "container") and r.content is not None:
                inner = tinycss2.parse_rule_list(r.content, skip_comments=True, skip_whitespace=True)
                out.append(f"@{kw} {prelude}{{{scope_rules(inner)}}}")
            else:  # keyframes, font-face, import: global by nature, left as they are
                out.append(tinycss2.serialize([r]))
    return "".join(out)


# The site's Markdown styles that would otherwise reach into the conversation.
RESET = (
    ".md-typeset .convo{font-size:initial;line-height:normal}"
    ".md-typeset .convo mark{background:none;color:inherit;box-shadow:none;padding:0;margin:0;border-radius:0}"
    ".md-typeset .convo a{color:inherit;text-decoration:inherit;word-break:normal}"
    ".md-typeset .convo h2{margin:0;font-weight:inherit;letter-spacing:inherit;line-height:inherit;color:inherit}"
    ".md-typeset .convo p,.md-typeset .convo ol,.md-typeset .convo ul,.md-typeset .convo li{margin:0;padding:0}"
    ".md-typeset .convo ol li{margin-left:0}"
    ".md-typeset .convo img{max-width:none}"
    ".md-typeset .convo sup{line-height:1}"
    ".md-typeset .convo aside,.md-typeset .convo small{font-size:inherit}"
    ".md-typeset .convo svg{max-width:none}"
    ".md-typeset .convo code{background:none;padding:0;font-size:inherit;color:inherit}"
)


EXT = {"image/png": "png", "image/jpeg": "jpg", "image/webp": "webp", "image/gif": "gif", "image/svg+xml": "svg"}


class _Fake:
    """A stand-in match for img(): the MIME type and the base64 data."""

    def __init__(self, mime, data):
        self._g = (None, mime, data)

    def group(self, i):
        return self._g[i]


# Last, so they win: the conversation paints no background of its own, takes the site's page
# colour wherever it masks "the page", and keeps to the post's column.
LAST = (
    "html .md-typeset .convo,html [data-md-color-scheme] .md-typeset .convo"
    "{--bg:var(--md-default-bg-color);background:transparent;max-width:100%;overflow-x:clip}"
    ".md-typeset .convo article{max-width:620px;margin:0 auto;padding:0 14px}"
    # the Compendium breaks out to the viewport on its own page; in the post it keeps to the column
    ".md-typeset .convo .compendium{width:auto;left:auto;translate:none}"
)


def main(src: str, dst: str, name: str) -> None:
    site = "docs/site"
    adir = f"assets/conversations/{name}"
    os.makedirs(f"{site}/{adir}/img", exist_ok=True)
    t = open(src, encoding="utf-8").read()
    head, body = t.split("<body>", 1)
    body = body.rsplit("</body>", 1)[0]
    links = "".join(re.findall(r'<link[^>]+fonts\.googleapis[^>]*>', head))
    css = "".join(re.findall(r"<style>(.*?)</style>", head, flags=re.S))
    rules = tinycss2.parse_stylesheet(css, skip_comments=True, skip_whitespace=True)
    open(f"{site}/{adir}/convo.css", "w", encoding="utf-8").write(RESET + scope_rules(rules) + LAST + "\n")
    scripts = re.findall(r"<script>(.*?)</script>", body, flags=re.S)
    open(f"{site}/{adir}/convo.js", "w", encoding="utf-8").write("\n".join(scripts) + "\n")
    body = re.sub(r"<script>.*?</script>", "", body, flags=re.S)
    # embedded images in the markup become files, one per distinct image
    seen = {}

    def img(m):
        mime, data = m.group(1), m.group(2)
        raw = base64.b64decode(data)
        h = hashlib.sha256(raw).hexdigest()[:16]
        fn = f"{h}.{EXT.get(mime, 'bin')}"
        if h not in seen:
            open(f"{site}/{adir}/img/{fn}", "wb").write(raw)
            seen[h] = fn
        return f'src="../{adir}/img/{fn}"'

    body = re.sub(r'src="data:([\w/+.-]+);base64,([A-Za-z0-9+/=]+)"', img, body)
    # SVG <image>: the site theme rewrites every relative [href], and an SVG image's href
    # cannot be assigned, so the file goes in xlink:href, which that selector does not match
    body = re.sub(r'(<image\b[^>]*?)\shref="data:([\w/+.-]+);base64,([A-Za-z0-9+/=]+)"',
                  lambda m: m.group(1) + " xlink:" + img(_Fake(m.group(2), m.group(3))).replace("src=\"../", "href=\"../../", 1), body)
    # the same for every other SVG element with an href (<use href="#...">, <textPath>, ...)
    body = re.sub(r'(<(?:use|textPath|feImage|mpath)\b[^>]*?)\shref=', r'\1 xlink:href=', body)
    # the post's own hero carries the title and byline
    body = re.sub(r"<header>.*?</header>", "", body, count=1, flags=re.S)
    body = body.replace("<main>", "", 1).replace("</main>", "", 1)
    # links written for the standalone page's location, rewritten for the post's (Markdown-relative)
    body = body.replace('src="../../assets/', 'src="../assets/')
    body = re.sub(r'href="\.\./([\w-]+)/(#[^"]*)?"', lambda m: f'href="{m.group(1)}.md{m.group(2) or ""}"', body)
    frag = (
        links
        + f'<link rel="stylesheet" href="../{adir}/convo.css">'
        + f'<div class="convo">{body}</div>'
        + f'<script src="../{adir}/convo.js" defer></script>'
    )
    # one raw HTML block for Markdown: no blank lines anywhere
    frag = "\n".join(line for line in frag.split("\n") if line.strip())
    os.makedirs(os.path.dirname(dst), exist_ok=True)
    open(dst, "w", encoding="utf-8").write(frag + "\n")
    print(f"{dst}: {len(frag):,} bytes; {len(seen)} images; {len(rules)} rules scoped")


if __name__ == "__main__":
    main(*sys.argv[1:4])
