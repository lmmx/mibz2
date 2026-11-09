"""Sentence complexity of a blog post: a plain, measurable check for overloaded prose.

    python docs/scripts/check_prose.py docs/site/blog/POST.md [...] [--all]
    python docs/scripts/check_prose.py --changed origin/master

For every sentence of running prose (front matter, HTML/SVG, code, tables, maths and headings
are skipped), it counts:
- words: the sentence's length;
- load: how many pieces it is built from. Each comma, semicolon, colon, dash, parenthesis
  and number adds one. A sentence of one idea has a load of 0-2.

A sentence is flagged when it has more than 25 words, or a load of 5 or more.

The dek (the subtitle, `{: .post-dek }`) must sit in a band: one well-built sentence, neither a
mess of overloaded fragments nor childishly plain (Louis, 9 November 2026):
- one sentence, of 16 to 28 words;
- at most one internal break (comma, colon, semicolon, dash or parenthesis);
- at most two figures;
- Flesch reading ease from 35 to 70: below 35 is dense, above 70 reads as a primer.

It prints the post's summary (sentences, mean words, share flagged, Flesch reading ease, where
higher is easier and 60+ is plain English), the dek's verdict, and the worst sentences.
--all lists every flagged sentence. --changed BASE checks the posts changed since BASE (CI's
site workflow runs it on every pull request). The exit status is 1 if a dek fails. The body's
share of flagged sentences is reported against a target of 15%; it isn't yet a gate, since
the existing posts are over it.
"""
import re
import sys

LONG, HEAVY = 25, 5
DEK_WORDS = (16, 28)
DEK_BREAKS, DEK_FIGURES = 1, 2
DEK_FLESCH = (35, 70)
MAX_FLAGGED = 0.15


def prose_blocks(text):
    text = re.sub(r"\A---\n.*?\n---\n", "", text, flags=re.S)
    text = re.sub(r"<figure.*?</figure>", "", text, flags=re.S)
    text = re.sub(r"<svg.*?</svg>", "", text, flags=re.S)
    text = re.sub(r"```.*?```", "", text, flags=re.S)
    text = re.sub(r"\$\$.*?\$\$", "", text, flags=re.S)
    dek, blocks = None, []
    for block in re.split(r"\n\s*\n", text):
        lines = [l for l in block.splitlines() if l.strip()]
        if not lines:
            continue
        if any(l.lstrip().startswith(("|", "#", "<", "{:", ">")) for l in lines[:1]):
            if lines[-1].strip() == "{: .post-dek }":
                pass
            else:
                continue
        is_dek = lines[-1].strip() == "{: .post-dek }"
        lines = [l for l in lines if not l.strip().startswith(("{:", "|", "<"))]
        body = " ".join(re.sub(r"^\s*(?:[-*]|\d+\.)\s+", "", l) for l in lines)
        body = re.sub(r"\[([^\]]*)\]\([^)]*\)", r"\1", body)  # links: keep the text
        body = re.sub(r"`[^`]*`", "X", body)
        body = re.sub(r"\$[^$]*\$", "X", body)
        body = re.sub(r"[*_]{1,2}", "", body)
        if is_dek:
            dek = body
        elif body.strip():
            blocks.append(body)
    return dek, blocks


def sentences(block):
    parts = re.split(r"(?<=[.!?])\s+(?=[A-Za-z0-9\"“(])", block.strip())
    return [p.strip() for p in parts if len(p.split()) >= 3]


def words(s):
    return re.findall(r"[A-Za-z0-9][A-Za-z0-9'’.%-]*", s)


def breaks(s):
    return len(re.findall(r"[,;:—–]|\s-\s|\(", s))


def figures(s):
    return len(re.findall(r"(?<![A-Za-z])\d+(?:[.,]\d+)*%?(?![A-Za-z])", s))


def load(s):
    return breaks(s) + figures(s)


def syllables(w):
    w = w.lower().strip(".'’%")
    if not w or w[0].isdigit():
        return 2
    groups = re.findall(r"[aeiouy]+", w)
    n = len(groups) - (1 if w.endswith("e") and len(groups) > 1 else 0)
    return max(1, n)


def flesch(sents):
    ws = [w for s in sents for w in words(s)]
    if not sents or not ws:
        return 0.0
    syl = sum(syllables(w) for w in ws)
    return 206.835 - 1.015 * len(ws) / len(sents) - 84.6 * syl / len(ws)


def dek_check(dek):
    ds = sentences(dek) or [dek]
    n, b, f, fl = len(words(dek)), breaks(dek), figures(dek), flesch(ds)
    problems = []
    if len(ds) != 1:
        problems.append(f"{len(ds)} sentences, not 1")
    if not DEK_WORDS[0] <= n <= DEK_WORDS[1]:
        problems.append(f"{n} words, not {DEK_WORDS[0]}-{DEK_WORDS[1]}")
    if b > DEK_BREAKS:
        problems.append(f"{b} breaks, more than {DEK_BREAKS}")
    if f > DEK_FIGURES:
        problems.append(f"{f} figures, more than {DEK_FIGURES}")
    if not DEK_FLESCH[0] <= fl <= DEK_FLESCH[1]:
        problems.append(f"reading ease {fl:.0f}, not {DEK_FLESCH[0]}-{DEK_FLESCH[1]}")
    summary = f"{n} words, {b} break(s), {f} figure(s), reading ease {fl:.0f}"
    return not problems, summary + ("; " + "; ".join(problems) if problems else "")


def changed_posts(base):
    import subprocess
    out = subprocess.run(["git", "diff", "--name-only", f"{base}...HEAD", "--",
                          "docs/site/blog/"], capture_output=True, text=True, check=True)
    return [p for p in out.stdout.split() if p.endswith(".md") and not p.endswith("index.md")]


def main():
    argv = sys.argv[1:]
    show_all = "--all" in argv
    if "--changed" in argv:
        files = changed_posts(argv[argv.index("--changed") + 1])
    else:
        files = [a for a in argv if not a.startswith("--")]
    if not files:
        print("no blog posts to check")
        return 0
    status = 0
    for path in files:
        status |= check(path, show_all)
    return status


def check(path, show_all):
    dek, blocks = prose_blocks(open(path, encoding="utf-8").read())
    sents = [s for b in blocks for s in sentences(b)]
    scored = [(len(words(s)), load(s), s) for s in sents]
    flagged = [x for x in scored if x[0] > LONG or x[1] >= HEAVY]
    share = len(flagged) / len(scored) if scored else 0.0
    mean = sum(x[0] for x in scored) / len(scored) if scored else 0.0
    print(path)
    print(f"  body: sentences {len(scored)}, mean {mean:.1f} words, flagged {len(flagged)} "
          f"({share:.0%}; target at most {MAX_FLAGGED:.0%}), reading ease {flesch(sents):.0f}"
          + ("" if share <= MAX_FLAGGED else "  [over target: reported, not yet a gate]"))
    ok = True
    if dek is None:
        print("  dek: none found")
    else:
        dek_ok, why = dek_check(dek)
        ok = dek_ok
        print(f"  dek: {'PASS' if dek_ok else 'FAIL'} ({why})")
    worst = sorted(flagged, key=lambda x: (x[1], x[0]), reverse=True)
    for n, l, s in worst if show_all else worst[:8]:
        print(f"  [{n} words, load {l}] {s[:200]}{'…' if len(s) > 200 else ''}")
    return 0 if ok else 1


if __name__ == "__main__":
    import signal
    signal.signal(signal.SIGPIPE, signal.SIG_DFL)
    sys.exit(main())
