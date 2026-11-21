#!/usr/bin/env python3
"""Why each release happened: CHANGELOG.md's notes and the blog posts that explain them, as
the History page's data.

    python3 docs/scripts/release_notes.py               # write the asset and the Markdown
    python3 docs/scripts/release_notes.py --candidates  # also list posts that link a shipped PR

Writes:

- docs/site/assets/release-notes.json, read by the History page's hero (bench.js): per
  release, newest first, its date, its lede, the "why" sentences, its shipped changes with
  their pull requests, and the posts tagged with it;
- docs/generated/release-notes.md, included by the History page: the same, one entry a
  release, each under an anchor `release-X-Y-Z` that the posts' "Shipped in" lines link to.

From each `## [X.Y.Z](…) - date` section of CHANGELOG.md (parse()):

- **the lede:** the first sentence of the section's first plain paragraph, before any
  `###` heading ("A kernel release: …");
- **the why:** the first sentence of each hand-written bold summary, a paragraph or a
  top-level bullet that opens with `**`. A bold bullet that names its pull request as a
  link, in a Performance, Bug Fixes, Features or Breaking section, is a change instead;
- **the changes:** the top-level bullets of the Performance, Bug Fixes, Features and Breaking
  sections, each with its scope and pull requests, and the bullets of the other sections
  (not Documentation) whose pull request the hand-written notes cite.

A sentence of fewer than four words ("Caveats.") takes the next one with it, and a bold label
with nothing after it ("**Caveats:**" over a list) is left out.

A post is tagged with the releases it explains through its front matter's `releases` key,
`releases: ["0.5.8"]`; every version it names must be a section of CHANGELOG.md. Candidate
links (--candidates) are posts that link a pull request shipped in a release they aren't
tagged with: a list for a person to judge, never applied.
"""

import argparse
import json
import re
import sys
from datetime import date
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CHANGELOG = ROOT / "CHANGELOG.md"
BLOG = ROOT / "docs" / "site" / "blog"
ASSET = ROOT / "docs" / "site" / "assets" / "release-notes.json"
OUT = ROOT / "docs" / "generated" / "release-notes.md"
REPO = "https://github.com/lmmx/mibz2"

VERSION = re.compile(
    r"^## \[?(\d+\.\d+\.\d+)\]?(?:\(([^)]*)\))?\s*[-–—]\s*(\d{4}-\d{2}-\d{2})\s*$"
)
CHANGE_SECTIONS = {"performance": "perf", "bug fixes": "fix", "features": "feat", "breaking": "breaking"}
PR_LINK = re.compile(r"\[#(\d+)\]\((https?://[^)\s]+)\)")
SCOPE = re.compile(r"^\*\(([^)]*)\)\*\s*")
# subsections of figures, whose bold labels are not reasons
MEASUREMENT = ("measured", "measurement", "the cost", "caveats", "who gets what", "packages",
               "downstream", "benchmarks")
# bold labels of figures and caveats, wherever they are
LABELS = re.compile(
    r"^(caveats?|the cost|cost|result|method|machine|workload|level|reference|same bytes|"
    r"memory|known issue|sustained)\b",
    re.I,
)
ABBREV = ("e.g.", "i.e.", "vs.", "cf.", "etc.", "approx.", "no.", "fig.")


def strip_md(text: str) -> str:
    """Inline Markdown as read: links to their text, emphasis marks dropped, code kept."""
    text = re.sub(r"\[([^\]]*)\]\([^)]*\)", r"\1", text)
    text = re.sub(r"\*\*(.+?)\*\*", r"\1", text)
    text = re.sub(r"(?<![\w*])\*(?!\s)(.+?)(?<!\s)\*(?![\w*])", r"\1", text)
    text = re.sub(r"<[^>]+>", "", text)
    return " ".join(text.split())


def sentences(text: str) -> list[str]:
    """Plain text split into sentences: at . ! ? before a space and a capital, digit or quote,
    not after a common abbreviation."""
    out, start = [], 0
    for m in re.finditer(r"[.!?](?=\s+[A-Z0-9`“\"(#§])", text):
        end = m.end()
        tail = text[max(0, end - 8):end].lower()
        if any(tail.endswith(a) for a in ABBREV):
            continue
        out.append(text[start:end].strip())
        start = end
    rest = text[start:].strip()
    if rest:
        out.append(rest)
    return [s for s in out if s]


