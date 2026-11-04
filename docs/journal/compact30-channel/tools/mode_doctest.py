"""Doctest for the MODE register: does each incantation obey the rules MODE.RATIONALE.md states?

Utterances checked: the fenced block in MODE.md, and every blockquote (lines starting "> ")
in MODE.RATIONALE.md, each blank-line-free run of quoted lines being one paragraph and each
whole blockquote one utterance.

Rules (from MODE.RATIONALE.md):
  R1 clipping is monotone: once a clipped form appears, its expanded form may not reappear
     later in the same utterance (an expanded form may open, as the unclipped baseline).
  R2 fervour, not violence: no violent or combative lexicon.
  R3 escalation does not fall before the release: the last intensifier before the final "Go"
     is not weaker than the strongest intensifier in the utterance.
  R5 no repeated capitals: an all-caps word is never immediately repeated ("HARDER HARDER");
     accent comes from a full stop and a slight variation or modifier ("HARDER. Harder still.").
  R4 determiner deletion: articles (a, an, the) are sparse — at most one per 25 words. Complete
     sentences may keep theirs: they are the articulate contrast that makes the fractures marked.
Usage: python3 mode_doctest.py MODE.md MODE.RATIONALE.md
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
VIOLENT = r"\b(attack\w*|hunt\w*|kill\w*|strike\w*|destroy\w*|crush\w*|smash\w*|weapon\w*|fight\w*|war)\b"
STRENGTH = [("HARDER", 3), ("HARD", 2)]


def utterances(path):
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


def strength(s):
    for k, v in STRENGTH:
        if s.upper().startswith(k) and (len(s) == len(k) or not s[len(k)].isalpha()):
            return v
    return 0


def check(name, u):
    low = u.lower()
    out = []
    for c, e in CLIP.items():
        mc = re.search(c, low)
        if mc and re.search(e, low[mc.end():]):
            out.append(f"R1 re-expands after clipping: {re.search(e, low[mc.end():]).group(0)!r}")
    for m in re.finditer(VIOLENT, low):
        out.append(f"R2 violent lexicon: {m.group(0)!r}")
    frags = [f.strip() for f in re.split(r"(?<=[.!?])\s+|\n", u) if f.strip()]
    ints = [(i, strength(f)) for i, f in enumerate(frags) if strength(f)]
    if ints and re.match(r"^go\b", frags[-1].strip(), re.I):
        peak = max(s for _, s in ints)
        last = ints[-1][1]
        if last < peak:
            out.append(f"R3 escalation falls before release: last {frags[ints[-1][0]]!r} < peak")
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
