#!/usr/bin/env python3
"""Check the built site for Markdown that did not render as intended.

    uv run --no-project -p 3.13 python docs/scripts/check_render.py [--site DIR] [PAGE ...]

A strict build passes on a page whose Markdown rendered as something other than what its
author meant: a list with no blank line before it becomes one paragraph, a wrapped line
that starts with `#145` becomes a heading, an unclosed `**` shows its asterisks. This
reads the built HTML (default: every page under site/; or the pages given, as paths
relative to the site such as `blog/2026-11-04-autonomous-agent-team/`) and reports, with
the text it found:

- **Leaked syntax** in the text of a paragraph, list item, table cell, heading or caption
  (outside code, maths and SVG): `**` or `__`, a backtick, a link's `](` or `][`, a line
  starting with a list marker or a table row, an admonition's `!!!` or `???`, an
  attribute list's `{: `, a footnote's `[^`, and an HTML tag shown as text.
- **Headings:** exactly one H1 a page, and no level skipped on the way down (an H2
  followed by an H4).
- **Images:** every image has a `src` and an `alt` (empty for a decorative one).

It also reads the Markdown sources under docs/site/ for two hazards that render as valid
but wrong HTML, which the built page can no longer show:

- a line that starts with `#` and no space (`#145, so…` after a rewrap): a heading;
- a nested list item indented by fewer than 4 spaces: a sibling of its parent, not a
  child.

It cannot see what only a browser shows (layout, overflow, maths that KaTeX rejects,
script errors): docs/scripts/look_pages.py does that, and a person reads its
screenshots. See docs/site/maintainers/blogging.md.

Exits with 1 and lists every problem found.
"""

import argparse
import re
import sys
from html.parser import HTMLParser
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

# Elements whose text is prose a reader sees as written.
BLOCKS = {"p", "li", "td", "th", "dt", "dd", "figcaption", "summary", "caption"} | {
    f"h{n}" for n in range(1, 7)
}
# Elements whose text is not prose: code, maths, drawings, scripts.
OPAQUE = {"code", "pre", "kbd", "samp", "script", "style", "svg", "math", "textarea"}
OPAQUE_CLASSES = {"arithmatex", "katex", "highlight", "mermaid"}
VOID = {"area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source",
        "track", "wbr"}

# After a line break only: a paragraph that starts "1. " (bold, or escaped) is deliberate,
# since Markdown would otherwise have made it a list.
LIST_MARKER = re.compile(r"\n[ \t]*(?:[-*+]|\d+[.)])[ \t]+\S")
LEAKS = [
    (re.compile(r"\*\*|(?<![\w/])__\w"), "unrendered emphasis (** or __)"),
    (re.compile(r"`"), "unrendered code (a backtick)"),
    (re.compile(r"\]\(|\]\[[^\]]*\]"), "unrendered link (]( or ][)"),
    (LIST_MARKER, "a list marker inside text: a list with no blank line before it, or a hard wrap"),
    (re.compile(r"(?:^|\n)[ \t]*\|.*\|[ \t]*(?:\n|$)"), "a table row inside text"),
    (re.compile(r"(?:^|\n)[ \t]*(?:!!!|\?\?\?\+?)[ \t]"), "an unrendered admonition"),
    (re.compile(r"\{:[ \t]*[.#\w]"), "an unrendered attribute list ({: ...})"),
    (re.compile(r"\[\^[^\]]+\]"), "an unrendered footnote ([^...])"),
    (re.compile(r"</?(?:div|span|p|a|img|figure|figcaption|details|summary|table|br|em|strong)"
                r"\b[^<>]*>"), "an HTML tag shown as text"),
]


class Page(HTMLParser):
    """Collect each prose block's text, the headings in order, and the images."""

    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.stack = []  # (tag, opaque)
        self.blocks = []  # (tag, text) for each finished prose block
        self.open_blocks = []  # [tag, [text parts]]
        self.headings = []  # (level, text)
        self.images = []  # attrs
        self.in_article = 0

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        if tag == "img":
            if self.in_article:
                self.images.append(attrs)
            return
        if tag in VOID:
            return
        classes = set((attrs.get("class") or "").split())
        opaque = (
            tag in OPAQUE
            or bool(classes & OPAQUE_CLASSES)
            or bool(self.stack and self.stack[-1][1])
        )
        if tag == "article":
            self.in_article += 1
        self.stack.append((tag, opaque))
        if tag in BLOCKS and not opaque and self.in_article:
            self.open_blocks.append([tag, []])

    def handle_endtag(self, tag):
        # A closing tag with no open element of its own (the theme has some) closes
        # nothing, rather than everything.
        if tag in VOID or all(open_tag != tag for open_tag, _ in self.stack):
            return
        while self.stack:
            open_tag, _ = self.stack.pop()
            if open_tag == "article":
                self.in_article -= 1
            if open_tag in BLOCKS and self.open_blocks and self.open_blocks[-1][0] == open_tag:
                block_tag, parts = self.open_blocks.pop()
                text = "".join(parts)
                self.blocks.append((block_tag, text))
                if block_tag[0] == "h" and block_tag[1:].isdigit():
                    self.headings.append((int(block_tag[1]), text.strip()))
            if open_tag == tag:
                break

    def handle_data(self, data):
        if self.stack and self.stack[-1][1]:
            return
        # Text goes to the innermost open block only: a list nested in a list item is
        # checked as its own items, not as lines of the outer item.
        if self.open_blocks:
            self.open_blocks[-1][1].append(data)


