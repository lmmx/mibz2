"""Adversarial checks of Neon's round-4 Theorem 1 and Corollary 1, and its Part 2 with a fast BWT.

Builds a variant of Neon's verbatim verifier (verify_boundary_records.py.txt) that adds a
"random" schedule policy, cross-checks a prefix-doubling BWT builder against Neon's rotation sort
on all small strings, then checks the exact work identity and the lower bound on 400 random and
repetitive BWTs under single, Max-Cut and random schedules. Run: python3 round4_adversarial.py
"""

import pathlib
import random
import runpy
import sys
import tempfile

HERE = pathlib.Path(__file__).resolve().parent
SRC = (HERE / "verify_boundary_records.py.txt").read_text()
SRC = SRC.replace("import random\n", "import random\nRNG = random.Random(12345)\n", 1)
SRC = SRC.replace(
    '            elif policy == "maxcut":',
    '            elif policy == "random":\n'
    "                if RNG.random() < 0.5 or chosen < 0:\n"
    "                    chosen = j\n"
    '            elif policy == "maxcut":',
    1,
)
with tempfile.NamedTemporaryFile("w", suffix=".py", delete=False) as fh:
    fh.write(SRC)
g = runpy.run_path(fh.name, run_name="lib")


def fast_bwt_lf(text):
    """BWT of the cyclic text by prefix doubling on rotations (stable ties by start index)."""
    n = len(text)
    rank = [ord(c) for c in text]
    sa = list(range(n))
    k = 1
    while True:
        key = lambda i: (rank[i], rank[(i + k) % n])
        sa.sort(key=lambda i: (key(i), i))
        new = [0] * n
        for a, b in zip(sa, sa[1:]):
            new[b] = new[a] + (key(a) != key(b))
        rank = new
        if rank[sa[-1]] == n - 1 or k >= n:
            break
        k *= 2
    # stable tie-break by start index already applied via (key, i)
    L = [text[(s - 1) % n] for s in sa]
    chars = sorted(set(L))
    C = {}
    acc = 0
    cb = []
    for ch in chars:
        if 0 < acc < n:
            cb.append(acc)
        C[ch] = acc
        acc += L.count(ch)
    seen = {ch: 0 for ch in chars}
    LF = [0] * n
    for i, ch in enumerate(L):
        LF[i] = C[ch] + seen[ch]
        seen[ch] += 1
    d = [0] + [i for i in range(1, n) if L[i] != L[i - 1]]
    return LF, d, cb


# cross-check fast builder against Neon's on all small strings
from itertools import product

for alph, ns in (("ab", range(2, 9)), ("abc", range(2, 6))):
    for n in ns:
        for t in product(alph, repeat=n):
            s = "".join(t)
            assert fast_bwt_lf(s) == g["build_bwt_lf"](s), s
print("fast builder agrees with Neon's on all small strings")

run, arcs, pred = g["run_mb_rv"], g["compute_arcs"], g["predict_from_arcs"]
rng = random.Random(7)
checked = 0
for trial in range(400):
    n = rng.randint(20, 400)
    sigma = rng.choice("ab abc abcd abcdefgh".split())
    s = "".join(rng.choice(sigma) for _ in range(n))
    if rng.random() < 0.5:  # repetitive, low r/n
        unit = s[: rng.randint(3, 12)]
        s = (unit * (n // len(unit) + 1))[:n]
        s = "".join(c if rng.random() > 0.05 else rng.choice(sigma) for c in s)
    T, d, cb = fast_bwt_lf(s)
    lb = 2 * len(d) - len(set(d) & {T[x] for x in d})
    for policy in ("single", "maxcut", "random", "random", "random"):
        cuts = () if policy == "single" else tuple(cb)
        for stop in (0, n // 4, n // 2):
            st, live, rd, el = run(T, d, cuts, policy, stop)
            assert pred(arcs(T, rd), el, st + 1) == (live, st), (s, policy, stop)
            if stop == 0:
                assert st >= lb, (s, policy, st, lb)
            checked += 1
print("adversarial random-schedule check:", checked, "runs, 0 mismatches")


def block(n, seed):
    """Return Neon's synthetic structured block."""
    return g["make_structured_block"](n, seed)


print("PART 2 (fast BWT builder, otherwise Neon's code):")
for seed in (1, 2, 3):
    text = block(20000, seed)
    n = len(text)
    T, d, cb = fast_bwt_lf(text)
    r = len(d)
    lb = 2 * r - len(set(d) & {T[x] for x in d})
    s_rv, _, rd1, e1 = run(T, d, (), "single", 0)
    s_mc, _, rd2, e2 = run(T, d, tuple(cb), "maxcut", 0)
    assert pred(arcs(T, rd1), e1, s_rv + 1) == (0, s_rv) and pred(arcs(T, rd2), e2, s_mc + 1) == (
        0,
        s_mc,
    )
    m = int(0.15 * n)
    s_rv_s = run(T, d, (), "single", m)[0]
    s_mc_s = run(T, d, tuple(cb), "maxcut", m)[0]
    print(
        f"  seed={seed} r/n={r / n:.3f} r={r} LB={lb} ({lb / r:.2f}r) full RV={s_rv} ({s_rv / r:.2f}r) "
        f"MaxCut={s_mc} ({s_mc / r:.2f}r) ratio={s_mc / s_rv:.3f} | stop@15%n RV={s_rv_s} MaxCut={s_mc_s} ratio={s_mc_s / s_rv_s:.3f}"
    )
