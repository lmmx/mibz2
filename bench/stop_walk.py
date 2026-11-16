#!/usr/bin/env python3
"""The stop-and-walk adoption run (`just bench-stop-walk`): its verdict, read from the run's
records only after the records are checked whole. Standard library only.

    stop_walk.py DIR [--rounds 10] [--threads 8,9,10,12,20] [--copies 4]

DIR holds the run's four record files (`ab run`'s JSON lines):
- `judged.jsonl`: `mibz2` and `mibz2-stop15` on the main corpus (`wikidata-xC`, C the copies
  of the Wikidata sample, `--copies`) and the text
  corpus (`text`), at the judged thread counts, every round;
- `forced.jsonl` (diagnostic): `mibz2-workers-lazy`, `-stop14`, `-stop15`, `-stop16` on the main
  corpus at the judged counts;
- `overlap.jsonl` (diagnostic): `mibz2` and `mibz2-stop15` on the main corpus at 10 threads with
  the held spinner sharing a decoder core;
- `dose.jsonl` (diagnostic): the forced arms on one copy (`wikidata`) at 1 and 10 threads.

**The registration** (#145; the Researcher's correction REVIEW 6921090225, accepted in LOG
6921203037). It decides whether stop-and-walk becomes *the* lazy kernel, where production
runs lazy workers now, and nothing else: no thread threshold of its own.
- *The arms:* `mibz2` is production's decoder. `mibz2-stop15` is production's decoder, plan and
  window with the lazy kernel replaced by stop-and-walk at β = 1.5
  (`StreamDecoder::new_lazy_as`), so the two differ in the kernel alone. On the i9, 12 and 20
  threads asked are ten lazy workers, as production runs them. Only β = 1.5 is eligible.
- *The judged run* has no spinner sharing a decoder core (the spinner is held only where a
  core is free: at 8 and 9 threads). The 10-thread case with the spinner sharing core 9 is a
  diagnostic.
- *A cell's step:* the rounds' paired wall ratios, `mibz2-stop15` over `mibz2`, their geometric
  mean and its 95% interval (Student's t on the logs). FASTER if the interval lies below 1,
  SLOWER if above, else INCONCLUSIVE (not equal).
- *Gain:* Holm's one-sided tests (the mean log ratio below 0) over the five main-corpus cells,
  family α = 0.05. The gain is established if Holm rejects at one cell or more.
- *Loss screen:* no judged cell, main or text, is SLOWER. (A noisy loss can still pass a
  screen that only finds the losses its intervals resolve.)
- *Memory gate:* at every judged cell, main and text, the median over the rounds of each run's
  peak resident memory, `mibz2-stop15` over `mibz2`, is at most 1.10.
- **ADOPTS:** gain established, loss screen clean, memory gate passed.
- **HELD ON MEMORY:** gain established and loss screen clean, memory gate failed: the
  Coordinator weighs it under PERFORMANCE-POLICY rule 4.
- **A FINDING, NOT ADOPTED:** gain established and a cell SLOWER. No range of its own: any
  change would be to the lazy threshold itself, a separate decision.
- **NOT ADOPTED:** otherwise.
- **VOID** (exit 1, no verdict): any record file missing or unreadable; any record not a
  successful run with finite positive wall, CPU and peak memory; any key (corpus, arm, threads,
  round) duplicated, unexpected or missing from the full roster; a corpus whose outputs (size
  and hash) differ; a judged, forced or dose run with the spinner sharing a decoder core.
"""
import argparse
import json
import math
import os
import statistics
import sys

JUDGED_ARMS = ["mibz2", "mibz2-stop15"]
FORCED_ARMS = ["mibz2-workers-lazy", "mibz2-workers-stop14", "mibz2-workers-stop15", "mibz2-workers-stop16"]
MAIN, GUARD, ONE = "wikidata-x4", "text", "wikidata"
ALPHA, RSS_GATE = 0.05, 1.10
# Student's t, two-sided 95%, by degrees of freedom (1..30); 1.96 beyond
T95 = [12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179,
       2.160, 2.145, 2.131, 2.120, 2.110, 2.101, 2.093, 2.086, 2.080, 2.074, 2.069, 2.064, 2.060,
       2.056, 2.052, 2.048, 2.045, 2.042]


