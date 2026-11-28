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
site workflow runs it on every pull request); there a dek fails only if it is new or edited. The exit status is 1 if a dek, the reader gate or a title fails. The body's
share of flagged sentences is reported against a target of 15%; it isn't yet a gate, since
the existing posts are over it.

**The reader gate: no internal shorthand** (Louis, 27 November 2026: the blog is "something
worth telling the world"; dense shorthand belongs in the lab notebook, docs/notebook/). The
sentence measures above can't see shorthand: a sentence made of code-formatted identifiers is
short and scores as plain. So the body (before the closing **Sources:** list, without figures,
code blocks and display maths) is also counted per 1,000 words, and a new or changed post fails
when it exceeds any of these:
- inline code spans, more than 25 (claim ids, constants, flags and paths read as code);
- numbers set as code (`1.89%`, `0.9811`), more than 5: a measurement is prose, not code;
- pull-request and issue references (#1118), more than 10: they belong in Sources.
Commit hashes in the body are counted and reported, not gated: earlier accepted posts cite a few.
The limits sit above every post published before the swarm (the highest: 24.6 code spans, 4.9
numbers as code, 9.4 references per 1,000 words; calibrated 27 November 2026) and well below
the swarm's shorthand digests (32 to 220 code spans). A new or edited title of a main post (not
a technical companion) also fails if it is a list of three or more things, two commas or more:
a title names one finding, not a digest of events.

**Claims match state** (Louis, 27 November 2026): a post's title and dek may claim a speed,
memory or size effect only for changes in state `default` in `frontier/ledger.tsv`; any other
state must be named in the prose that reports it ("behind an experimental feature", "a
prototype", "the target"). For any post from 26 November 2026 onward (or named by a card in
`frontier/blog/*.toml`), a new or edited title or dek containing a throughput, percentage, time
or memory figure fails unless its blog card lists `changes` whose ledger rows are all `default`.
For each ledgered `experimental`, `prototype`, `target` or `awaiting-review` change on the card,
its first mention in the body (located by a `/pull/<N>` link, `#<N>` citation or a plain phrase
in the card's optional `[change_labels]` table, or across the body when unlinked) must carry the
matching state word.
"""

import csv
import pathlib
import re
import subprocess
import sys
import tomllib

LONG, HEAVY = 25, 5
DEK_WORDS = (16, 28)
DEK_BREAKS, DEK_FIGURES = 1, 2
DEK_FLESCH = (35, 70)
MAX_FLAGGED = 0.15
# The reader gate, per 1,000 words of body (see the docstring).
MAX_CODE, MAX_NUMCODE, MAX_REFS = 25, 5, 10

REPO = pathlib.Path(__file__).resolve().parents[2]
LEDGER_PATH = REPO / "frontier" / "ledger.tsv"
BLOG_CARDS_DIR = REPO / "frontier" / "blog"
# Posts published before the frontier blog board and shipped-changes ledger began (26 Nov 2026)
# have no blog card or ledger row and are exempt unless a card explicitly names them.
LEDGER_BOARD_START = "2026-11-26"

HEADLINE_FIGURE_RE = re.compile(
    r"(?<![A-Za-z0-9])"
    r"(?!(?:19|20)\d{2}s\b)"
    r"[+−-]?\d+(?:,\d{3})*(?:\.\d+)?"
    r"(?:"
    r"\s*(?:%"
    r"|(?:[kKMGT]i?)?B/s\b"
    r"|(?:Ki|Mi|Gi|Ti|K|k|M|G|T)B\b"
    r"|(?:ns|us|µs|μs|ms|secs?|s)\b)"
    r"|\s+(?:per\s*cent|percent"
    r"|bytes?(?:\s*/\s*s|\s+per\s+second)?"
    r"|(?:kilo|mega|giga|tera|kibi|mebi|gibi|tebi)bytes?(?:\s+per\s+second)?"
    r"|(?:[kKMGT]i?)?B\s+per\s+second"
    r"|(?:nano|micro|milli)?seconds?)\b"
    r")",
    re.IGNORECASE,
)

BODY_STATE_WORDS = {
    "experimental": (
        re.compile(
            r"\b(?:experimental|feature[- ]gated|opt[- ]in|"
            r"behind (?:an? )?(?:experimental |opt[- ]in |cargo |research )?"
            r"(?:feature|build option|option)(?:\s+flag)?)\b",
            re.IGNORECASE,
        ),
        '"experimental" or "behind an experimental feature"',
    ),
    "prototype": (
        re.compile(r"\b(?:prototypes?|prototyped|prototyping)\b", re.IGNORECASE),
        '"prototype"',
    ),
    "target": (
        re.compile(r"\b(?:targets?|targeted|targeting|projected|projection)\b", re.IGNORECASE),
        '"target"',
    ),
    "awaiting-review": (
        re.compile(
            r"\b(?:awaiting review|under review|in review|open pull request|open PR|unmerged|not yet merged)\b",
            re.IGNORECASE,
        ),
        '"awaiting review" or "in review"',
    ),
}

_UNSET = object()


def _norm_pr(val):
    s = str(val).strip()
    if s.startswith("#"):
        s = s[1:]
    return int(s) if s.isdigit() else s


def load_ledger(path=LEDGER_PATH):
    """Read frontier/ledger.tsv into a mapping from normalized PR id to row dict."""
    p = pathlib.Path(path)
    if not p.is_file():
        return {}
    lines = [
        ln for ln in p.read_text(encoding="utf-8").splitlines() if ln.strip() and not ln.lstrip().startswith("#")
    ]
    if not lines:
        return {}
    rows = {}
    for row in csv.DictReader(lines, delimiter="\t"):
        pr = row.get("pr")
        if pr is None or not pr.strip():
            continue
        key = _norm_pr(pr)
        rows[key] = {k: (v.strip() if isinstance(v, str) else v) for k, v in row.items()}
    return rows


def _norm_post_rel(post_path):
    p = pathlib.Path(post_path)
    try:
        return p.resolve().relative_to(REPO.resolve()).as_posix()
    except ValueError:
        return p.as_posix().lstrip("./")


def find_blog_cards(post_path, cards_dir=BLOG_CARDS_DIR):
    """Return every frontier/blog/*.toml card whose `post` names `post_path`."""
    d = pathlib.Path(cards_dir)
    if not d.is_dir():
        return []
    rel = _norm_post_rel(post_path)
    name = pathlib.Path(post_path).name
    matches = []
    for card_path in sorted(d.glob("*.toml")):
        data = tomllib.loads(card_path.read_text(encoding="utf-8"))
        card_post = data.get("post", "")
        if not isinstance(card_post, str) or not card_post.strip():
            continue
        card_rel = card_post.strip().lstrip("./")
        if card_rel == rel or pathlib.Path(card_rel).name == name:
            matches.append(data)
    non_legacy = [c for c in matches if not c.get("legacy")]
    return non_legacy or matches


def find_blog_card(post_path, cards_dir=BLOG_CARDS_DIR):
    """Return the matching blog card for `post_path`, or None."""
    cards = find_blog_cards(post_path, cards_dir=cards_dir)
    return cards[0] if cards else None


def _post_date(path):
    if path is None:
        return None
    m = re.match(r"^(\d{4}-\d{2}-\d{2})-", pathlib.Path(path).name)
    return m.group(1) if m else None


def headline_figures(s):
    """Return throughput, percentage, time or memory figures found in a title or dek."""
    if not s:
        return []
    return [m.group(0).strip() for m in HEADLINE_FIGURE_RE.finditer(s)]


def _headline_title(text):
    t = title_of(text)
    if t is not None:
        return t
    m = re.search(r"^#\s+(.+)$", text, re.M)
    return m.group(1).strip() if m else None


def _resolve_cards_and_ledger(path, card, ledger, cards_dir, ledger_path):
    if card is _UNSET:
        cards = find_blog_cards(path, cards_dir=cards_dir) if path is not None else []
    elif card is None:
        cards = []
    elif isinstance(card, list):
        cards = card
    else:
        cards = [card]
    led = load_ledger(ledger_path) if ledger is _UNSET else (ledger or {})
    return cards, led


def _card_changes(cards):
    changes = []
    seen = set()
    for c in cards:
        raw = c.get("changes") or []
        for item in raw:
            key = _norm_pr(item)
            if key not in seen:
                seen.add(key)
                changes.append(key)
    return changes


def _card_change_labels(cards):
    labels = {}
    for c in cards:
        raw = c.get("change_labels")
        if isinstance(raw, dict):
            for k, v in raw.items():
                if isinstance(v, str) and v.strip():
                    labels[_norm_pr(k)] = v.strip()
    return labels


def _headline_state_issues(text, *, path=None, card=_UNSET, ledger=_UNSET, cards_dir=BLOG_CARDS_DIR, ledger_path=LEDGER_PATH):
    """Return list of (where, message) for headline figures unsupported by default ledger rows."""
    cards, led = _resolve_cards_and_ledger(path, card, ledger, cards_dir, ledger_path)
    if not cards:
        post_date = _post_date(path)
        if post_date is not None and post_date < LEDGER_BOARD_START:
            return []
    title = _headline_title(text)
    dek, _ = prose_blocks(text)
    found = []
    for where, snippet in (("title", title), ("dek", dek)):
        figs = headline_figures(snippet)
        if figs:
            found.append((where, figs))
    if not found:
        return []

    changes = _card_changes(cards)
    issues = []
    for where, figs in found:
        fig_str = ", ".join(figs)
        if not cards:
            issues.append(
                (where, f"{where} claims {fig_str}, but no blog card in frontier/blog/ names this post with default-state changes")
            )
        elif not changes:
            issues.append((where, f"{where} claims {fig_str}, but the blog card lists no ledgered changes"))
        else:
            bad = []
            for pr in changes:
                row = led.get(pr)
                if row is None:
                    bad.append(f"#{pr} is not in frontier/ledger.tsv")
                elif row.get("state") != "default":
                    bad.append(f"#{pr} is in state '{row.get('state', '')}', not 'default'")
            if bad:
                issues.append((where, f"{where} claims {fig_str}, but card changes are not all default ({'; '.join(bad)})"))
    return issues


def headline_state_problems(
    text, *, path=None, card=_UNSET, ledger=_UNSET, cards_dir=BLOG_CARDS_DIR, ledger_path=LEDGER_PATH
):
    """Return error strings for title/dek figures unsupported by all-default ledgered changes."""
    return [
        msg
        for _, msg in _headline_state_issues(
            text, path=path, card=card, ledger=ledger, cards_dir=cards_dir, ledger_path=ledger_path
        )
    ]


def _body_paragraphs(text):
    """Body paragraphs before **Sources, preserving Markdown link targets for PR citation matching."""
    body = re.sub(r"\A---\n.*?\n---\n", "", text, flags=re.S)
    for pat in (r"<figure.*?</figure>", r"<svg.*?</svg>", r"```.*?```", r"\$\$.*?\$\$"):
        body = re.sub(pat, "\n\n", body, flags=re.S)
    i = body.rfind("**Sources")
    if i > 0:
        body = body[:i]
    paras = []
    for block in re.split(r"\n\s*\n", body):
        lines = [ln for ln in block.splitlines() if ln.strip()]
        if not lines:
            continue
        if any(ln.strip() in ("{: .post-dek }", "{: .post-kicker }", "{: .post-meta }") for ln in lines):
            continue
        if all(ln.lstrip().startswith(("<", "#", "{:")) for ln in lines):
            continue
        paras.append(" ".join(lines))
    return paras


def body_state_problems(
    text, *, path=None, card=_UNSET, ledger=_UNSET, cards_dir=BLOG_CARDS_DIR, ledger_path=LEDGER_PATH
):
    """Check that each non-default ledgered change on the card carries its state word at first mention."""
    cards, led = _resolve_cards_and_ledger(path, card, ledger, cards_dir, ledger_path)
    changes = _card_changes(cards)
    if not changes:
        return []
    labels = _card_change_labels(cards)
    paras = _body_paragraphs(text)
    full_body = "\n\n".join(paras)
    problems = []
    for pr in changes:
        row = led.get(pr)
        if row is None:
            continue
        state = row.get("state", "")
        if state not in BODY_STATE_WORDS:
            continue
        state_re, hint = BODY_STATE_WORDS[state]
        pr_str = re.escape(str(pr))
        cite_re = re.compile(rf"(?:/pull/{pr_str}\b|(?<![\w&])#{pr_str}\b|\bPR\s+{pr_str}\b)", re.IGNORECASE)
        label = labels.get(pr)
        label_re = re.compile(re.escape(label), re.IGNORECASE) if label else None
        first_para = None
        for para in paras:
            if cite_re.search(para) or (label_re is not None and label_re.search(para)):
                first_para = para
                break
        if first_para is not None:
            if not state_re.search(first_para):
                problems.append(f"first mention of {state} change #{pr} omits its state word (expected {hint})")
        elif not state_re.search(full_body):
            problems.append(f"body reports {state} change #{pr} without naming its state (expected {hint})")
    return problems


def claim_state_problems(
    text, *, path=None, card=_UNSET, ledger=_UNSET, cards_dir=BLOG_CARDS_DIR, ledger_path=LEDGER_PATH
):
    """Combined headline and body state-claim problems for a post."""
    return headline_state_problems(
        text, path=path, card=card, ledger=ledger, cards_dir=cards_dir, ledger_path=ledger_path
    ) + body_state_problems(
        text, path=path, card=card, ledger=ledger, cards_dir=cards_dir, ledger_path=ledger_path
    )


def gate_body(text):
    """The body the reader gate counts.

    No front matter, figures, code blocks, display maths or Sources list; links keep their text.
    """
    text = re.sub(r"\A---\n.*?\n---\n", "", text, flags=re.S)
    for pat in (r"<figure.*?</figure>", r"<svg.*?</svg>", r"```.*?```", r"\$\$.*?\$\$"):
        text = re.sub(pat, " ", text, flags=re.S)
    i = text.rfind("**Sources")
    if i > 0:
        text = text[:i]
    return re.sub(r"\]\([^)]*\)", "]", text)


def shorthand(text):
    """The reader gate's counts for a post: (problems, summary)."""
    body = gate_body(text)
    n = len(re.findall(r"[A-Za-z0-9]+", re.sub(r"<[^>]+>", " ", body)))
    per_k = 1000 / max(n, 1)
    code = re.findall(r"`[^`\n]+`", body)
    numcode = [c for c in code if re.fullmatch(r"`[~≈<>+−-]?[\d.,]+\s*(?:%|ms|s|MB|GB|MiB|KiB|GiB|×|x)?`", c)]
    plain = re.sub(r"`[^`\n]+`", " ", body)
    refs = re.findall(r"(?<![\w&])#\d{3,5}\b", plain) + [c for c in code if re.search(r"#\d{3,5}", c)]
    refs += re.findall(r"\bPRs? \d{3,5}\b", plain)
    hashes = re.findall(r"\b(?=[0-9a-f]*\d)(?=[0-9a-f]*[a-f])[0-9a-f]{7,40}\b", body)
    rates = (("inline code spans", len(code) * per_k, MAX_CODE),
             ("numbers set as code", len(numcode) * per_k, MAX_NUMCODE),
             ("PR or issue references", len(refs) * per_k, MAX_REFS))
    problems = [f"{name} {r:.1f} per 1,000 words, over {lim}" for name, r, lim in rates if r > lim]
    summary = ", ".join(f"{name} {r:.1f}/1k" for name, r, _ in rates) + f", hashes {len(hashes)}"
    return problems, summary


def title_of(text):
    """The post's front-matter title, or None."""
    m = re.search(r'^title: "(.*)"$', text, re.M)
    return m.group(1) if m else None


def title_problem(text):
    """A main post's title that lists three or more things (two commas or more)."""
    t = title_of(text)
    if t is None or "Companion to" in text:
        return None
    return f"the title lists {t.count(',') + 1} things; name one finding" if t.count(",") >= 2 else None


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
    out = subprocess.run(["git", "diff", "--name-only", "--diff-filter=d", f"{base}...HEAD", "--",
                          "docs/site/blog/"], capture_output=True, text=True, check=True)
    return [p for p in out.stdout.split() if p.endswith(".md") and not p.endswith("index.md")]


def main():
    argv = sys.argv[1:]
    show_all = "--all" in argv
    base = None
    if "--changed" in argv:
        base = argv[argv.index("--changed") + 1]
        files = changed_posts(base)
    else:
        files = [a for a in argv if not a.startswith("--")]
    if not files:
        print("no blog posts to check")
        return 0
    status = 0
    for path in files:
        status |= check(path, show_all, base)
    return status


def text_at(base, path):
    """The post's raw text at BASE, or None if the post is new there."""
    out = subprocess.run(["git", "show", f"{base}:{path}"], capture_output=True, text=True)
    return out.stdout if out.returncode == 0 else None


def dek_at(base, path):
    """The post's dek at BASE, or None if the post is new there."""
    old = text_at(base, path)
    return prose_blocks(old)[0] if old is not None else None


def check(path, show_all, base=None):
    text = pathlib.Path(path).read_text(encoding="utf-8")
    dek, blocks = prose_blocks(text)
    old_text = text_at(base, path) if base is not None else None
    old_dek = prose_blocks(old_text)[0] if old_text is not None else None
    old_title = title_of(old_text) if old_text is not None else None
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
        # With --changed, a dek is gated only if it is new or edited: a post touched for other
        # reasons (front matter, a fix) keeps a dek written before the rule, reported not gated.
        unchanged = base is not None and old_dek == dek
        ok = dek_ok or unchanged
        verdict = "PASS" if dek_ok else ("unchanged since " + base + ", reported, not gated" if unchanged else "FAIL")
        print(f"  dek: {verdict} ({why})")
    sh_problems, sh_summary = shorthand(text)
    print(f"  reader gate: {'FAIL' if sh_problems else 'PASS'} ({sh_summary}"
          + ("; " + "; ".join(sh_problems) if sh_problems else "") + ")")
    if sh_problems:
        print("    dense shorthand belongs in the lab notebook (docs/notebook/); a post is written for a reader outside the project")
        ok = False
    tp = title_problem(text)
    if tp:
        if old_title is not None and old_title == title_of(text):
            print(f"  title: {tp} (unchanged since {base}, reported, not gated)")
        else:
            print(f"  title: FAIL ({tp})")
            ok = False
    hl_issues = _headline_state_issues(text, path=path)
    body_issues = body_state_problems(text, path=path)
    if not hl_issues and not body_issues:
        print("  state gate: PASS")
    else:
        for where, msg in hl_issues:
            unchanged = (
                base is not None
                and (
                    (where == "title" and old_title is not None and old_title == title_of(text))
                    or (where == "dek" and old_dek is not None and old_dek == dek)
                )
            )
            if unchanged:
                print(f"  state gate ({where}): {msg} (unchanged since {base}, reported, not gated)")
            else:
                print(f"  state gate ({where}): FAIL ({msg})")
                ok = False
        for msg in body_issues:
            print(f"  state gate (body): FAIL ({msg})")
            ok = False
    worst = sorted(flagged, key=lambda x: (x[1], x[0]), reverse=True)
    for n, l, s in worst if show_all else worst[:8]:
        print(f"  [{n} words, load {l}] {s[:200]}{'…' if len(s) > 200 else ''}")
    return 0 if ok else 1


if __name__ == "__main__":
    import signal
    signal.signal(signal.SIGPIPE, signal.SIG_DFL)
    sys.exit(main())
