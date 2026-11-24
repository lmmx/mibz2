"""Find where a blog post drops its reader in medias res.

    python3 docs/scripts/link_audit.py docs/site/blog/<post>.md

It reports two things, for the link-and-gloss skill (.claude/skills/link-and-gloss/SKILL.md):

1. **Used before it is explained.** A term the post defines in bold (**term**) or names in a
   heading, but uses earlier in the prose, with the line of first use and the line of the
   definition.
2. **Explained elsewhere, not linked here.** A term that another blog post defines (in its title,
   its headings or its bold terms), used in this post with no link on the paragraph of its first
   use. Each comes with the posts that explain it, so the writer can link the right one.

Figures, code, front matter and the Sources footer are skipped. The report is a worklist, not a
verdict: a common word can match by accident, and the writer decides what each line needs (a link
at first use, a gloss beside it, or nothing).
"""

import re
import sys
from pathlib import Path

BLOG = Path(__file__).resolve().parents[1] / "site" / "blog"
STOP = {
    "the", "a", "an", "and", "or", "of", "in", "on", "to", "for", "with", "by", "is", "are", "it",
    "what", "how", "why", "where", "when", "this", "that", "from", "at", "as", "be", "not", "its",
    "our", "we", "one", "two", "three", "new", "first", "next", "more", "less", "faster", "slower",
}


def prose(text):
    """The post's prose paragraphs as (line number, text), skipping figures, code and the footer."""
    lines = text.splitlines()
    out, skip, fence = [], False, False
    body_start = 0
    if lines and lines[0] == "---":
        body_start = lines.index("---", 1) + 1
    for i, line in enumerate(lines[body_start:], start=body_start + 1):
        if line.startswith("```"):
            fence = not fence
            continue
        if fence:
            continue
        if "<figure" in line:
            skip = True
        if skip:
            if "</figure>" in line:
                skip = False
            continue
        if line.strip() == "---" or line.startswith("**Sources:**") or "{ #compendium" in line:
            break
        out.append((i, line))
    return out


def terms_defined(text):
    """Bold terms and heading phrases a post defines."""
    found = set()
    for m in re.finditer(r"\*\*([^*\n]{2,60}?)\*\*", text):
        t = re.sub(r"\[([^\]]+)\]\([^)]*\)", r"\1", m.group(1)).strip(" .,:;")
        if t:
            found.add(t)
    for m in re.finditer(r"^#{1,3} (?:\d+\.\s*)?(.+)$", text, re.M):
        found.add(m.group(1).strip())
    return found


def key(term):
    words = [w for w in re.findall(r"[\w₁'’-]+", term.lower()) if w not in STOP]
    return " ".join(words)


def index_other_posts(skip, here):
    """Terms other posts explain: their bold terms and the short form of their titles.

    Only multi-word terms and proper names (with a capital or a digit) are kept, so that a
    common word in someone else's bold lead-in doesn't flood the report."""
    idx = {}
    for p in sorted(BLOG.glob("*.md")):
        if p.name in (skip, "index.md") or p.name.endswith("-notes.md"):
            continue
        t = p.read_text()
        title = re.search(r'^title: "(.*)"$', t, re.M)
        names = {re.sub(r"\[([^\]]+)\]\([^)]*\)", r"\1", m.group(1)).strip(" .,:;")
                 for m in re.finditer(r"\*\*([^*\n]{2,60}?)\*\*", t)}
        if title:
            names.add(title.group(1).split(":")[0])
        for n in names:
            k = key(n)
            words = k.split()
            single = len(n.split()) == 1
            acronym = bool(re.fullmatch(r"[A-Z0-9₁]{2,}", n))
            # a single word counts only if it is a name: capitalised mid-sentence in this post
            name = single and bool(re.search(r"[a-z,;] " + re.escape(n.capitalize()) + r"\b", here))
            proper = acronym or name
            if ((2 <= len(words) <= 4 and len(n.split()) <= 5) or proper) and len(k) >= 3:
                idx.setdefault(k, set()).add(p.name)
    return idx


def main(path):
    path = Path(path)
    text = path.read_text()
    paras = prose(text)
    joined = [(n, l) for n, l in paras]

    def first_use(term):
        pat = re.compile(r"(?<![\w-])" + re.escape(term) + r"(?![\w-])", re.I)
        for n, l in joined:
            if pat.search(re.sub(r"\]\([^)]*\)", "]", l)):
                return n, l
        return None, None

    print(f"# Link audit: {path.name}\n")
    print("## 1. Used before it is explained\n")
    hits = 0
    for t in sorted(terms_defined(text)):
        if len(t) < 3 or len(t.split()) > 5:
            continue
        defn = next((n for n, l in joined if f"**{t}" in l or re.match(rf"#+ (\d+\.\s*)?{re.escape(t)}", l)), None)
        use, _ = first_use(t)
        if defn and use and use < defn:
            hits += 1
            print(f"- `{t}`: first used at line {use}, explained at line {defn}")
    if not hits:
        print("- none")
    print("\n## 2. Explained in another post, not linked at first use here\n")
    idx = index_other_posts(path.name, text)
    hits = 0
    for k, posts in sorted(idx.items()):
        if k in STOP or len(k) < 4:
            continue
        n, line = first_use(k)
        if not n:
            continue
        para = [l for m, l in joined if abs(m - n) <= 2]
        line = next(l for m, l in joined if m == n)
        # linked if the paragraph links one of the explaining posts, or the line of first use
        # links anything (the writer may have chosen a better post than the index suggests)
        linked = any(p.replace(".md", "") in " ".join(para) for p in posts) or "](" in line
        if not linked:
            hits += 1
            print(f"- `{k}` (line {n}): explained in {', '.join(sorted(posts)[:3])}")
    if not hits:
        print("- none")


if __name__ == "__main__":
    main(sys.argv[1])