def betainc(a, b, x):
    """The regularized incomplete beta function I_x(a, b) (a continued fraction, Lentz)"""
    if x <= 0:
        return 0.0
    if x >= 1:
        return 1.0
    if x > (a + 1) / (a + b + 2):
        return 1.0 - betainc(b, a, 1.0 - x)
    lfront = math.lgamma(a + b) - math.lgamma(a) - math.lgamma(b) + a * math.log(x) + b * math.log1p(-x)
    tiny = 1e-300
    c, d = 1.0, 1.0 - (a + b) * x / (a + 1)
    d = 1.0 / (d if abs(d) > tiny else tiny)
    f = d
    for m in range(1, 400):
        for num in (m * (b - m) * x / ((a + 2 * m - 1) * (a + 2 * m)),
                    -(a + m) * (a + b + m) * x / ((a + 2 * m) * (a + 2 * m + 1))):
            d = 1.0 + num * d
            d = 1.0 / (d if abs(d) > tiny else tiny)
            c = 1.0 + num / c
            c = c if abs(c) > tiny else tiny
            f *= c * d
        if abs(c * d - 1.0) < 1e-15:
            break
    return math.exp(lfront) * f / a


def t_cdf(t, df):
    """P(T <= t), T Student's t with df degrees of freedom"""
    if math.isinf(t):
        return 0.0 if t < 0 else 1.0
    tail = 0.5 * betainc(df / 2, 0.5, df / (df + t * t))
    return tail if t < 0 else 1.0 - tail


def step(logs):
    """(geometric mean, 95% low, 95% high, one-sided p that the mean log is below 0)"""
    n = len(logs)
    mu = statistics.mean(logs)
    sd = statistics.stdev(logs)
    t95 = T95[n - 2] if n - 1 <= len(T95) else 1.96
    h = t95 * sd / math.sqrt(n)
    if sd == 0:
        p = 0.0 if mu < 0 else 1.0
    else:
        p = t_cdf(mu / (sd / math.sqrt(n)), n - 1)
    return math.exp(mu), math.exp(mu - h), math.exp(mu + h), p


def label(lo, hi):
    return "FASTER" if hi < 1 else "SLOWER" if lo > 1 else "INCONCLUSIVE"


def holm(ps, alpha):
    """Which of `ps` (a dict) Holm's step-down procedure rejects at family `alpha`"""
    rejected, m = set(), len(ps)
    for i, (key, p) in enumerate(sorted(ps.items(), key=lambda kv: kv[1])):
        if p > alpha / (m - i):
            break
        rejected.add(key)
    return rejected


def load(path, roster, errors, outputs, spinner_ok):
    """The file's records by key, after checking each record and the roster; errors appended"""
    name = os.path.basename(path)
    rows = {}
    if not os.path.exists(path):
        errors.append(f"{name}: missing")
        return rows
    with open(path) as f:
        for ln, line in enumerate(f, 1):
            if not line.strip():
                continue
            where = f"{name}:{ln}"
            try:
                r = json.loads(line)
            except ValueError as e:
                errors.append(f"{where}: not JSON ({e})")
                continue
            if not isinstance(r, dict):
                errors.append(f"{where}: not a record")
                continue
            try:
                key = (str(r["corpus"]), str(r["arm"]), int(r["threads"]), int(r["round"]))
            except (KeyError, TypeError, ValueError):
                errors.append(f"{where}: no corpus, arm, threads or round")
                continue
            if key in rows:
                errors.append(f"{where}: {key} duplicated")
                continue
            if key not in roster:
                errors.append(f"{where}: {key} not in the roster")
            if r.get("err") or r.get("termination") != "success":
                errors.append(f"{where}: {key} did not succeed ({r.get('termination')!r}, {r.get('err')!r})")
            for field in ("wall", "cpu", "rss_kb"):
                v = r.get(field)
                if isinstance(v, bool) or not isinstance(v, (int, float)) or not math.isfinite(v) or v <= 0:
                    errors.append(f"{where}: {key} {field} {v!r} is not a finite positive number")
            out, digest = r.get("out"), r.get("sha256") or r.get("hash")
            if isinstance(out, bool) or not isinstance(out, int) or out <= 0 or not isinstance(digest, str) or not digest:
                errors.append(f"{where}: {key} output size {out!r} or hash {digest!r} missing")
            else:
                outputs.setdefault(key[0], set()).add((out, digest))
            hs = r.get("held_spinner")
            if not spinner_ok and isinstance(hs, dict) and hs.get("overlap"):
                errors.append(f"{where}: {key} ran beside a spinner sharing a decoder core")
            rows[key] = r
    missing = sorted(roster - set(rows))
    if missing:
        errors.append(f"{name}: {len(missing)} of {len(roster)} roster keys missing, e.g. {missing[:3]}")
    return rows