def check_page(path, label, problems):
    page = Page()
    page.feed(path.read_text())
    for tag, text in page.blocks:
        # The heading-anchor permalink (¶) and similar decorations are not prose.
        shown = text.replace("¶", "").strip()
        if not shown:
            continue
        for pattern, what in LEAKS:
            m = pattern.search(shown)
            if m:
                start = max(0, m.start() - 40)
                excerpt = shown[start : m.end() + 60].replace("\n", "⏎")
                problems.append(f"{label}: <{tag}> {what}: …{excerpt}…")
    h1s = [text for level, text in page.headings if level == 1]
    if len(h1s) != 1:
        listed = "; ".join(t[:60] for t in h1s) or "none"
        problems.append(f"{label}: {len(h1s)} H1 headings, not 1 ({listed})")
    previous = 1
    for level, text in page.headings:
        if level > previous + 1:
            problems.append(f"{label}: H{level} '{text[:60]}' follows H{previous} (a level skipped)")
        previous = level
    for attrs in page.images:
        if not attrs.get("src"):
            problems.append(f"{label}: an image with no src")
        # alt="" is right for a decorative image; no alt at all is not.
        if "alt" not in attrs:
            problems.append(f"{label}: image {attrs.get('src', '')[:80]} has no alt attribute")


FENCE = re.compile(r"^[ \t]*(```|~~~)")
ITEM = re.compile(r"^( *)(?:[-*+]|\d+[.)])[ \t]+\S")
# Python-Markdown reads "#145" at the start of a line as a heading: no space is needed.
HASH_LINE = re.compile(r"^#{1,6}[^#\s]")


def check_source(path, label, problems):
    """Hazards that the HTML no longer shows: they render as something valid but wrong."""
    in_fence = False
    in_html = 0
    previous_item_indent = None  # the indent of the list item above, while in a list
    for number, line in enumerate(path.read_text().splitlines(), 1):
        if FENCE.match(line):
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        # Inside raw HTML blocks (figures, SVG) Markdown rules do not apply.
        in_html += len(re.findall(r"<(?:svg|figure|div)\b", line))
        in_html -= len(re.findall(r"</(?:svg|figure|div)>", line))
        if in_html > 0:
            continue
        if HASH_LINE.match(line) and number > 1:
            problems.append(
                f"{label}:{number}: a line starting '{line[:12]}' renders as a heading; "
                "rewrap the paragraph so the line does not start with #"
            )
        m = ITEM.match(line)
        if m:
            indent = len(m.group(1))
            if previous_item_indent is not None and previous_item_indent < indent < previous_item_indent + 4:
                problems.append(
                    f"{label}:{number}: a nested list item indented {indent - previous_item_indent} "
                    "spaces under its parent renders as a sibling, not nested; indent it by 4"
                )
            previous_item_indent = indent
        elif not line.strip():
            pass  # a blank line may sit between items of one list
        elif not line.startswith(" "):
            previous_item_indent = None


def pages(site, selected):
    if selected:
        for name in selected:
            path = site / name.strip("/")
            if path.is_dir() or not path.suffix:
                path = path / "index.html"
            yield path
        return
    for path in sorted(site.rglob("*.html")):
        rel = path.relative_to(site)
        if rel.parts[0] in {"assets", "pr-preview"} or path.name == "404.html":
            continue
        yield path


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--site", default=str(ROOT / "site"))
    parser.add_argument("pages", nargs="*")
    args = parser.parse_args()
    site = Path(args.site)
    problems = []
    docs = ROOT / "docs" / "site"
    for path in sorted(docs.rglob("*.md")):
        check_source(path, str(path.relative_to(ROOT)), problems)
    checked = 0
    for path in pages(site, args.pages):
        if not path.exists():
            problems.append(f"no built page at {path}")
            continue
        check_page(path, str(path.parent.relative_to(site)) + "/", problems)
        checked += 1
    print(f"{checked} pages checked for unrendered Markdown", file=sys.stderr)
    for problem in problems:
        print(problem)
    sys.exit(1 if problems else 0)


if __name__ == "__main__":
    main()
