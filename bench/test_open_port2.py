#!/usr/bin/env python3
"""Checks of bench/open_port2.py, the open port's successor verdict, on made-up bundles: the
four-file roster complete, the decision conditions as registered, G2 (the guards' exact Wilcoxon
rule) catching a real slowdown and passing noise, the exact p against enumeration, its size, each
VOID, and the successor's own one attempt"""

import io
import itertools
import json
import math
import os
import random
import shutil
import subprocess
import sys
import tempfile
from contextlib import redirect_stdout
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import open_port as op  # noqa: E402
import open_port2 as o2  # noqa: E402
import test_open_port as t1  # noqa: E402

PORT_SHA = t1.PORT_SHA


def noise(seed, n, scale=0.02, outliers=()):
    """Symmetric noise for n rounds: pairs of +e and -e in a shuffled order, so a cell with no
    effect has no tendency either way; `outliers` adds (round, value) spikes"""
    rng = random.Random(seed)
    half = [abs(rng.gauss(0, scale)) for _ in range((n + 1) // 2)]
    xs = [v for e in half for v in (e, -e)][:n]
    rng.shuffle(xs)
    for i, v in outliers:
        xs[i] += v
    return xs


def write(path, rows):
    path.write_text("".join(json.dumps(r) + "\n" for r in rows))


def cellrows(corpus, t, effect, xs, rss=100000, rss_ratio=1.0, round0=0):
    out = []
    for i, x in enumerate(xs):
        base = 0.05
        out.append(t1.rec(corpus, t, t1.REF, round0 + i, base, rss))
        out.append(
            t1.rec(
                corpus,
                t,
                t1.PORT,
                round0 + i,
                base * math.exp(math.log(effect) + x),
                rss * rss_ratio,
            )
        )
    return out


def build2(
    gstep=None,
    outl=None,
    sustained=0.97,
    sus_n=o2.SUSTAINED_ROUNDS,
    a_n=o2.EXTRA_A,
    b_n=o2.EXTRA_B,
    drop=(),
    aedit=None,
    **kw,
):
    """The first registration's decision bundle (test_open_port.build), with G2's files: every
    guard cell's extension rounds, and the sustained cell at its new count. `gstep` maps
    (corpus, t) to the port's true wall ratio on that guard (applied to the decision rounds too);
    `outl` maps (corpus, t) to outlier spikes in its extension rounds."""
    gstep, outl = gstep or {}, outl or {}
    step = dict(kw.pop("step", {}) or {})
    for k, v in gstep.items():
        step[k] = v
    d = t1.build(step=step, **kw)
    seed = 0
    for name, tier, n in (("guards-a.jsonl", o2.TIER_A, a_n), ("guards-b.jsonl", o2.TIER_B, b_n)):
        rows = []
        for t in op.COUNTS:
            for c in tier:
                seed += 1
                xs = noise(seed, n, outliers=outl.get((c, t), ()))
                for r in cellrows(c, t, gstep.get((c, t), 1.0), xs):
                    r = aedit(r) if aedit else r
                    if r is not None:
                        rows.append(r)
        if name not in drop:
            write(d / name, rows)
    srows = []
    xs = noise(99, sus_n, scale=0.01)
    for i, x in enumerate(xs):
        base = 1.6
        srows.append(t1.rec(op.SUSTAINED, 10, t1.REF, i, base, 120000))
        srows.append(t1.rec(op.SUSTAINED, 10, t1.PORT, i, base * sustained * math.exp(x), 84000))
    if "sustained.jsonl" not in drop:
        write(d / "sustained.jsonl", srows)
    else:
        (d / "sustained.jsonl").unlink()
    return d


def expect(name, d, code, phrases, absent=()):
    try:
        buf = io.StringIO()
        with redirect_stdout(buf):
            got = o2.main(["open_port2.py", "judge", str(d), "--port", PORT_SHA])
        out = buf.getvalue()
    finally:
        shutil.rmtree(d)
    ok = got == code and all(p in out for p in phrases) and not any(p in out for p in absent)
    print(
        f"{'ok  ' if ok else 'FAIL'} {name}: exit {got}, "
        + ", ".join(
            [f"{'found' if p in out else 'missing'} {p!r}" for p in phrases]
            + [f"{'present' if p in out else 'absent'} {p!r}" for p in absent]
        )
    )
    if not ok:
        print(out)
    return ok


def brute(xs):
    """The exact p by enumerating every sign assignment (small n only)"""
    nz = [x for x in xs if x != 0]
    mags = [abs(x) for x in nz]
    srt = sorted(mags)
    rank = {m: (srt.index(m) + 1 + len(srt) - srt[::-1].index(m)) / 2 for m in set(mags)}
    obs = sum(rank[abs(x)] for x in nz if x > 0)
    hits = sum(
        sum(rank[m] for m, s in zip(mags, signs) if s) >= obs - 1e-9
        for signs in itertools.product((0, 1), repeat=len(nz))
    )
    return hits / 2 ** len(nz) if nz else 1.0


def guard_checks():
    """The successor's one attempt: its own evidence path, failing closed"""

    def git(cwd, *args):
        subprocess.run(["git", "-C", str(cwd), *args], check=True, capture_output=True)

    def run(target, this, remote, cwd):
        buf = io.StringIO()
        old = Path.cwd()
        os.chdir(cwd)
        try:
            with redirect_stdout(buf):
                got = o2.main(
                    ["open_port2.py", "guard", str(target), "--this", str(this), "--remote", remote]
                )
        finally:
            os.chdir(old)
        return got, buf.getvalue()

    root = Path(tempfile.mkdtemp())
    out = []
    try:
        ident = ["-c", "user.name=t", "-c", "user.email=t@t"]
        origin = root / "origin.git"
        git(root, "init", "-q", "--bare", str(origin))
        seed = root / "seed"
        git(root, "init", "-q", str(seed))
        git(seed, "checkout", "-q", "--orphan", "evidence")
        (seed / "open-port").mkdir()
        (seed / "open-port" / "run-20261119T125538Z.tar.gz").write_bytes(b"first")
        git(seed, "add", "open-port")
        git(seed, *ident, "commit", "-q", "-m", "the first registration's attempt")
        git(seed, "push", "-q", str(origin), "evidence")
        work = root / "work"
        git(root, "clone", "-q", str(origin), str(work))
        target = work / "target" / "open-port-2"
        this = target / "run-20261120T000000Z"
        this.mkdir(parents=True)
        (this / "timing-started.txt").write_text("now\n")

        def check(name, want, phrase, remote="origin"):
            got, text = run(target, this, remote, work)
            ok = got == want and phrase in text
            print(
                f"{'ok  ' if ok else 'FAIL'} guard: {name}: exit {got}"
                + ("" if ok else f"\n{text}")
            )
            return ok

        out.append(
            check(
                "the first registration's open-port/ bundle is not this one's attempt: passes",
                0,
                "",
            )
        )
        out.append(
            check("an unreadable evidence branch: refuses", 1, "can't be fetched", remote="nowhere")
        )
        timed = target / "run-20261119T200000Z"
        timed.mkdir()
        (timed / "timing-started.txt").write_text("then\n")
        out.append(check("a timed attempt started here: refuses", 1, "already started here"))
        shutil.rmtree(timed)
        (seed / "open-port-2").mkdir()
        (seed / "open-port-2" / "run-20261121T000000Z.tar.gz").write_bytes(b"x")
        git(seed, "add", "open-port-2")
        git(seed, *ident, "commit", "-q", "-m", "archived")
        git(seed, "push", "-q", str(origin), "evidence")
        out.append(
            check(
                "an open-port-2 attempt archived on the evidence branch: refuses",
                1,
                "already archived",
            )
        )
    finally:
        shutil.rmtree(root)
    return out


def main():
    res = []
    res.append(
        expect(
            "the port as hoped, guards with no effect: ADOPTS",
            build2(),
            0,
            [
                "VERDICT: ADOPTS",
                f"records {15 * 3 * 2 * 12} of",
                f"{4 * 3 * 2 * 108} of",
                f"{9 * 3 * 2 * 48} of",
                "80 of 80 (sustained)",
                "120 rounds",
                "60 rounds",
            ],
            absent=["VOID", "FAILS"],
        )
    )
    # G2 catches a real slowdown on one guard cell, in either tier
    for c in ("ho-prose-l1", "text"):
        res.append(
            expect(
                f"{c} at 10 threads truly 3% slower: G2 fails it",
                build2(gstep={(c, 10): 1.03}),
                0,
                [f"G2: {c} at 10 threads slower", "VERDICT: NOT ADOPTED"],
                absent=["VERDICT: ADOPTS"],
            )
        )
    res.append(
        expect(
            "a guard truly 2% slower, in its noise: G2 fails it",
            build2(gstep={("random", 8): 1.02}),
            0,
            ["G2: random at 8 threads slower", "NOT ADOPTED"],
        )
    )
    # noise with outliers, the shape of the first attempt's failing cells, is no slowdown
    res.append(
        expect(
            "a guard with no effect but upward spikes (geometric mean above 1.02): G2 passes",
            build2(outl={("ho-code-l1", 10): [(0, 0.29), (5, 0.25), (9, 0.3), (20, 0.28)]}),
            0,
            ["VERDICT: ADOPTS"],
            absent=["G2: ho-code-l1"],
        )
    )
    res.append(
        expect(
            "the sustained cell truly 3% slower: G2 fails it",
            build2(sustained=1.03),
            0,
            ["G2: sustained slower", "NOT ADOPTED"],
        )
    )
    # the decision conditions, as registered
    res.append(
        expect(
            "no gain at 20 threads: P1 fails",
            build2(step={("wikidata-x4", 20): 1.0}),
            0,
            ["P1: no gain on wikidata-x4 at 20 threads", "NOT ADOPTED"],
        )
    )
    res.append(
        expect(
            "Wikidata memory above 1.00: P2 fails",
            build2(mem={("wikidata-x4", 8): 1.01}),
            0,
            ["P2: peak RSS on wikidata-x4 at 8 threads", "NOT ADOPTED"],
        )
    )
    res.append(
        expect(
            "no held-out replication: H fails",
            build2(step={("ho-wikidata-l9", 10): 1.0}),
            0,
            ["H: ho-wikidata-l9 at 10 threads", "NOT ADOPTED"],
        )
    )
    res.append(
        expect(
            "a guard's memory above 1.10: G fails",
            build2(mem={("records", 20): 1.2}),
            0,
            ["G: records at 20 threads peak RSS 1.200, above 1.1", "NOT ADOPTED"],
        )
    )
    # VOID: the four files' rosters
    for name, d, phrase in [
        ("guards-a.jsonl missing", build2(drop=("guards-a.jsonl",)), "guards-a.jsonl: missing"),
        ("guards-b.jsonl missing", build2(drop=("guards-b.jsonl",)), "guards-b.jsonl: missing"),
        ("tier A short by a round", build2(a_n=o2.EXTRA_A - 1), "records missing"),
        ("tier B with an extra round", build2(b_n=o2.EXTRA_B + 1), "not a record of the roster"),
        (
            "the sustained cell at the first registration's 8 rounds",
            build2(sus_n=8),
            "records missing",
        ),
        (
            "a tier B corpus in the tier A file",
            build2(aedit=lambda r: {**r, "corpus": "text"} if r["corpus"] == "runs" else r),
            "not a record of the roster",
        ),
        (
            "an extension record at threads 8.5",
            build2(
                aedit=lambda r: (
                    {**r, "threads": 8.5} if r["round"] == 3 and r["threads"] == 8 else r
                )
            ),
            "threads 8.5",
        ),
        (
            "an extension record with a NaN wall",
            build2(aedit=lambda r: {**r, "wall": float("nan")} if r["round"] == 7 else r),
            "not a finite positive number",
        ),
        ("another port", build2(commits=f"port {'1' * 40}\nv0.5.6 {op.V056}\n"), "commits.txt"),
        ("an input off the manifest", build2(digests={"text.bz2": "5a" * 32}), "not the frozen"),
    ]:
        res.append(expect(f"{name}: VOID", d, 1, ["VOID", phrase], absent=["VERDICT"]))
    d = build2()
    (d / "DRY-RUN.txt").write_text("dry\n")
    res.append(expect("a dry run: VOID", d, 1, ["VOID", "a dry run"], absent=["VERDICT"]))
    # the exact p, against enumeration, with ties and zeros
    rng = random.Random(5)
    ok = True
    for _ in range(60):
        n = rng.randint(1, 12)
        xs = [rng.choice((-3, -2, -1, 0, 1, 2, 3)) * 0.01 for _ in range(n)]
        if abs(o2.signed_rank_p(xs) - brute(xs)) > 1e-12:
            ok = False
            print("   ", xs, o2.signed_rank_p(xs), brute(xs))
    print(
        f"{'ok  ' if ok else 'FAIL'} the exact signed-rank p equals enumeration on 60 small samples with ties and zeros"
    )
    res.append(ok)
    # its size: with signs at random (no effect), P(p <= alpha) <= alpha
    rng = random.Random(9)
    for n, alpha in ((20, 0.05), (40, 0.01)):
        mags = [abs(rng.gauss(0, 1)) for _ in range(n)]
        hits = sum(
            o2.signed_rank_p([m * rng.choice((-1, 1)) for m in mags]) <= alpha for _ in range(3000)
        )
        ok = hits / 3000 <= alpha + 3 * math.sqrt(alpha * (1 - alpha) / 3000)
        print(
            f"{'ok  ' if ok else 'FAIL'} the test's size at n {n}: {hits / 3000:.4f} at α {alpha}"
        )
        res.append(ok)
    ok = (
        o2.FAMILY == 40
        and abs(o2.ALPHA2 - 0.0012) < 1e-12
        and set(o2.TIER_A) | set(o2.TIER_B) == set(o2.GUARDS)
    )
    print(
        f"{'ok  ' if ok else 'FAIL'} the family: 39 guard cells and the sustained cell at α 0.048/40; the tiers cover the 13 guards"
    )
    res.append(ok)
    res.extend(guard_checks())
    failed = res.count(False)
    print(f"\n{len(res) - failed} of {len(res)} checks pass")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