def roster(corpora, arms, threads, rounds):
    return {(c, a, t, i) for c in corpora for a in arms for t in threads for i in range(rounds)}


def cell(rows, corpus, arm, ref, t, rounds, key="wall"):
    return [math.log(rows[(corpus, arm, t, i)][key] / rows[(corpus, ref, t, i)][key]) for i in range(rounds)]


def peak(rows, corpus, arm, t, rounds):
    return statistics.median(rows[(corpus, arm, t, i)]["rss_kb"] for i in range(rounds))


def main(argv):
    ap = argparse.ArgumentParser()
    ap.add_argument("dir")
    ap.add_argument("--rounds", type=int, default=10)
    ap.add_argument("--threads", default="8,9,10,12,20")
    ap.add_argument("--copies", type=int, default=4)
    a = ap.parse_args(argv[1:])
    global MAIN
    MAIN = f"wikidata-x{a.copies}"
    threads = [int(t) for t in a.threads.split(",")]
    R = a.rounds
    errors, outputs = [], {}
    if R < 2:
        errors.append(f"--rounds {R}: at least 2")
    files = {
        "judged": (roster([MAIN, GUARD], JUDGED_ARMS, threads, R), False),
        "forced": (roster([MAIN], FORCED_ARMS, threads, R), False),
        "overlap": (roster([MAIN], JUDGED_ARMS, [10], R), True),
        "dose": (roster([ONE], FORCED_ARMS, [1, 10], R), False),
    }
    data = {f: load(os.path.join(a.dir, f + ".jsonl"), ros, errors, outputs, ok) for f, (ros, ok) in files.items()}
    for corpus, outs in sorted(outputs.items()):
        if len(outs) != 1:
            errors.append(f"{corpus}: the runs' outputs differ: {sorted(outs)[:3]}")
    n = sum(len(d) for d in data.values())
    print(f"records {n} (judged {len(data['judged'])}, forced {len(data['forced'])}, "
          f"overlap {len(data['overlap'])}, dose {len(data['dose'])}); roster "
          + ", ".join(f"{f} {len(ros)}" for f, (ros, _) in files.items()))
    if errors:
        print(f"VOID: the records do not make the registered run ({len(errors)} problems); no verdict")
        for e in errors[:25]:
            print(f"  {e}")
        if len(errors) > 25:
            print(f"  ... and {len(errors) - 25} more")
        return 1

    J = data["judged"]
    print(f"\nThe judged run: mibz2-stop15 over mibz2, the paired rounds' wall ratios: geometric mean "
          f"[95% interval]; one-sided p (gain); peak memory (median of the runs' peaks) over mibz2's")
    steps, ps, mem = {}, {}, {}
    for corpus in (MAIN, GUARD):
        print(f"\n{corpus}{' (gain family)' if corpus == MAIN else ' (loss and memory guard)'}")
        for t in threads:
            g, lo, hi, p = step(cell(J, corpus, "mibz2-stop15", "mibz2", t, R))
            steps[(corpus, t)] = label(lo, hi)
            mem[(corpus, t)] = peak(J, corpus, "mibz2-stop15", t, R) / peak(J, corpus, "mibz2", t, R)
            if corpus == MAIN:
                ps[t] = p
            print(f"  {t:>2} threads: {g:.4f} [{lo:.4f}, {hi:.4f}] {steps[(corpus, t)]}"
                  + (f"; p {p:.2e}" if corpus == MAIN else "") + f"; memory {mem[(corpus, t)]:.3f}")
    rejected = holm(ps, ALPHA)
    print(f"\nHolm at family α {ALPHA} over the {len(ps)} main-corpus cells: "
          + ", ".join(f"{t} p {ps[t]:.2e} {'rejects' if t in rejected else 'does not'}"
                      for t in sorted(ps, key=ps.get)))
    gain = bool(rejected)
    slower = sorted(k for k, v in steps.items() if v == "SLOWER")
    over = sorted(k for k, v in mem.items() if v > RSS_GATE)
    print()
    if gain and not slower and not over:
        print(f"VERDICT: ADOPTS. A gain at {sorted(rejected)} threads (Holm), no judged cell SLOWER, and peak memory "
              f"within {RSS_GATE:.2f} of production's at every judged cell: stop-and-walk at β = 1.5 becomes the lazy kernel")
    elif gain and not slower:
        print(f"VERDICT: HELD ON MEMORY. A gain at {sorted(rejected)} threads (Holm) and no judged cell SLOWER, but peak "
              f"memory is above {RSS_GATE:.2f} of production's at {over}: PERFORMANCE-POLICY rule 4, the Coordinator's to weigh")
    elif gain:
        print(f"VERDICT: A FINDING, NOT ADOPTED. A gain at {sorted(rejected)} threads (Holm), but SLOWER at {slower}. "
              "No range of its own: any change would be to the lazy threshold itself, a separate decision")
    else:
        print("VERDICT: NOT ADOPTED. No gain established (Holm rejects at no main-corpus cell)"
              + (f"; SLOWER at {slower}" if slower else "") + (f"; memory above the gate at {over}" if over else ""))

    print("\nDiagnostics (decide nothing):")
    F = data["forced"]
    print("  forced workers, each arm over mibz2-workers-lazy on the main corpus (12 and 20 are that many workers):")
    for arm in FORCED_ARMS[1:]:
        cells = []
        for t in threads:
            g, lo, hi, _ = step(cell(F, MAIN, arm, "mibz2-workers-lazy", t, R))
            cells.append(f"{t} {g:.3f} {label(lo, hi)}")
        print(f"    {arm}: " + ", ".join(cells))
    O = data["overlap"]
    g, lo, hi, _ = step(cell(O, MAIN, "mibz2-stop15", "mibz2", 10, R))
    shared = sum(bool((r.get("held_spinner") or {}).get("overlap")) for r in O.values())
    print(f"  10 threads, the spinner sharing a decoder core ({shared} of {len(O)} runs shared): "
          f"{g:.4f} [{lo:.4f}, {hi:.4f}] {label(lo, hi)}")
    D = data["dose"]
    for arm in FORCED_ARMS[1:]:
        g, lo, hi, _ = step(cell(D, ONE, arm, "mibz2-workers-lazy", 1, R))
        print(f"  1 thread, {arm} over mibz2-workers-lazy: wall {g:.4f} [{lo:.4f}, {hi:.4f}]")
    dose = {arm: {i: D[(ONE, arm, 10, i)]["cpu"] / D[(ONE, arm, 1, i)]["cpu"] for i in range(R)} for arm in FORCED_ARMS}
    for arm in FORCED_ARMS:
        g, lo, hi, _ = step([math.log(v) for v in dose[arm].values()])
        line = f"  dose, {arm}: CPU at 10 over CPU at 1, {g:.4f} [{lo:.4f}, {hi:.4f}]"
        if arm != "mibz2-workers-lazy":
            g2, lo2, hi2, _ = step([math.log(dose[arm][i] / dose["mibz2-workers-lazy"][i]) for i in range(R)])
            line += f"; over mibz2-workers-lazy's, round by round, {g2:.4f} [{lo2:.4f}, {hi2:.4f}]"
        print(line)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
