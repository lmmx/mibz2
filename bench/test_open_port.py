#!/usr/bin/env python3
"""Checks of bench/open_port.py, the open port's adoption verdict, on made-up bundles: the
registered roster complete, then each condition failing, each VOID, and the Researcher's
counterexamples (REVIEW 7007213171) as regressions"""

import io
import json
import os
import shutil
import sys
import tempfile
from contextlib import redirect_stdout
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import open_port as op  # noqa: E402

PORT_SHA = "9743bbe0d51ba451b8b8ab9e6fd731c238940052"
REF = "mibz2@/w/target/open-port/build-v0.5.6/target/release/examples/ab"
PORT = "mibz2@/w/target/open-port/build-port/target/release/examples/ab"
I9 = "Model name:                           Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz\n"
ENV = {"cores": 10, "cpus": 20}


def rec(corpus, t, arm, i, wall, rss, workers=None):
    w = op.WORKERS.get(t, t) if workers is None else workers
    return {
        "round": i,
        "corpus": corpus,
        "threads": t,
        "arm": arm,
        "termination": "success",
        "wall": wall,
        "rss_kb": rss,
        "out": 1000,
        "sha256": corpus[:2],
        "observed_threads": " ".join(
            ["ab", *(f"mibz2-{k}" for k in range(w)), "mibz2-read", "mibz2-scan", "sampler"]
        ),
    }


def build(
    step=None,
    mem=None,
    sustained=0.97,
    edit=None,
    sedit=None,
    lscpu=I9,
    env=ENV,
    corpora=op.CORPORA,
    commits=None,
    digests=None,
):
    """A bundle of the registered roster: `step` and `mem` map (corpus, t) to the port's wall and
    peak over v0.5.6's (by default a 6% Wikidata gain, held-out too, at 0.7 of the memory, and
    level guards)"""
    step, mem = step or {}, mem or {}
    d = Path(tempfile.mkdtemp())
    rows = []
    for t in op.COUNTS:
        for i in range(op.ROUNDS):
            for c in corpora:
                base = 0.5 * (1 + 0.01 * ((i * 7 + t) % 5 - 2))
                wig = 1 + 0.005 * (1 if i % 2 else -1)
                f = step.get((c, t), 0.94 if "wikidata" in c else 1.0) * wig
                m = mem.get((c, t), 0.7 if "wikidata" in c else 1.0)
                for r in (
                    rec(c, t, REF, i, base, 100000),
                    rec(c, t, PORT, i, base * f, 100000 * m),
                ):
                    r = edit(r) if edit else r
                    if r is not None:
                        rows.append(r)
    srows = []
    for i in range(op.SUSTAINED_ROUNDS):
        base = 1.6 * (1 + 0.01 * (i % 3 - 1))
        wig = 1 + 0.004 * (1 if i % 2 else -1)
        for r in (
            rec(op.SUSTAINED, 10, REF, i, base, 120000),
            rec(op.SUSTAINED, 10, PORT, i, base * sustained * wig, 84000),
        ):
            r = sedit(r) if sedit else r
            if r is not None:
                srows.append(r)
    (d / "runs.jsonl").write_text("".join(json.dumps(r) + "\n" for r in rows))
    (d / "sustained.jsonl").write_text("".join(json.dumps(r) + "\n" for r in srows))
    if lscpu is not None:
        (d / "lscpu.txt").write_text(lscpu)
    (d / "env.json").write_text(json.dumps(env))
    (d / "commits.txt").write_text(
        commits if commits is not None else f"port {PORT_SHA}\nv0.5.6 {op.V056}\n"
    )
    # the receipt: a well-formed digest for every registered input, the frozen ones as frozen;
    # `digests` overrides entries (None drops one), or is the receipt's whole text if a str
    if isinstance(digests, str):
        (d / "inputs.sha256").write_text(digests)
    else:
        good = {n: "0" * 63 + str(k % 10) for k, n in enumerate(op.INPUTS)}
        good.update(op.FROZEN)
        good.update(digests or {})
        (d / "inputs.sha256").write_text(
            "".join(f"{v}  /w/c/{k}\n" for k, v in good.items() if v is not None)
        )
    return d