def first_sentence(text: str, limit: int = 320) -> str | None:
    """The first sentence of a block of Markdown, with the next one if it is under four words
    or is only the block's bold label ("**Wider admission** (#559)."); None when there is
    nothing past a bare label. Long sentences are cut at a word, with …."""
    ss = sentences(strip_md(text))
    if not ss:
        return None
    s = ss[0]
    # a bold label with no claim of its own (no colon), its pull request aside
    bold = re.match(r"^\*\*(.+?)\*\*", text)
    unref = lambda t: re.sub(r"\s*\(#\d+[^)]*\)", "", t).rstrip(" .")  # noqa: E731
    label = unref(strip_md(bold.group(1))) if bold else None
    bare = bool(label) and ":" not in label and unref(s) == label
    if len(s.split()) < 4 or bare:
        if len(ss) < 2:
            return None
        s = f"{s} {ss[1]}"
    if len(s) > limit:
        s = s[:limit].rsplit(" ", 1)[0].rstrip(",;:") + "…"
    return s


def blocks(lines: list[str]):
    """A section's top-level blocks: (kind, subsection, first paragraph's text, whole text).

    kind is "para" or "item" (a top-level bullet). <details> and fenced code are skipped, and
    so are tables and headings below ###. An item's first paragraph stops at a blank line, a
    nested bullet or a table; its whole text runs to the next top-level block.
    """
    section, cur, depth, fence = None, None, 0, False
    out = []

    def close():
        nonlocal cur
        if cur:
            out.append((cur["kind"], cur["section"], " ".join(cur["first"]), " ".join(cur["all"])))
        cur = None

    for raw in lines:
        line = raw.rstrip("\n")
        s = line.strip()
        if s.startswith("```"):
            fence = not fence
            continue
        if fence:
            continue
        depth += len(re.findall(r"<details\b", s)) - len(re.findall(r"</details>", s))
        if depth > 0 or s.startswith("</details>"):
            close()
            continue
        if line.startswith("### "):
            close()
            section = re.sub(r"<!--.*?-->", "", line[4:]).strip()
            continue
        if not s:
            if cur:
                cur["open"] = False
            if cur and cur["kind"] == "para":
                close()
            continue
        top_item = re.match(r"^[-*] ", line)
        if top_item and cur and cur["kind"] == "para" and cur["first"][0].startswith("**"):
            # a list straight under a bold summary, with no blank line: that summary's own
            cur["open"] = False
            cur["all"].append(s)
            continue
        if top_item:
            close()
            cur = {"kind": "item", "section": section, "first": [line[2:].strip()], "all": [line[2:].strip()], "open": True}
            continue
        if line.startswith(("|", "#", "<")):
            if cur:
                cur["open"] = False
            if not line[:1].isspace() and cur and cur["kind"] == "para":
                close()
            continue
        if line[:1].isspace():
            if cur is None:
                continue
            nested = re.match(r"^\s+([-*]|\d+\.) ", line) or s.startswith("|")
            if nested:
                cur["open"] = False
            elif cur["open"]:
                cur["first"].append(s)
            cur["all"].append(s)
            continue
        # a line at column 0 that is not a bullet: a paragraph, or a paragraph's next line
        if cur and cur["kind"] == "para":
            cur["first"].append(s)
            cur["all"].append(s)
        else:
            close()
            cur = {"kind": "para", "section": section, "first": [s], "all": [s], "open": True}
    close()
    return out


def measurement(section: str | None) -> bool:
    """A subsection of figures and their caveats, not of reasons (### Measured, ### The cost)."""
    return (section or "").strip().lower().startswith(MEASUREMENT)


def change_kind(section: str | None) -> str | None:
    return CHANGE_SECTIONS.get((section or "").strip().lower())


