#!/usr/bin/env python3
"""Checks of bench/open_port3.py, open-port-3's verdict, on made-up bundles: every part complete
and as hoped (ADOPT); each part's failure (B1 and B2 pooled and flagged, B3's gain and memory,
B4, the guards' memory, Part A's unattributed trigger); each VOID, within and across files; the
statistics against known values; and the registration's own one attempt"""

import io
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
import open_port3 as o3  # noqa: E402
import test_open_port as t1  # noqa: E402

PORT_SHA = t1.PORT_SHA


def noise(seed, n, scale=0.01):
    """Symmetric noise for n rounds (n even): pairs of +e and -e, shuffled, mean exactly 0"""
    rng = random.Random(seed)
    half = [abs(rng.gauss(0, scale)) for _ in range(n // 2)]
    xs = [v for e in half for v in (e, -e)]
    rng.shuffle(xs)
    return xs


def write(path, rows):
    path.write_text("".join(json.dumps(r) + "\n" for r in rows))


GOOD_A = {
    "voids": [],
    "triggers": [
        {"corpus": "text", "threads": "t8", "event": "I1mr", "ratio": 1.04, "attribution": "layout"}
    ],
}


def build3(step=None, vectors=None, mem=None, edit=None, part_a=GOOD_A, drop=(), **kw):
    """A bundle of every part: `step` maps (corpus, t) to the port's true wall ratio (a 6% gain on
    the B3 corpora, none elsewhere, by default), `vectors` to a cell's exact log ratios, `mem` to its
    peak ratio (0.8 on B3, 1.0 elsewhere); `edit` edits a record (file, record) -> record or None"""
    step, vectors, mem = step or {}, vectors or {}, mem or {}
    d = t1.build(corpora=(), **kw)
    for f in ("runs.jsonl", "sustained.jsonl"):
        (d / f).unlink()
    seed = 0
    for name, corpora, counts, rounds, _ in o3.PARTS:
        rows = []
        for t in counts:
            for c in corpora:
                seed += 1
                benefit = c in o3.B3 or c == op.SUSTAINED
                xs = vectors.get((c, t)) or noise(seed, rounds)
                f = step.get((c, t), 0.94 if benefit else 1.0)
                m = mem.get((c, t), 0.8 if benefit else 1.0)
                for i, x in enumerate(xs):
                    base = 0.05
                    for r in (
                        t1.rec(c, t, t1.REF, i, base, 100000),
                        t1.rec(c, t, t1.PORT, i, base * f * math.exp(x), 100000 * m),
                    ):
                        r = edit(name, r) if edit else r
                        if r is not None:
                            rows.append(r)
        if name not in drop:
            write(d / name, rows)
    if part_a is not None:
        (d / "part-a.json").write_text(json.dumps(part_a) if isinstance(part_a, dict) else part_a)
    return d


def expect(name, d, code, phrases, absent=()):
    try:
        buf = io.StringIO()
        args = ["open_port3.py", "judge", str(d), "--port", PORT_SHA]
        if (d / "part-a.json").exists():
            args += ["--part-a", str(d / "part-a.json")]
        with redirect_stdout(buf):
            got = o3.main(args)
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


def guard_checks():
    """The registration's one attempt: its own evidence path, failing closed"""

    def git(cwd, *args):
        subprocess.run(["git", "-C", str(cwd), *args], check=True, capture_output=True)

    def run(target, this, remote, cwd):
        buf = io.StringIO()
        old = Path.cwd()
        os.chdir(cwd)
        try:
            with redirect_stdout(buf):
                got = o3.main(
                    ["open_port3.py", "guard", str(target), "--this", str(this), "--remote", remote]
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
        target = work / "target" / "open-port-3"
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
        (seed / "open-port-3").mkdir()
        (seed / "open-port-3" / "run-20261121T000000Z.tar.gz").write_bytes(b"x")
        git(seed, "add", "open-port-3")
        git(seed, *ident, "commit", "-q", "-m", "archived")
        git(seed, "push", "-q", str(origin), "evidence")
        out.append(
            check(
                "an open-port-3 attempt archived on the evidence branch: refuses",
                1,
                "already archived",
            )
        )
    finally:
        shutil.rmtree(root)
    return out


def stats_checks():
    out = []
    # Student's t quantiles against tables (df 15: 0.975 -> 2.1314, 0.99 -> 2.6025, 0.995 -> 2.9467)
    got = [o3.t_quantile(q, 15) for q in (0.975, 0.99, 0.995)]
    ok = all(abs(a - b) < 5e-4 for a, b in zip(got, (2.1314, 2.6025, 2.9467)))
    print(f"{'ok  ' if ok else 'FAIL'} t quantiles at df 15: {[round(g, 4) for g in got]}")
    out.append(ok)
    # the bootstrap: reproducible, centred, and its interval about a t-interval's on one cell
    cells = [noise(k, 16) for k in range(27)]
    a, b = o3.pooled(cells), o3.pooled(cells)
    ok = a == b and abs(a[0]) < 1e-12 and a[1] < 0 < a[2]
    print(f"{'ok  ' if ok else 'FAIL'} the pooled bootstrap is reproducible and centred: {a}")
    out.append(ok)
    xs = [x + 0.003 for x in noise(5, 16)]
    est, lo, hi = o3.pooled([xs], boot=20000)
    se = (sum((x - est) ** 2 for x in xs) / 15) ** 0.5 / 4
    ok = abs((hi - lo) / (2 * 1.96 * se) - 1) < 0.15
    print(
        f"{'ok  ' if ok else 'FAIL'} one cell's bootstrap interval is about ±1.96 se: [{lo:.5f}, {hi:.5f}], se {se:.5f}"
    )
    out.append(ok)
    return out


def main():
    res = []
    res.append(
        expect(
            "the port as hoped, guards with no effect: ADOPT",
            build3(),
            0,
            [
                "VERDICT: ADOPT",
                "b1.jsonl 864 of 864",
                "b2.jsonl 384 of 384",
                "b3.jsonl 144 of 144",
                "b4.jsonl 16 of 16",
                "Part A: 0 VOIDs; 1 triggers, 1 with a written attribution",
            ],
            absent=["VOID:", "FLAGGED", "NOT ADOPTED"],
        )
    )
    every_b1 = {(c, t): 1.015 for c in o3.B1 for t in o3.COUNTS}
    res.append(
        expect(
            "every zero-route cell truly 1.5% slower: B1's pooled bound fails",
            build3(step=every_b1),
            0,
            ["B1: the pooled upper bound", "NOT ADOPTED"],
            absent=["FLAGGED"],
        )
    )
    res.append(
        expect(
            "every admitted guard truly 1.5% slower: B2's pooled bound fails",
            build3(step={(c, t): 1.015 for c in o3.B2 for t in o3.COUNTS}),
            0,
            ["B2: the pooled upper bound", "NOT ADOPTED"],
        )
    )
    res.append(
        expect(
            "one zero-route cell truly 5% slower: flagged as a credible regression",
            build3(step={("ho-code-l9", 20): 1.05}),
            0,
            [
                "ho-code-l9 20t",
                "FLAGGED",
                "B1: a credible regression, ho-code-l9 at 20 threads",
                "NOT ADOPTED",
            ],
        )
    )
    res.append(
        expect(
            "one admitted guard truly 5% slower: flagged",
            build3(step={("ho-prose-l1", 8): 1.05}),
            0,
            ["B2: a credible regression, ho-prose-l1 at 8 threads", "NOT ADOPTED"],
        )
    )
    res.append(
        expect(
            "one zero-route cell 1.5% slower, noisy: neither pooled nor flagged (by design: 1% common, 2% single)",
            build3(step={("text", 10): 1.015}),
            0,
            ["VERDICT: ADOPT"],
            absent=["FLAGGED"],
        )
    )
    res.append(
        expect(
            "no gain on ho-wikidata-l9 at 20 threads: B3 fails",
            build3(step={("ho-wikidata-l9", 20): 1.0}),
            0,
            ["B3: no gain on ho-wikidata-l9 at 20 threads", "NOT ADOPTED"],
        )
    )
    res.append(
        expect(
            "more memory on wikidata-x4 at 10 threads: B3 fails",
            build3(mem={("wikidata-x4", 10): 1.01}),
            0,
            ["B3: peak RSS on wikidata-x4 at 10 threads 1.010", "NOT ADOPTED"],
        )
    )
    res.append(
        expect(
            "sustained truly 3% slower: B4 fails",
            build3(step={(op.SUSTAINED, 10): 1.03}),
            0,
            ["B4: sustained SLOWER", "NOT ADOPTED"],
        )
    )
    res.append(
        expect(
            "a zero-route guard's memory 1.2: the cap fails",
            build3(mem={("records", 20): 1.2}),
            0,
            ["memory: records at 20 threads peak RSS 1.200", "NOT ADOPTED"],
        )
    )
    res.append(
        expect(
            "an admitted guard's memory 1.2: the cap fails",
            build3(mem={("ho-structured-l9", 8): 1.2}),
            0,
            ["memory: ho-structured-l9 at 8 threads", "NOT ADOPTED"],
        )
    )
    bare = {
        "voids": [],
        "triggers": [{"corpus": "random", "threads": "t8", "event": "Bim", "ratio": 2.4}],
    }
    res.append(
        expect(
            "Part A: a trigger without a written attribution: NOT ADOPTED",
            build3(part_a=bare),
            0,
            ["Part A: an unattributed trigger: random at t8, Bim", "NOT ADOPTED"],
        )
    )
    blank = {
        "voids": [],
        "triggers": [
            {"corpus": "random", "threads": 8, "event": "Bim", "ratio": 2.4, "attribution": "  "}
        ],
    }
    res.append(
        expect(
            "Part A: a blank attribution: NOT ADOPTED",
            build3(part_a=blank),
            0,
            ["unattributed", "NOT ADOPTED"],
        )
    )
    other_port = t1.PORT.replace("/w/target/", "/elsewhere/target/")
    for name, d, phrase in [
        ("Part A not given", build3(part_a=None), "Part A: no summary given"),
        ("Part A unreadable", build3(part_a="{not json"), "Part A: "),
        (
            "Part A with a VOID (a block opened)",
            build3(part_a={"voids": ["text t8 #1: the open kernel ran"], "triggers": []}),
            "Part A VOID",
        ),
        ("b2.jsonl missing", build3(drop=("b2.jsonl",)), "b2.jsonl: missing"),
        (
            "text's digest different in b1 than... b1 holds it alone; ho-wikidata-l1's in b2 differs by round",
            build3(
                edit=lambda f, r: (
                    {**r, "sha256": "zz"}
                    if r["corpus"] == "ho-wikidata-l1" and r["round"] == 3
                    else r
                )
            ),
            "ho-wikidata-l1: the outputs differ",
        ),
        (
            "the port's binary another path in b3",
            build3(
                edit=lambda f, r: (
                    {**r, "arm": other_port} if f == "b3.jsonl" and r["arm"] == t1.PORT else r
                )
            ),
            "the port build's arm differs across files",
        ),
        (
            "a B1 corpus recorded in b2",
            build3(
                edit=lambda f, r: (
                    {**r, "corpus": "text"}
                    if f == "b2.jsonl" and r["corpus"] == "ho-prose-l1"
                    else r
                )
            ),
            "not a record of the roster",
        ),
        (
            "a round short in b4",
            build3(edit=lambda f, r: None if f == "b4.jsonl" and r["round"] == 7 else r),
            "records missing",
        ),
        (
            "ho-wikidata-l1 decoded by 6 workers at 10 threads",
            build3(
                edit=lambda f, r: (
                    {**r, "observed_threads": "ab mibz2-0 mibz2-1 mibz2-2 mibz2-3 mibz2-4 mibz2-5"}
                    if r["corpus"] == "ho-wikidata-l1" and r["threads"] == 10 and r["round"] == 2
                    else r
                )
            ),
            "observed workers",
        ),
        (
            "a NaN wall",
            build3(edit=lambda f, r: {**r, "wall": float("nan")} if r["round"] == 5 else r),
            "not a finite positive number",
        ),
        ("another port", build3(commits=f"port {'1' * 40}\nv0.5.6 {op.V056}\n"), "commits.txt"),
        ("an input off the manifest", build3(digests={"random.bz2": "5a" * 32}), "not the frozen"),
        ("not the i9", build3(lscpu="Model name: AMD EPYC\n"), "not the registered machine"),
    ]:
        res.append(expect(f"{name}: VOID", d, 1, ["VOID", phrase], absent=["VERDICT"]))
    d = build3()
    (d / "DRY-RUN.txt").write_text("dry\n")
    res.append(expect("a dry run: VOID", d, 1, ["VOID", "a dry run"], absent=["VERDICT"]))
    ok = sorted(o3.B1 + o3.B2 + o3.B3) == sorted(op.CORPORA) and len(o3.B1) == 9 and len(o3.B2) == 4
    print(
        f"{'ok  ' if ok else 'FAIL'} the parts: 9 zero-route, 4 admitted and 2 benefit corpora, the 15 of the manifest"
    )
    res.append(ok)
    res.extend(stats_checks())
    res.extend(guard_checks())
    failed = res.count(False)
    print(f"\n{len(res) - failed} of {len(res)} checks pass")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
