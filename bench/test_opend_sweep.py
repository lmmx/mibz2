#!/usr/bin/env python3
"""Checks of bench/opend_sweep.py, the opend worker sweep's verdict, on made-up records"""
import io
import json
import math
import shutil
import sys
import tempfile
from contextlib import redirect_stdout
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import opend_sweep as osw  # noqa: E402

COUNTS, R, REPS = list(range(2, 11)), 12, 5
I9 = "Model name:                           Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz\n"
OUT = {"wikidata-x4": (838800784, "aa"), "text": (49067679, "bb")}


def ratio_falling(w):
    """opend over LazyCells: 1.08 at W 2, falling to 0.92 at W 10 (record 71's prediction)"""
    return 1.08 - 0.02 * (w - 2)


def build(steps=ratio_falling, stalls=lambda w: 0.8 if w >= 7 else 1.1, text=lambda w: 1.0, edit=None,
          stall_edit=None, lscpu=I9):
    d = Path(tempfile.mkdtemp())
    rows = []
    for w in COUNTS:
        o, lz = osw.arms(w)
        for i in range(R):
            for corpus, (out, h) in OUT.items():
                base = 0.5 * (1 + 0.01 * ((i * 7 + w) % 5 - 2))
                wig = 1 + 0.004 * (1 if i % 2 else -1)
                for arm in (lz, o):
                    f = (steps(w) if corpus == "wikidata-x4" else text(w)) * wig if arm == o else 1.0
                    r = {"round": i, "corpus": corpus, "threads": w, "arm": arm, "termination": "success",
                         "wall": base * f, "out": out, "sha256": h, "observed_threads": " ".join(
                             ["ab", *(f"mibz2-{k}" for k in range(w)), "mibz2-read", "mibz2-scan", "sampler"])}
                    r = edit(r) if edit else r
                    if r is not None:
                        rows.append(r)
    (d / "runs.jsonl").write_text("".join(json.dumps(r) + "\n" for r in rows))
    for w in COUNTS:
        o, lz = osw.arms(w)
        recs = []
        for rep in range(REPS):
            for arm in (lz, o):
                for g, events in enumerate([["cycles", "instructions"], ["cycles", osw.STALL]]):
                    per = 2.0 * (stalls(w) * (1 + 0.01 * (rep - 2)) if arm == o else 1.0)
                    counts = {e: [int(1e9), 100.0, "x"] for e in events}
                    if osw.STALL in events:
                        counts[osw.STALL] = [per * 838800784, 100.0, "x"]
                    rec = {"arm": arm, "threads": w, "rep": rep, "group": g, "rc": 0, "out": 838800784,
                           "hash": "aa", "counts": counts}
                    rec = stall_edit(w, rec) if stall_edit else rec
                    if rec is not None:
                        recs.append(rec)
        (d / f"stalls-{w}.jsonl").write_text("".join(json.dumps(r) + "\n" for r in recs))
    if lscpu is not None:
        (d / "lscpu.txt").write_text(lscpu)
    return d


def run(d, extra=()):
    buf = io.StringIO()
    with redirect_stdout(buf):
        code = osw.main(["opend_sweep.py", str(d / "runs.jsonl"), "--lscpu", str(d / "lscpu.txt"),
                         "--stalls-dir", str(d), *extra])
    return code, buf.getvalue()


def expect(name, d, code, phrases, absent=(), extra=()):
    try:
        got, out = run(d, extra)
    finally:
        shutil.rmtree(d)
    ok = got == code and all(p in out for p in phrases) and not any(p in out for p in absent)
    print(f"{'ok  ' if ok else 'FAIL'} {name}: exit {got}, " + ", ".join(
        [f"{'found' if p in out else 'missing'} {p!r}" for p in phrases]
        + [f"{'present' if p in out else 'absent'} {p!r}" for p in absent]))
    if not ok:
        print(out)
    return ok


