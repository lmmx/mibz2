#!/usr/bin/env python3
"""The opend worker sweep's verdict (`just bench-opend-sweep`; #145 QUEUED 6992388066, step 1 of
the B5 study; MONODROMY's record 71): opend against LazyCells, both built the same way
(`StreamDecoder::workers_with`, `mibz2-opend-W-4W` against `mibz2-occ-W-4W`), unpinned on the
i9 at W = 2 to 10 workers. A diagnostic: it decides nothing in production. Standard library
only.

    opend_sweep.py RUNS.jsonl --lscpu LSCPU.txt --stalls-dir DIR [--rounds 12] [--reps 5]
                   [--counts 2,3,4,5,6,7,8,9,10] [--machine-model i9-10900K]

**The records, checked whole first.** The verdict is VOID, and nothing is read, where:
- a timed record is missing, duplicated, malformed or failed, for any (corpus, W, arm, round);
- a corpus's outputs differ between any two records;
- an arm's record does not observe exactly its W workers (`mibz2-0` … `mibz2-{W-1}`);
- the run's `lscpu` does not name exactly one CPU model, containing `--machine-model`;
- a counters record at a W is missing, failed, or has an event not counted or not positive;
- the run is a dry run (`DRY-RUN.txt` beside the records): never evidence.

**The ratios.** At each W, on each corpus, the rounds' paired log ratios of wall time,
opend over LazyCells: their mean as a geometric mean, with a two-sided 95% t-interval.

**The predictions scored** (record 71 §3, frozen before the data):
- **M1, the ratio falls with W:** the log ratio at W = 2 exceeds that at W = 10, the 95%
  interval of the difference (Welch, its degrees of freedom the smaller of the two) excluding 0.
- **M2, it crosses 1 between W = 2 and 6:** W = 2's interval lies wholly above 1, and some W
  from 3 to 6 has its interval wholly below 1. Reported, secondary.
- **M3, fewer L3-miss stalls a byte at W ≥ 7:** at W = 8, `cycle_activity.stalls_l3_miss` per
  output byte, opend over LazyCells, the median of the reps' paired ratios below 1 and every
  rep's ratio below 1. W = 7, 9 and 10 are reported beside it.

**The verdict:**
- **REFUTED** where M1 fails (the interval includes 0, or the ratio rises with W), or M3 fails
  at W = 8 (the median paired ratio at 1 or above). Record 71's refutation: "flat in W, or
  opend's stalls a byte no lower at W = 8".
- **SUPPORTED** where M1 and M3 hold.
- **INCONCLUSIVE** otherwise: M3's median below 1 at W = 8, but not every rep.

Text is an A/A check: opend runs LazyCells as shipped there (no text block is under the density
line). Its ratio at each W is reported, and an interval excluding 1 is flagged, not judged.
"""
import argparse
import json
import math
import os
import re
import statistics
import sys

OPEND, LAZY = "mibz2-opend-{w}-{s}", "mibz2-occ-{w}-{s}"
MAIN, AA = "wikidata-x4", "text"
STALL = "cycle_activity.stalls_l3_miss"
# two-sided 97.5% quantiles of Student's t, by degrees of freedom (beyond 30, the normal's)
T975 = {1: 12.706, 2: 4.303, 3: 3.182, 4: 2.776, 5: 2.571, 6: 2.447, 7: 2.365, 8: 2.306, 9: 2.262,
        10: 2.228, 11: 2.201, 12: 2.179, 13: 2.160, 14: 2.145, 15: 2.131, 16: 2.120, 17: 2.110,
        18: 2.101, 19: 2.093, 20: 2.086, 21: 2.080, 22: 2.074, 23: 2.069, 24: 2.064, 25: 2.060,
        26: 2.056, 27: 2.052, 28: 2.048, 29: 2.045, 30: 2.042}


def t975(df):
    return T975.get(df, 1.96)


def arms(w):
    return OPEND.format(w=w, s=4 * w), LAZY.format(w=w, s=4 * w)


def workers(r):
    names = [p.partition("=")[0] for p in str(r.get("observed_threads") or "").split()]
    return sorted(n for n in names if re.fullmatch(r"mibz2-\d+", n))


def machine(path, want, errors):
    try:
        text = open(path).read()
    except OSError as e:
        errors.append(f"--lscpu {path}: unreadable ({e.strerror or e}): the machine is unidentified")
        return None
    models = set(re.findall(r"^\s*Model name:\s*(.+?)\s*$", text, re.M))
    if len(models) != 1:
        errors.append(f"--lscpu: {len(models)} model names, not one: the machine is unidentified")
        return None
    model = models.pop()
    if want not in model:
        errors.append(f"not the registered machine: the CPU is {model!r}, not a {want!r}")
        return None
    return model