def parse_change(first: str, kind: str) -> dict:
    """A changelog bullet as a change: its scope, its title as read, and its pull requests."""
    scope = None
    m = SCOPE.match(first)
    if m:
        scope, first = m.group(1), first[m.end():]
    prs = [{"n": int(n), "url": url} for n, url in PR_LINK.findall(first)]
    bold = re.match(r"^\*\*(.+?)\*\*", first)
    if bold:
        title = strip_md(bold.group(1))
    else:
        # the text before the first "([#N](…))"
        title = strip_md(re.split(r"\s*\(\[#\d+\]", first, maxsplit=1)[0])
    title = title.rstrip(" .:")
    seen, uniq = set(), []
    for p in prs:
        if p["n"] not in seen:
            seen.add(p["n"])
            uniq.append(p)
    return {"kind": kind, "scope": scope, "title": title, "prs": uniq}


def gh_anchor(heading: str) -> str:
    """GitHub's anchor for a Markdown heading's rendered text."""
    s = heading.strip().lower()
    s = re.sub(r"[^\w\- ]", "", s)
    return s.replace(" ", "-")


def parse(text: str) -> list[dict]:
    """Each released version's notes, newest first (as CHANGELOG.md lists them)."""
    lines = text.splitlines()
    heads = [(i, VERSION.match(line)) for i, line in enumerate(lines)]
    heads = [(i, m) for i, m in heads if m]
    out = []
    for k, (i, m) in enumerate(heads):
        end = heads[k + 1][0] if k + 1 < len(heads) else len(lines)
        # stop at the next level-2 heading of any kind (an [Unreleased] after it, say)
        for j in range(i + 1, end):
            if lines[j].startswith("## "):
                end = j
                break
        version, compare, day = m.group(1), m.group(2), m.group(3)
        bl = blocks(lines[i + 1:end])
        lede, plain_lede, why, changes, cited_from = None, None, [], [], []
        other_items = []
        for kind, section, first, whole in bl:
            ck = change_kind(section)
            is_bold = first.startswith("**")
            if kind == "item" and ck and not (is_bold and not PR_LINK.search(first)):
                # a change names its pull request or its scope, as release-plz writes it
                if PR_LINK.search(first) or SCOPE.match(first):
                    changes.append(parse_change(first, ck))
                    continue
                cited_from.append(whole)
                continue
            if kind == "item" and not is_bold and PR_LINK.search(first):
                # a release-plz bullet in another section: kept if the notes cite it
                if (section or "").strip().lower() != "documentation":
                    other_items.append(first)
                continue
            if kind == "item" and not is_bold:
                if section is None or (section or "").strip().lower() not in ("documentation", "other"):
                    cited_from.append(whole)
                continue
            cited_from.append(whole)
            if is_bold:
                if measurement(section) or LABELS.match(strip_md(first)):
                    continue
                s = first_sentence(first)
                if s and not s.endswith(":") and s not in why:
                    why.append(s)
            elif kind == "para" and section is None and lede is None:
                lede = first_sentence(first)
            elif kind == "para" and not measurement(section) and not plain_lede and len(first.split()) >= 30:
                plain_lede = first_sentence(first)
        # no plain paragraph before the first ###: the first long one under any (0.2.6's,
        # under Other), not a closing note ("The step has its row in …")
        lede = lede or plain_lede
        cited = {int(n) for n in re.findall(r"#(\d+)\b", " ".join(cited_from))}
        have = {p["n"] for c in changes for p in c["prs"]}
        for first in other_items:
            c = parse_change(first, "other")
            if any(p["n"] in cited and p["n"] not in have for p in c["prs"]):
                changes.append(c)
                have |= {p["n"] for p in c["prs"]}
        heading = f"{version} - {day}"
        out.append({
            "version": version,
            "date": day,
            "anchor": "release-" + version.replace(".", "-"),
            "compare": compare,
            "changelog": f"{REPO}/blob/master/CHANGELOG.md#{gh_anchor(heading)}",
            "lede": lede,
            "why": why,
            "changes": changes,
        })
    return out


def front_matter(path: Path) -> dict:
    """A page's front matter: flat `key: value` lines, quotes off a scalar."""
    m = re.match(r"---\n(.*?)\n---\n", path.read_text(), re.S)
    meta = {}
    if m:
        for line in m.group(1).splitlines():
            key, _, value = line.partition(":")
            if key.strip():
                meta[key.strip()] = value.strip()
    return meta


