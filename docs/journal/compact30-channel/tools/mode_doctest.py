"""Doctest for the MODE register, against the rules MODE.RATIONALE.md states.

Does each incantation obey them?

Utterances checked: the fenced block in MODE.md, and every blockquote (lines starting "> ")
in MODE.RATIONALE.md, each blank-line-free run of quoted lines being one paragraph and each
whole blockquote one utterance.

Surface conventions only (MODE.RATIONALE.md, "Checking"); conformance is a reading, not this.
Rules:
  R1 clipping consistency: an expanded form returning after its clipped form in one utterance.
     A preference, not a law: a hit is a question for the reader, not a verdict.
  R2 fervour, not violence: no violent or combative lexicon.
  R5 no bare doubled capitals ("HARDER HARDER").
  R4 article count: more than one per 25 words is flagged. The threshold is arbitrary and
     never a target; determiner deletion matters only against otherwise competent syntax.
Run on MODE.RATIONALE.md or MODE.ORIGIN.md, it flags the verbatim historical quotations.
Usage: python3 mode_doctest.py MODE.md
"""

import re
import sys

CLIP = {  # clipped form -> expanded form
    r"\bmath eng\b": r"\bmathematical engineering\b",
    r"\bimpl\b": r"\bimplementation\b",
    r"\bperf\b": r"\bperformance\b",
    r"\basm\b": r"\bassembly\b",
    r"\bdep chains?\b": r"\bdependency chains?\b",
}
VIOLENT = (
    r"\b(attack\w*|hunt\w*|kill\w*|strike\w*|destroy\w*|crush\w*|smash\w*|weapon\w*|fight\w*|war)\b"
)


def utterances(path):
    """Yield (name, text) for each utterance in a MODE file."""
    text = open(path).read()
    if path.endswith("MODE.md"):
        m = re.search(r"```\n(.*?)```", text, re.S)
        yield "MODE.md block", re.sub(r"\n+", "\n", m.group(1)).strip()
        return
    block, start = [], 0
    for i, line in enumerate(text.splitlines() + [""], 1):
        if line.startswith(">"):
            if not block:
                start = i
            block.append(line[1:].strip() or "\n")
        elif block:
            joined = re.sub(r" ?\n ?", "\n", " ".join(block)).strip()
            yield f"{path.split('/')[-1]}:{start}", joined
            block = []


def check(name, u):
    """Return the rule violations found in one utterance."""
    low = u.lower()
    out = []
    for c, e in CLIP.items():
        mc = re.search(c, low)
        if mc and re.search(e, low[mc.end() :]):
            out.append(f"R1 re-expands after clipping: {re.search(e, low[mc.end() :]).group(0)!r}")
    for m in re.finditer(VIOLENT, low):
        out.append(f"R2 violent lexicon: {m.group(0)!r}")
    for m in re.finditer(r"\b([A-Z]{2,})\b[\s.,!]+\1\b", u):
        out.append(f"R5 repeated capitals: {m.group(0)!r}")
    words = re.findall(r"[A-Za-z']+", u)
    arts = [w for w in words if w.lower() in ("a", "an", "the")]
    if words and len(arts) > max(1, len(words) // 25):
        out.append(f"R4 articles dense: {len(arts)} in {len(words)} words")
    return out


fails = 0
for p in sys.argv[1:]:
    for name, u in utterances(p):
        r = check(name, u)
        fails += bool(r)
        print(f"{'FAIL' if r else 'ok  '} {name}: {u.splitlines()[0][:60]!r}")
        for x in r:
            print("       " + x)
sys.exit(1 if fails else 0)