def load_runs(path, counts, rounds, errors):
    rows, outs = {}, {}
    roster = {(c, w, arm, i) for c in (MAIN, AA) for w in counts for arm in arms(w) for i in range(rounds)}
    if not os.path.exists(path):
        errors.append(f"{path}: missing")
        return rows
    for ln, line in enumerate(open(path), 1):
        if not line.strip():
            continue
        try:
            r = json.loads(line)
            key = (r["corpus"], int(r["threads"]), r["arm"], int(r["round"]))
        except (ValueError, KeyError, TypeError):
            errors.append(f"runs line {ln}: not a record")
            continue
        if key not in roster:
            errors.append(f"runs line {ln}: {key} not in the roster")
            continue
        if key in rows:
            errors.append(f"runs line {ln}: {key} duplicated")
            continue
        wall = r.get("wall")
        if r.get("termination") != "success" or r.get("err") or isinstance(wall, bool) \
                or not isinstance(wall, (int, float)) or not wall > 0:
            errors.append(f"{key}: failed or no wall time ({r.get('termination')!r}, {r.get('err')!r})")
            continue
        w = key[1]
        if workers(r) != sorted(f"mibz2-{k}" for k in range(w)):
            errors.append(f"{key}: observed workers {workers(r)}, not the arm's {w}")
        outs.setdefault(key[0], set()).add((r.get("out"), r.get("sha256") or r.get("hash")))
        rows[key] = r
    missing = sorted(roster - set(rows))
    if missing:
        errors.append(f"{len(missing)} of {len(roster)} timed records missing, e.g. {missing[:3]}")
    for c, o in sorted(outs.items()):
        if len(o) != 1:
            errors.append(f"{c}: the outputs differ: {sorted(o, key=str)[:3]}")
    return rows


def load_stalls(d, counts, reps, errors):
    """{(w, arm, rep): stalls a byte}"""
    out = {}
    for w in counts:
        path = os.path.join(d, f"stalls-{w}.jsonl")
        if not os.path.exists(path):
            errors.append(f"stalls-{w}.jsonl: missing")
            continue
        for ln, line in enumerate(open(path), 1):
            if not line.strip():
                continue
            try:
                r = json.loads(line)
                arm, rep = r["arm"], int(r["rep"])
            except (ValueError, KeyError, TypeError):
                errors.append(f"stalls-{w}.jsonl line {ln}: not a record")
                continue
            counts_ = r.get("counts") or {}
            if STALL not in counts_:
                continue
            if r.get("err") or r.get("rc") != 0 or r.get("child_err"):
                errors.append(f"stalls-{w}.jsonl line {ln}: {arm} rep {rep} failed")
                continue
            v, outb = counts_[STALL][0], r.get("out")
            if not isinstance(v, (int, float)) or isinstance(v, bool) or not v > 0 or not isinstance(outb, int) or outb <= 0:
                errors.append(f"stalls-{w}.jsonl line {ln}: {arm} rep {rep}: {STALL} {v!r} not counted")
                continue
            out[(w, arm, rep)] = v / outb
        for arm in arms(w):
            for rep in range(reps):
                if (w, arm, rep) not in out:
                    errors.append(f"stalls-{w}.jsonl: {arm} rep {rep} has no {STALL}")
    return out


def cell(rows, corpus, w, rounds):
    o, l = arms(w)
    xs = [math.log(rows[(corpus, w, o, i)]["wall"] / rows[(corpus, w, l, i)]["wall"]) for i in range(rounds)]
    m, sd = statistics.mean(xs), statistics.stdev(xs)
    se = sd / math.sqrt(len(xs))
    h = t975(len(xs) - 1) * se
    return m, se, (m - h, m + h)