def expect(name, d, code, phrases, absent=()):
    try:
        buf = io.StringIO()
        with redirect_stdout(buf):
            got = op.main(["open_port.py", "judge", str(d), "--port", PORT_SHA])
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
    """The one-attempt guard (`open_port.py guard`) against scratch repositories: it refuses a
    timed attempt started here, an attempt archived on the evidence branch, and (the Researcher's
    second correction) an evidence branch it can't fetch or read; it passes only a readable
    branch with no archived attempt, whatever dry runs lie here"""
    import subprocess

    def git(cwd, *args):
        subprocess.run(["git", "-C", str(cwd), *args], check=True, capture_output=True)

    def run(target, this, remote="origin", cwd=None):
        buf = io.StringIO()
        old = Path.cwd()
        os.chdir(cwd)
        try:
            with redirect_stdout(buf):
                got = op.main(
                    ["open_port.py", "guard", str(target), "--this", str(this), "--remote", remote]
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
        (seed / "README").write_text("evidence\n")
        git(seed, "checkout", "-q", "--orphan", "evidence")
        git(seed, "add", "README")
        git(seed, *ident, "commit", "-q", "-m", "evidence")
        git(seed, "push", "-q", str(origin), "evidence")
        work = root / "work"
        git(root, "clone", "-q", str(origin), str(work))
        target = work / "target" / "open-port"
        this = target / "run-20261120T000000Z"
        this.mkdir(parents=True)
        (this / "started.txt").write_text("now\n")
        dry = target / "run-20261119T111634Z"
        dry.mkdir()
        (dry / "started.txt").write_text("then\n")
        (dry / "DRY-RUN.txt").write_text("dry\n")

        def check(name, want, phrase, **kw):
            got, text = run(target, this, cwd=work, **kw)
            ok = got == want and phrase in text
            print(
                f"{'ok  ' if ok else 'FAIL'} guard: {name}: exit {got}"
                + ("" if ok else f"\n{text}")
            )
            return ok

        out.append(check("a readable evidence branch, no attempt, a dry run here: passes", 0, ""))
        out.append(
            check(
                "Researcher 8: no local marker and an unreadable evidence branch: refuses",
                1,
                "can't be fetched",
                remote="nowhere",
            )
        )
        # a stale local copy of a clean evidence branch must not stand in for a fetch that fails
        git(work, "remote", "add", "stale", str(root / "gone.git"))
        git(work, "update-ref", "refs/remotes/stale/evidence", "refs/remotes/origin/evidence")
        out.append(
            check(
                "an unreachable remote, a stale clean copy of its branch here: refuses",
                1,
                "can't be fetched",
                remote="stale",
            )
        )
        git(work, "remote", "add", "empty", str(root / "empty.git"))
        git(root, "init", "-q", "--bare", str(root / "empty.git"))
        out.append(
            check(
                "a remote with no evidence branch: refuses", 1, "can't be fetched", remote="empty"
            )
        )
        setup = target / "run-20261119T190000Z"
        setup.mkdir()
        (setup / "started.txt").write_text("then\n")
        out.append(check("a run stopped in its untimed setup here: passes", 0, ""))
        timed = target / "run-20261119T200000Z"
        timed.mkdir()
        (timed / "started.txt").write_text("then\n")
        (timed / "timing-started.txt").write_text("then\n")
        out.append(check("a timed attempt started here: refuses", 1, "already started here"))
        shutil.rmtree(timed)
        (seed / "open-port").mkdir()
        (seed / "open-port" / "run-20261121T000000Z.tar.gz").write_bytes(b"x")
        git(seed, "add", "open-port")
        git(seed, *ident, "commit", "-q", "-m", "archived")
        git(seed, "push", "-q", str(origin), "evidence")
        out.append(
            check("an attempt archived on the evidence branch: refuses", 1, "already archived")
        )
    finally:
        shutil.rmtree(root)
    return out


def main():
    res = []
    nan = float("nan")
    res.append(
        expect(
            "the port as hoped: ADOPTS",
            build(),
            0,
            [
                "records 1080 of 1080 and 16 of 16 sustained",
                "the machine: Intel(R) Core(TM) i9-10900K",
                "wikidata-x4 (primary): 0.94",
                "ho-wikidata-l9 (held-out replication)",
                "text (guard): 1.00",
                "ho-structured-l9 (guard)",
                "Holm at α 0.05 over 3 cells",
                "VERDICT: ADOPTS",
            ],
            absent=["VOID", "NOT ADOPTED"],
        )
    )
    # each condition failing
    res.append(
        expect(
            "no gain at 20 threads: P1 fails, NOT ADOPTED",
            build(step={("wikidata-x4", 20): 1.0}),
            0,
            ["P1: no gain on wikidata-x4 at 20 threads", "VERDICT: NOT ADOPTED"],
        )
    )
    res.append(
        expect(
            "Wikidata memory above 1.00: P2 fails",
            build(mem={("wikidata-x4", 8): 1.01}),
            0,
            ["P2: peak RSS on wikidata-x4 at 8 threads 1.010, above 1.00", "NOT ADOPTED"],
        )
    )
    res.append(
        expect(
            "the gain does not replicate on held-out Wikidata: H fails",
            build(step={("ho-wikidata-l9", 10): 1.0}),
            0,
            ["H: ho-wikidata-l9 at 10 threads", "not FASTER"],
        )
    )
    res.append(
        expect(
            "text SLOWER: G fails",
            build(step={("text", 8): 1.05}),
            0,
            ["G: text at 8 threads SLOWER", "NOT ADOPTED"],
            absent=["VERDICT: ADOPTS"],
        )
    )
    res.append(
        expect(
            "a held-out guard above its bound, within noise: G fails",
            build(
                step={("ho-prose-l1", 10): 1.025},
                edit=lambda r: (
                    {**r, "wall": r["wall"] * (1 + 0.06 * (1 if r["round"] % 2 else -1))}
                    if r["arm"] == PORT and r["corpus"] == "ho-prose-l1" and r["threads"] == 10
                    else r
                ),
            ),
            0,
            ["G: ho-prose-l1 at 10 threads 1.02", "above the bound 1.02", "NOT ADOPTED"],
        )
    )
    res.append(
        expect(
            "a guard's memory above 1.10: G fails",
            build(mem={("random", 20): 1.2}),
            0,
            ["G: random at 20 threads peak RSS 1.200, above 1.1", "NOT ADOPTED"],
        )
    )
    res.append(
        expect(
            "sustained SLOWER: S fails",
            build(sustained=1.05),
            0,
            ["S: sustained SLOWER", "NOT ADOPTED"],
        )
    )
    res.append(
        expect(
            "every failure named",
            build(
                step={("wikidata-x4", 8): 1.0, ("text", 8): 1.06},
                mem={("wikidata-x4", 10): 1.05},
                sustained=1.05,
            ),
            0,
            [
                "4 conditions fail",
                "P1: no gain on wikidata-x4 at 8 threads",
                "P2: peak RSS on wikidata-x4 at 10",
                "G: text at 8 threads SLOWER",
                "S: sustained SLOWER",
            ],
        )
    )
    # the Researcher's counterexamples (REVIEW 7007213171), now regressions
    res.append(
        expect(
            "Researcher 1: only the primary and the held-out replication, no guard at all: VOID",
            build(corpora=("wikidata-x4", "ho-wikidata-l9")),
            1,
            ["VOID", "records missing", "opens no retry"],
            absent=["VERDICT"],
        )
    )
    res.append(
        expect(
            "Researcher 2: a NaN peak RSS on the primary: VOID",
            build(
                edit=lambda r: (
                    {**r, "rss_kb": nan}
                    if r["arm"] == PORT and r["corpus"] == "wikidata-x4" and r["round"] == 0
                    else r
                )
            ),
            1,
            ["VOID", "not a finite positive number"],
            absent=["VERDICT"],
        )
    )
    res.append(
        expect(
            "Researcher 3: an empty sustained file: VOID, not a crash",
            build(sedit=lambda r: None),
            1,
            ["VOID", "sustained.jsonl: no records"],
            absent=["VERDICT"],
        )
    )
    # the roster, exactly
    res.append(
        expect(
            "text missing at one count: VOID",
            build(edit=lambda r: None if r["corpus"] == "text" and r["threads"] == 20 else r),
            1,
            ["VOID", "records missing"],
        )
    )
    res.append(
        expect(
            "one guard absent everywhere: VOID",
            build(corpora=tuple(c for c in op.CORPORA if c != "onebyte")),
            1,
            ["VOID", "records missing"],
        )
    )
    res.append(
        expect(
            "a record outside the roster (a 13th round): VOID",
            build(
                edit=lambda r: (
                    {**r, "round": 12}
                    if r["corpus"] == "text" and r["round"] == 11 and r["threads"] == 8
                    else r
                )
            ),
            1,
            ["VOID", "not a record of the roster"],
        )
    )
    res.append(
        expect(
            "a corpus not registered: VOID",
            build(corpora=(*op.CORPORA, "extra")),
            1,
            ["VOID", "not a record of the roster"],
        )
    )
    res.append(
        expect(
            "a sustained round missing: VOID",
            build(sedit=lambda r: None if r["round"] == 7 and r["arm"] == PORT else r),
            1,
            ["VOID", "records missing"],
        )
    )
    res.append(
        expect(
            "a NaN wall: VOID",
            build(
                edit=lambda r: (
                    {**r, "wall": nan} if r["corpus"] == "runs" and r["round"] == 3 else r
                )
            ),
            1,
            ["VOID", "not a finite positive number"],
        )
    )
    res.append(
        expect(
            "an infinite wall: VOID",
            build(
                edit=lambda r: (
                    {**r, "wall": float("inf")}
                    if r["corpus"] == "pattern" and r["round"] == 3
                    else r
                )
            ),
            1,
            ["VOID", "not a finite positive number"],
        )
    )
    res.append(
        expect(
            "a zero peak RSS: VOID",
            build(sedit=lambda r: {**r, "rss_kb": 0} if r["round"] == 1 else r),
            1,
            ["VOID", "not a finite positive number"],
        )
    )
    res.append(
        expect(
            "a record with no digest: VOID",
            build(
                edit=lambda r: (
                    {k: v for k, v in r.items() if k != "sha256"}
                    if r["corpus"] == "records" and r["round"] == 0
                    else r
                )
            ),
            1,
            ["VOID", "no output size or digest"],
        )
    )
    # identities
    res.append(
        expect(
            "another port commit: VOID",
            build(commits=f"port {'1' * 40}\nv0.5.6 {op.V056}\n"),
            1,
            ["VOID", "commits.txt: the builds are"],
        )
    )
    res.append(
        expect(
            "another reference commit: VOID",
            build(commits=f"port {PORT_SHA}\nv0.5.6 {'2' * 40}\n"),
            1,
            ["VOID", "commits.txt"],
        )
    )
    res.append(expect("no commits.txt: VOID", build(commits=""), 1, ["VOID", "commits.txt"]))
    res.append(
        expect(
            "another Wikidata sample: VOID",
            build(digests={"wikidata.bz2": "ab" * 32}),
            1,
            ["VOID", "wikidata.bz2 is 'abab", "not the frozen 82f65ace"],
        )
    )
    res.append(
        expect(
            "other held-out windows: VOID",
            build(digests={"wikidata-heldout.bz2": "cd" * 32}),
            1,
            ["VOID", "wikidata-heldout.bz2 is 'cdcd", "not the frozen 7f5c9a00"],
        )
    )
    res.append(
        expect(
            "a corpus with no input digest: VOID",
            build(digests={"ho-code-l9.bz2": None}),
            1,
            ["VOID", "no digest for ['ho-code-l9.bz2']"],
        )
    )
    # the machine, the arms, the workers, the outputs
    res.append(
        expect(
            "another CPU: VOID",
            build(lscpu="Model name: Intel(R) Xeon(R) Processor @ 2.10GHz\n"),
            1,
            ["VOID", "not the registered machine"],
            absent=["VERDICT"],
        )
    )
    res.append(expect("no lscpu: VOID", build(lscpu=None), 1, ["VOID", "not one i9-10900K"]))
    res.append(
        expect(
            "another topology: VOID",
            build(env={"cores": 4, "cpus": 4}),
            1,
            ["VOID", "4 cores, 4 CPUs, not 10 and 20"],
        )
    )
    res.append(
        expect(
            "the port observing 10 workers at 10 threads: VOID",
            build(
                edit=lambda r: (
                    rec(r["corpus"], 10, PORT, r["round"], r["wall"], r["rss_kb"], workers=10)
                    if r["arm"] == PORT
                    and r["threads"] == 10
                    and r["round"] == 3
                    and r["corpus"] == "wikidata-x4"
                    else r
                )
            ),
            1,
            ["VOID", "not the shipped 7 at 10 threads"],
        )
    )
    res.append(
        expect(
            "v0.5.6 observing 7 workers at 8 threads: VOID",
            build(
                edit=lambda r: (
                    rec(r["corpus"], 8, REF, r["round"], r["wall"], r["rss_kb"], workers=7)
                    if r["arm"] == REF
                    and r["threads"] == 8
                    and r["round"] == 0
                    and r["corpus"] == "ho-wikidata-l9"
                    else r
                )
            ),
            1,
            ["VOID", "not the shipped 8 at 8 threads"],
        )
    )
    res.append(
        expect(
            "a guard's short decode seeing fewer workers: no VOID (read on Wikidata alone)",
            build(
                edit=lambda r: (
                    rec(
                        r["corpus"],
                        r["threads"],
                        r["arm"],
                        r["round"],
                        r["wall"],
                        r["rss_kb"],
                        workers=2,
                    )
                    if r["corpus"] == "random"
                    else r
                )
            ),
            0,
            ["VERDICT: ADOPTS"],
            absent=["VOID"],
        )
    )
    res.append(
        expect(
            "a failed decode: VOID",
            build(
                edit=lambda r: (
                    {**r, "termination": "exit 1"}
                    if r["corpus"] == "text" and r["round"] == 2
                    else r
                )
            ),
            1,
            ["VOID", "failed"],
        )
    )
    res.append(
        expect(
            "outputs differ: VOID",
            build(
                edit=lambda r: (
                    {**r, "sha256": "zz"}
                    if r["corpus"] == "text" and r["round"] == 5 and r["arm"] == PORT
                    else r
                )
            ),
            1,
            ["VOID", "text: the outputs differ"],
        )
    )
    res.append(
        expect(
            "a third arm: VOID",
            build(
                edit=lambda r: (
                    {**r, "arm": "lbzip2"}
                    if r["corpus"] == "text" and r["round"] == 1 and r["arm"] == REF
                    else r
                )
            ),
            1,
            ["VOID", "is neither the port's build"],
        )
    )
    res.append(
        expect(
            "two reference binaries: VOID",
            build(
                edit=lambda r: (
                    {**r, "arm": REF + "2"}
                    if r["corpus"] == "text" and r["round"] == 1 and r["arm"] == REF
                    else r
                )
            ),
            1,
            ["VOID", "more than one reference binary"],
        )
    )
    # the Researcher's second counterexamples (REVIEW 7008014645), now regressions
    res.append(
        expect(
            "Researcher 4: `not-a-digest` for text.bz2: VOID",
            build(digests={"text.bz2": "not-a-digest"}),
            1,
            ["VOID", "not a SHA-256 for ['text.bz2']"],
            absent=["VERDICT"],
        )
    )
    res.append(
        expect(
            "Researcher 5: the judged held-out copy another digest, its base frozen: VOID",
            build(digests={"ho-wikidata-l9.bz2": "11" * 32}),
            1,
            ["VOID", "ho-wikidata-l9.bz2 is '1111", "not the frozen 7f5c9a00"],
            absent=["VERDICT"],
        )
    )
    first_text = lambda r: r["corpus"] == "text" and r["threads"] == 8 and r["round"] == 0  # noqa: E731
    res.append(
        expect(
            "Researcher 6: the first text pair at threads 8.5: VOID, not coerced to 8",
            build(edit=lambda r: {**r, "threads": 8.5} if first_text(r) else r),
            1,
            ["VOID", "threads 8.5 is not one of the registered [8, 10, 20]", "records missing"],
            absent=["VERDICT"],
        )
    )
    res.append(
        expect(
            "Researcher 7: a numeric arm: VOID, not an AttributeError",
            build(edit=lambda r: {**r, "arm": 1} if first_text(r) else r),
            1,
            ["VOID", "arm 1 is not a string"],
            absent=["VERDICT"],
        )
    )
    # and their kin: each field's type and value, each receipt entry
    for name, edit, phrase in [
        ("threads the string '8'", {"threads": "8"}, "threads '8' is not one"),
        ("threads true", {"threads": True}, "threads True is not one"),
        ("round 0.0", {"round": 0.0}, "round 0.0 is not one"),
        ("a corpus that is not a string", {"corpus": 7}, "corpus 7 is not one"),
        ("an output digest that is not a string", {"sha256": 5}, "no output size or digest"),
    ]:
        res.append(
            expect(
                f"{name}: VOID",
                build(edit=lambda r, e=edit: {**r, **e} if first_text(r) else r),
                1,
                ["VOID", phrase],
                absent=["VERDICT"],
            )
        )
    d = build()
    with open(d / "runs.jsonl", "a") as f:
        f.write("[1, 2]\n")
    res.append(expect("a record that is not an object: VOID", d, 1, ["VOID", "not an object"]))
    for name, digests, phrase in [
        ("the primary's judged copy", {"wikidata-x4.bz2": "22" * 32}, "wikidata-x4.bz2 is '2222"),
        ("the sustained copy", {"wikidata-x16.bz2": "33" * 32}, "wikidata-x16.bz2 is '3333"),
        ("a digest in capitals", {"runs.bz2": "AB" * 32}, "not a SHA-256 for ['runs.bz2']"),
        ("a short digest", {"runs.bz2": "ab" * 31}, "not a SHA-256 for ['runs.bz2']"),
        ("an input outside the registration", {"other.bz2": "44" * 32}, "outside the registration"),
    ]:
        res.append(
            expect(f"{name}: VOID", build(digests=digests), 1, ["VOID", phrase], absent=["VERDICT"])
        )
    good = {n: "ab" * 32 for n in op.INPUTS} | op.FROZEN
    receipt = "".join(f"{v}  /w/c/{k}\n" for k, v in good.items())
    res.append(
        expect(
            "an input named twice: VOID",
            build(digests=receipt + f"{'ef' * 32}  /w/other/text.bz2\n"),
            1,
            ["VOID", "text.bz2 named twice"],
            absent=["VERDICT"],
        )
    )
    res.append(
        expect(
            "a receipt line of three fields: VOID",
            build(digests=receipt + "ab  cd  /w/c/text.bz2\n"),
            1,
            ["VOID", "not one digest and one path"],
            absent=["VERDICT"],
        )
    )
    res.append(
        expect(
            "the receipt as hoped, every digest well formed: ADOPTS",
            build(digests=receipt),
            0,
            ["VERDICT: ADOPTS"],
        )
    )
    # the Researcher's third correction (REVIEW 7008312938)
    res.append(
        expect(
            "Researcher 9: an empty effective digest (hash '', no sha256): VOID",
            build(
                edit=lambda r: (
                    {**{k: v for k, v in r.items() if k != "sha256"}, "hash": ""}
                    if r["corpus"] == "text"
                    else r
                )
            ),
            1,
            ["VOID", "no output size or digest"],
            absent=["VERDICT"],
        )
    )
    res.append(
        expect(
            "a blank sha256 and no hash: VOID",
            build(edit=lambda r: {**r, "sha256": "  "} if first_text(r) else r),
            1,
            ["VOID", "no output size or digest"],
            absent=["VERDICT"],
        )
    )
    for name in ("ho-prose-l9.bz2", "text.bz2", "ho-structured-l1.bz2"):
        res.append(
            expect(
                f"Researcher 10: a generated guard {name} off the manifest: VOID",
                build(digests={name: "5a" * 32}),
                1,
                ["VOID", f"{name} is '5a5a", "not the frozen"],
                absent=["VERDICT"],
            )
        )
    import heldout

    m = op.manifest()
    ok = (
        sorted(m) == sorted(op.INPUTS)
        and m["ho-wikidata-l9.bz2"] == op.HELDOUT_SHA256
        and heldout.SOURCE == op.V056
    )
    print(
        f"{'ok  ' if ok else 'FAIL'} the manifest names every input once, the held-out copy as its "
        "base, and the held-out sources are v0.5.6's tree"
    )
    res.append(ok)
    for name, d, want in [
        ("verify: the registered builds and inputs pass", build(), 0),
        ("verify: an input off the manifest refuses", build(digests={"runs.bz2": "6b" * 32}), 1),
        ("verify: another port refuses", build(commits=f"port {'1' * 40}\nv0.5.6 {op.V056}\n"), 1),
    ]:
        buf = io.StringIO()
        try:
            with redirect_stdout(buf):
                got = op.main(["open_port.py", "verify", str(d), "--port", PORT_SHA])
        finally:
            shutil.rmtree(d)
        print(f"{'ok  ' if got == want else 'FAIL'} {name}: exit {got}")
        res.append(got == want)
    res.extend(guard_checks())
    d = build()
    (d / "DRY-RUN.txt").write_text("dry\n")
    res.append(expect("a dry run: VOID", d, 1, ["VOID", "a dry run"], absent=["VERDICT"]))
    res.append(
        all(
            abs(0.5 * op.betainc(df / 2, 0.5, df / (df + t * t)) - p) < 2e-4
            for t, df, p in [
                (2.201, 11, 0.025),
                (1.796, 11, 0.05),
                (3.106, 11, 0.005),
                (12.706, 1, 0.025),
            ]
        )
    )
    print(f"{'ok  ' if res[-1] else 'FAIL'} Student's t, one-sided tails at df 11 and 1")
    failed = res.count(False)
    print(f"\n{len(res) - failed} of {len(res)} checks pass")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
