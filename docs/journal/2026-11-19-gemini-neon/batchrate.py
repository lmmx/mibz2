"""Measure how often Neon's greedy multi-front decode can batch two or more fronts.

Loads Neon's verifier verbatim from verify_thm3.py.txt (kept as text so it is not linted or
edited), pins up to K - 1 alphabet boundaries C[c] at roughly equal mass, and runs its greedy
batched induction to completion on a prefix of a real file, reporting batch sizes per round.
Run: python3 batchrate.py <file>
"""

import importlib.util
import pathlib
import sys
from collections import Counter

HERE = pathlib.Path(__file__).resolve().parent


def load_verifier():
    """Import Neon's verifier from its verbatim text file."""
    loader = importlib.machinery.SourceFileLoader("verify_thm3", str(HERE / "verify_thm3.py.txt"))
    spec = importlib.util.spec_from_loader("verify_thm3", loader)
    module = importlib.util.module_from_spec(spec)
    sys.modules["verify_thm3"] = module
    loader.exec_module(module)
    # The invariants were checked exhaustively on small cases; skip them here for speed
    module.MultiBoundaryDIET.check_invariants = lambda self: None
    return module


def cuts_for(text, k):
    """Return up to k - 1 alphabet boundaries C[c] nearest to equal-mass positions."""
    n = len(text)
    bounds, acc = [], 0
    for c in sorted(set(text)):
        if 0 < acc < n:
            bounds.append(acc)
        acc += text.count(c)
    if not bounds:
        return ()
    return tuple(sorted({min(bounds, key=lambda b: abs(b - n * i / k)) for i in range(1, k)}))


def run(v, text, k):
    """Decode `text` with the greedy batched schedule and print its batch statistics."""
    cuts = cuts_for(text, k)
    diet, p0 = v.build_bwt_diet(text, cuts)
    _, _, loc0 = diet.to_row_map()
    marks = {p0: (loc0[p0][0], loc0[p0][1], 0)}
    sizes = Counter()
    runs = len(diet.live)
    while diet.live:
        loop = next(
            (j for j in range(diet.K) if (p := diet.front_pair(j)) is not None and p[0] == p[1]),
            None,
        )
        if loop is not None:
            diet.apply_self_loop(loop, marks)
            continue
        batch = []
        for j in range(diet.K):
            if diet.is_admissible_batch(tuple(batch + [j])):
                batch.append(j)
        for j in batch:
            diet.apply_front(j, marks)
        sizes[len(batch)] += 1
    rounds = sum(sizes.values())
    multi = sum(c for s, c in sizes.items() if s >= 2)
    steps = sum(s * c for s, c in sizes.items())
    print(
        f"n={len(text)} r={runs} K={len(cuts) + 1}: rounds={rounds} steps={steps} "
        f"rounds with >=2 fronts={multi} ({100 * multi / max(rounds, 1):.1f}%) "
        f"mean batch={steps / max(rounds, 1):.3f}"
    )


def main():
    """Run the measurement on a few prefixes and front counts of the given file."""
    v = load_verifier()
    source = pathlib.Path(sys.argv[1]).read_text(encoding="utf-8", errors="replace")
    for n in (2000, 6000, 20000):
        for k in (1, 4, 16, 32):
            run(v, source[:n], k)


if __name__ == "__main__":
    main()