def main(argv):
    ap = argparse.ArgumentParser()
    ap.add_argument("runs")
    ap.add_argument("--lscpu", required=True)
    ap.add_argument("--stalls-dir", required=True)
    ap.add_argument("--rounds", type=int, default=12)
    ap.add_argument("--reps", type=int, default=5)
    ap.add_argument("--counts", default="2,3,4,5,6,7,8,9,10")
    ap.add_argument("--machine-model", default="i9-10900K")
    a = ap.parse_args(argv[1:])
    counts = [int(w) for w in a.counts.split(",")]
    errors = []
    if a.rounds < 2 or a.reps < 1 or not {2, 8, 10} <= set(counts):
        errors.append("--rounds at least 2, --reps at least 1, and the counts must include 2, 8 and 10")
    if os.path.exists(os.path.join(os.path.dirname(os.path.abspath(a.runs)), "DRY-RUN.txt")):
        errors.append("a dry run (DRY-RUN.txt beside the records): not evidence")
    model = machine(a.lscpu, a.machine_model, errors)
    rows = load_runs(a.runs, counts, a.rounds, errors)
    stalls = load_stalls(a.stalls_dir, counts, a.reps, errors)
    print(f"records {len(rows)}; W {counts}; {a.rounds} rounds; {a.reps} counter reps"
          + (f"; the machine: {model}" if model else ""))
    if errors:
        print(f"VOID: the records do not make the registered run ({len(errors)} problems); no verdict")
        for e in errors[:25]:
            print(f"  {e}")
        return 1

    print("\nopend over LazyCells, wall, the rounds' paired ratios: geometric mean [95% interval]")
    res = {}
    for w in counts:
        m, se, (lo, hi) = cell(rows, MAIN, w, a.rounds)
        res[w] = (m, se)
        am, _, (alo, ahi) = cell(rows, AA, w, a.rounds)
        flag = "" if alo <= 0 <= ahi else "  (A/A: text's interval excludes 1, flagged)"
        print(f"  W {w:2d}: {MAIN} {math.exp(m):.4f} [{math.exp(lo):.4f}, {math.exp(hi):.4f}]; "
              f"{AA} {math.exp(am):.4f} [{math.exp(alo):.4f}, {math.exp(ahi):.4f}]{flag}")

    (m2, s2), (m10, s10) = res[2], res[10]
    d, sd = m2 - m10, math.sqrt(s2 ** 2 + s10 ** 2)
    h = t975(a.rounds - 1) * sd
    m1 = d - h > 0
    print(f"\nM1, the ratio falls with W: W 2 over W 10 {math.exp(d):.4f} [{math.exp(d - h):.4f}, {math.exp(d + h):.4f}]: "
          + ("HOLDS" if m1 else "FAILS"))

    def below(w):
        m, se = res[w]
        return m + t975(a.rounds - 1) * se < 0
    above2 = res[2][0] - t975(a.rounds - 1) * res[2][1] > 0
    cross = [w for w in counts if 3 <= w <= 6 and below(w)]
    m2_ = above2 and bool(cross)
    print(f"M2, it crosses 1 between W 2 and 6: W 2 above 1 {'yes' if above2 else 'no'}; below 1 at "
          f"{cross or 'none of 3–6'}: " + ("CONFIRMED" if m2_ else "NOT CONFIRMED") + " (secondary)")

    print(f"\n{STALL} a byte, opend over LazyCells, the median of {a.reps} reps' paired ratios:")
    m3 = {}
    for w in counts:
        o, l = arms(w)
        rs = [stalls[(w, o, r)] / stalls[(w, l, r)] for r in range(a.reps)]
        med = statistics.median(rs)
        m3[w] = (med, all(x < 1 for x in rs))
        print(f"  W {w:2d}: {med:.4f} (reps {', '.join(f'{x:.3f}' for x in rs)}); "
              f"LazyCells {statistics.median(stalls[(w, l, r)] for r in range(a.reps)):.3f} a byte, "
              f"opend {statistics.median(stalls[(w, o, r)] for r in range(a.reps)):.3f}")
    med8, all8 = m3[8]
    m3h = med8 < 1 and all8
    print(f"M3, fewer L3-miss stalls a byte at W 8: median {med8:.4f}, every rep below 1: {'yes' if all8 else 'no'}: "
          + ("HOLDS" if m3h else "FAILS" if med8 >= 1 else "NOT EVERY REP"))

    if not m1 or med8 >= 1:
        verdict = "REFUTED: " + ("the ratio does not fall from W 2 to W 10" if not m1 else "") + \
                  (" and " if not m1 and med8 >= 1 else "") + \
                  ("opend's L3-miss stalls a byte are no lower at W 8" if med8 >= 1 else "")
    elif m3h:
        verdict = "SUPPORTED: the ratio falls with W, and opend stalls less on L3 misses a byte at W 8"
    else:
        verdict = "INCONCLUSIVE: the ratio falls with W; at W 8 the stalls' median is below 1 but not every rep"
    print(f"\nTHE MECHANISM (record 71 §3): {verdict}. A diagnostic: it decides nothing in production.")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