def post_releases(value: str | None, where: str) -> list[str]:
    """A post's `releases` value: a JSON list of version strings, such as ["0.5.8"]."""
    if not value:
        return []
    try:
        got = json.loads(value)
    except json.JSONDecodeError as e:
        raise SystemExit(f"{where}: releases must be a list of quoted versions, such as [\"0.5.8\"] ({e})")
    if not isinstance(got, list) or not all(isinstance(v, str) and re.fullmatch(r"\d+\.\d+\.\d+", v) for v in got):
        raise SystemExit(f"{where}: releases must be a list of quoted versions, such as [\"0.5.8\"]")
    return got


def posts(blog: Path) -> list[dict]:
    """Every post with its title, its site path, its tagged releases and the PRs it links."""
    out = []
    for p in sorted(blog.glob("*.md")):
        if p.name == "index.md":
            continue
        meta = front_matter(p)
        text = p.read_text()
        prs = {int(n) for n in re.findall(r"github\.com/lmmx/mibz2/pull/(\d+)", text)}
        out.append({
            "file": p.name,
            "title": meta.get("title", "").strip('"'),
            "path": f"blog/{p.stem}/",
            "date": p.name[:10],
            "releases": post_releases(meta.get("releases"), f"blog/{p.name}"),
            "prs": sorted(prs),
        })
    return out


def attach(notes: list[dict], allposts: list[dict]) -> None:
    """Each release's tagged posts; fails on a tag naming no CHANGELOG section."""
    by = {r["version"]: r for r in notes}
    for r in notes:
        r["posts"] = []
    bad = []
    for p in allposts:
        for v in p["releases"]:
            if v not in by:
                bad.append(f"blog/{p['file']}: releases names {v}, which has no section in CHANGELOG.md")
                continue
            by[v]["posts"].append({"title": p["title"], "path": p["path"], "file": p["file"]})
    if bad:
        raise SystemExit("\n".join(bad))


def candidates(notes: list[dict], allposts: list[dict]) -> list[dict]:
    """Posts that link a pull request shipped in a release they aren't tagged with."""
    shipped = {}
    for r in notes:
        for c in r["changes"]:
            for pr in c["prs"]:
                shipped.setdefault(pr["n"], (r["version"], c["title"]))
    out = []
    for p in allposts:
        for n in p["prs"]:
            if n in shipped and shipped[n][0] not in p["releases"]:
                out.append({"post": p["file"], "release": shipped[n][0], "pr": n, "change": shipped[n][1]})
    return out


SHIPPED = re.compile(r"^Shipped in mibz2 .*\n\{: \.post-shipped \}\n\n?", re.M)
META = re.compile(r"^\{: \.post-meta \}\n", re.M)
CHIP = re.compile(r'<span class="post-card-rel">[^<]*</span>')


def shipped_line(versions: list[str], by: dict) -> str:
    """A tagged post's line under its date: each release, linked to its entry on the History
    page and to its CHANGELOG section."""
    parts = [
        f"[{v}](../benchmarks/history.md#{by[v]['anchor']}) ([changelog]({by[v]['changelog']}))"
        for v in versions
    ]
    joined = parts[0] if len(parts) == 1 else ", ".join(parts[:-1]) + " and " + parts[-1]
    return f"Shipped in mibz2 {joined}\n{{: .post-shipped }}\n"


def chip(versions: list[str]) -> str:
    return f'<span class="post-card-rel">Shipped in {" and ".join(versions)}</span>'