def main():
    res = []
    res.append(expect("the prediction met: SUPPORTED, the crossover confirmed", build(), 0, [
        "records 432", "the machine: Intel(R) Core(TM) i9-10900K", "W  2: wikidata-x4 1.0800",
        "M1, the ratio falls with W: W 2 over W 10 1.1739", "HOLDS", "M2, it crosses 1 between W 2 and 6",
        "CONFIRMED", "M3, fewer L3-miss stalls a byte at W 8: median 0.8000", "THE MECHANISM (record 71 §3): SUPPORTED"],
        absent=["VOID", "flagged"]))
    res.append(expect("flat in W: REFUTED", build(steps=lambda w: 0.95), 0, [
        "M1, the ratio falls with W", "FAILS", "REFUTED: the ratio does not fall from W 2 to W 10"],
        absent=["SUPPORTED"]))
    res.append(expect("rising in W: REFUTED", build(steps=lambda w: 0.9 + 0.01 * w), 0, ["FAILS", "REFUTED"]))
    res.append(expect("stalls no lower at W 8: REFUTED", build(stalls=lambda w: 1.05), 0, [
        "REFUTED: opend's L3-miss stalls a byte are no lower at W 8"], absent=["SUPPORTED"]))
    res.append(expect("both fail: REFUTED, both named", build(steps=lambda w: 0.95, stalls=lambda w: 1.05), 0, [
        "REFUTED: the ratio does not fall from W 2 to W 10 and opend's L3-miss stalls"]))
    res.append(expect("stalls below 1 at the median, not every rep: INCONCLUSIVE",
                      build(stall_edit=lambda w, r: {**r, "counts": {**r["counts"], osw.STALL: [
                          r["counts"][osw.STALL][0] * 1.4, 100.0, "x"]}} if w == 8 and r["rep"] == 4
                          and r["arm"].startswith("mibz2-opend") and osw.STALL in r["counts"] else r), 0,
                      ["NOT EVERY REP", "INCONCLUSIVE"]))
    res.append(expect("no crossover in 3–6: M2 not confirmed, the verdict unchanged",
                      build(steps=lambda w: 1.2 - 0.02 * (w - 2)), 0, ["NOT CONFIRMED", "SUPPORTED"]))
    res.append(expect("text off 1: flagged, not judged", build(text=lambda w: 1.05), 0, [
        "flagged", "SUPPORTED"]))
    res.append(expect("another CPU: VOID", build(lscpu="Model name: Intel(R) Xeon(R) Processor @ 2.10GHz\n"), 1, [
        "VOID", "not the registered machine"], absent=["THE MECHANISM"]))
    res.append(expect("no lscpu: VOID", build(lscpu=None), 1, ["VOID", "unidentified"]))
    res.append(expect("a timed record missing: VOID", build(
        edit=lambda r: None if r["threads"] == 7 and r["round"] == 3 and r["corpus"] == "text" else r), 1,
        ["VOID", "timed records missing"]))
    res.append(expect("a record with other workers: VOID", build(
        edit=lambda r: {**r, "observed_threads": "ab mibz2-0"} if r["threads"] == 9 and r["round"] == 0 else r), 1,
        ["VOID", "observed workers ['mibz2-0'], not the arm's 9"]))
    res.append(expect("no thread census: VOID", build(
        edit=lambda r: {k: v for k, v in r.items() if k != "observed_threads"} if r["threads"] == 2 else r), 1,
        ["VOID", "not the arm's 2"]))
    res.append(expect("outputs differ: VOID", build(
        edit=lambda r: {**r, "sha256": "zz"} if r["threads"] == 4 and r["round"] == 1 and r["corpus"] == "text" else r),
        1, ["VOID", "text: the outputs differ"]))
    res.append(expect("a failed decode: VOID", build(
        edit=lambda r: {**r, "termination": "exit 1"} if r["threads"] == 5 and r["round"] == 2 else r), 1,
        ["VOID", "failed or no wall time"]))
    res.append(expect("a stall count missing: VOID", build(
        stall_edit=lambda w, r: None if w == 8 and r["rep"] == 0 and r["group"] == 1 and "opend" in r["arm"] else r),
        1, ["VOID", "has no cycle_activity.stalls_l3_miss"]))
    res.append(expect("a stall count not counted: VOID", build(
        stall_edit=lambda w, r: {**r, "counts": {**r["counts"], osw.STALL: [None, 0.0, "<not counted>"]}}
        if w == 10 and osw.STALL in r["counts"] and r["rep"] == 1 else r), 1, ["VOID", "not counted"]))
    res.append(expect("a stalls file missing: VOID", build(stall_edit=lambda w, r: r), 1,
                      ["VOID", "stalls-11.jsonl: missing"], extra=["--counts", "2,3,4,5,6,7,8,9,10,11"]))
    def dry():
        d = build()
        (d / "DRY-RUN.txt").write_text("bench-opend-sweep: DRY RUN\n")
        return d
    res.append(expect("a dry run: VOID, never evidence", dry(), 1, ["VOID", "a dry run"], absent=["THE MECHANISM"]))
    res.append(abs(math.log(osw.t975(11)) - math.log(2.201)) < 1e-9 and osw.t975(100) == 1.96)
    print(f"{'ok  ' if res[-1] else 'FAIL'} t's quantiles")
    failed = res.count(False)
    print(f"\n{len(res) - failed} of {len(res)} checks pass")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