def sync_posts(allposts: list[dict], by: dict, blog: Path, write: bool) -> list[str]:
    """Each tagged post's "Shipped in" line, under its date, and its card's chip on the blog
    index: rewritten when write, else listed where they differ from the front matter."""
    problems = []
    index = blog / "index.md"
    idx = index.read_text() if index.exists() else ""
    new_idx = idx
    for p in allposts:
        path = blog / p["file"]
        text = path.read_text()
        want = shipped_line(p["releases"], by) if p["releases"] else ""
        have = SHIPPED.search(text)
        fixed = SHIPPED.sub("", text)
        if want:
            m = META.search(fixed)
            if not m:
                problems.append(f"blog/{p['file']}: tagged with releases but has no post-meta line")
                continue
            fixed = fixed[:m.end()] + "\n" + want + fixed[m.end():]
        if fixed != text:
            if write:
                path.write_text(fixed)
            else:
                problems.append(
                    f"blog/{p['file']}: its \"Shipped in\" line " + ("is missing or differs from" if want else "has no")
                    + " its front matter's releases (python3 docs/scripts/release_notes.py --write-posts)"
                )
        # the card: <a class="post-card" href="FILE"> … </span></a>, the chip last in its text
        cm = re.search(rf'<a class="post-card" href="{re.escape(p["file"])}">.*?</a>', new_idx)
        if not cm:
            continue
        card = cm.group(0)
        bare = CHIP.sub("", card)
        want_card = bare.replace("</span></a>", chip(p["releases"]) + "</span></a>") if p["releases"] else bare
        if want_card != card:
            new_idx = new_idx.replace(card, want_card)
    if new_idx != idx:
        if write:
            index.write_text(new_idx)
        else:
            problems.append("blog/index.md: a card's release chip differs from its post's releases (--write-posts)")
    return problems


def long_date(day: str) -> str:
    d = date.fromisoformat(day)
    return f"{d.day} {d.strftime('%B')} {d.year}"


def md_escape(text: str) -> str:
    """Plain text as Markdown: outside code spans, emphasis marks, brackets and < escaped."""
    parts = re.split(r"(`[^`]*`)", text)
    for i in range(0, len(parts), 2):
        parts[i] = re.sub(r"([*_\[\]\\])", r"\\\1", parts[i]).replace("<", "&lt;")
    return "".join(parts)


def markdown(notes: list[dict]) -> str:
    """The History page's entries, one a release, newest first."""
    out = []
    for r in notes:
        out += [f"### {r['version']} {{#{r['anchor']}}}", ""]
        meta = [long_date(r["date"]), f"[the CHANGELOG]({r['changelog']})"]
        if r["compare"]:
            meta.append(f"[its diff]({r['compare']})")
        out += [" · ".join(meta), "{: .rn-meta }", ""]
        if r["lede"]:
            out += [md_escape(r["lede"]), ""]
        if r["why"]:
            out += [f"- {md_escape(w)}" for w in r["why"]] + [""]
        if r["changes"]:
            items = []
            for c in r["changes"]:
                prs = ", ".join(f"[#{p['n']}]({p['url']})" for p in c["prs"])
                items.append(f"{md_escape(c['title'])}{f' ({prs})' if prs else ''}")
            out += ["**Changes:** " + "; ".join(items) + ".", ""]
        if r["posts"]:
            reads = ", ".join(f"[{p['title']}](../blog/{p['file']})" for p in r["posts"])
            out += [f"**Read:** {reads}", ""]
    return "\n".join(out).rstrip() + "\n"


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--candidates", action="store_true", help="list posts that link a shipped PR untagged")
    ap.add_argument("--write-posts", action="store_true", help="rewrite the posts' Shipped in lines and the cards' chips")
    args = ap.parse_args()
    notes = parse(CHANGELOG.read_text())
    allposts = posts(BLOG)
    attach(notes, allposts)
    problems = sync_posts(allposts, {r["version"]: r for r in notes}, BLOG, args.write_posts)
    if problems:
        raise SystemExit("\n".join(problems))
    ASSET.write_text(json.dumps({"releases": notes}, ensure_ascii=False, separators=(",", ":")))
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(markdown(notes))
    tagged = sum(1 for p in allposts if p["releases"])
    print(f"{OUT.relative_to(ROOT)}, {ASSET.relative_to(ROOT)}: {len(notes)} releases, {tagged} tagged posts")
    if args.candidates:
        for c in candidates(notes, allposts):
            print(f"candidate: {c['post']} -> {c['release']} (#{c['pr']}: {c['change']})")


if __name__ == "__main__":
    sys.exit(main())
