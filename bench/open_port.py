#!/usr/bin/env python3
"""The open port's adoption verdict (`just bench-open-port`; the C6 registration
docs/journal/2026-11-19-open-port-registration.md; #145 CHECKPOINT 7004759994): the port's
release build (`mibz2`) against v0.5.6's (`mibz2@<v0.5.6's ab>`), both through the same public
entry point (`BzReader::parallel`), unpinned on the i9 at the shipped thread counts.

    open_port.py judge RUNS.jsonl SUSTAINED.jsonl --lscpu LSCPU --env ENV.json
                       [--counts 8,10,20] [--rounds 12] [--sustained-rounds 8]
    open_port.py pairs RUNS.jsonl --pair X/Y [--pair X/Y ...]   (context: reported, not judged)

**VOID, and nothing is read,** where:
- a timed record is missing, duplicated, malformed or failed;
- a corpus's outputs differ;
- the machine is not the registered one: `lscpu` names no single i9-10900K, or `ab env` records
  other than 10 cores and 20 CPUs;
- either arm observes other than the shipped workers at a count: 8 at 8 threads, 7 at 10 and 20
  (0.5.6's plan and the port's are the same plan; only the kernel differs);
- the run is a dry run (`DRY-RUN.txt` beside the records).

**Each cell** (corpus, count): the rounds' paired log ratios of wall, the port over v0.5.6, as a
geometric mean with a two-sided 95% t-interval; a one-sided p for a gain; and peak RSS as the
median of the port's peaks over the median of v0.5.6's.

**The rule:**
- **P1, wall (primary):** a gain on `wikidata-x4` at every judged count, Holm at family
  α 0.05 over those cells.
- **P2, peak RSS (co-primary):** on `wikidata-x4`, at most 1.00 of v0.5.6's at every count.
- **H, held-out replication:** on `ho-wikidata-l9` (Wikidata windows never measured before),
  FASTER (the interval wholly below 1) at every count.
- **G, non-regression on every other corpus at every count:** not SLOWER (the interval wholly
  above 1), a geometric mean at most 1.02, and peak RSS at most 1.10.
- **S, sustained:** 10 threads, back to back (no settle gap), on `wikidata-x16`: not SLOWER, and a
  geometric mean at most 1.02.

**ADOPTS** only where P1, P2, H, every G and S hold. Otherwise **NOT ADOPTED**, every failing
condition named. Standard library only.
"""
import argparse
import json
import math
import os
import re
import statistics
import sys

CAND, MAIN, HELDOUT, SUSTAINED = "mibz2", "wikidata-x4", "ho-wikidata-l9", "wikidata-x16"
WORKERS = {8: 8, 10: 7, 20: 7}
ALPHA, GUARD_MAX, RSS_GATE = 0.05, 1.02, 1.10
T975 = {1: 12.706, 2: 4.303, 3: 3.182, 4: 2.776, 5: 2.571, 6: 2.447, 7: 2.365, 8: 2.306, 9: 2.262,
        10: 2.228, 11: 2.201, 12: 2.179, 13: 2.160, 14: 2.145, 15: 2.131, 16: 2.120, 17: 2.110,
        18: 2.101, 19: 2.093, 20: 2.086, 21: 2.080, 22: 2.074, 23: 2.069, 24: 2.064, 25: 2.060}


def t975(df):
    return T975.get(df, 1.96)


def p_gain(xs):
    """One-sided p that the mean log ratio is below 0 (Student's t, by a series that is exact
    enough here: the regularised incomplete beta by continued fraction)"""
    n = len(xs)
    m, sd = statistics.mean(xs), statistics.stdev(xs)
    if sd == 0:
        return 0.0 if m < 0 else 1.0
    t = m / (sd / math.sqrt(n))
    df = n - 1
    x = df / (df + t * t)
    tail = 0.5 * betainc(df / 2, 0.5, x)
    return tail if t < 0 else 1 - tail


def betainc(a, b, x):
    if x <= 0:
        return 0.0
    if x >= 1:
        return 1.0
    lbeta = math.lgamma(a + b) - math.lgamma(a) - math.lgamma(b)
    front = math.exp(math.log(x) * a + math.log(1 - x) * b + lbeta)
    if x > (a + 1) / (a + b + 2):
        return 1 - betainc(b, a, 1 - x)
    # Lentz's continued fraction
    f, c, d = 1.0, 1.0, 0.0
    for i in range(400):
        m = i // 2
        if i == 0:
            num = 1.0
        elif i % 2 == 0:
            num = m * (b - m) * x / ((a + 2 * m - 1) * (a + 2 * m))
        else:
            num = -((a + m) * (a + b + m) * x) / ((a + 2 * m) * (a + 2 * m + 1))
        d = 1 + num * d
        d = 1 / (d if abs(d) > 1e-300 else 1e-300)
        c = 1 + num / (c if abs(c) > 1e-300 else 1e-300)
        f *= c * d
        if abs(1 - c * d) < 1e-14:
            break
    return front * (f - 1) / a


def holm(ps, alpha):
    out = set()
    for i, (k, p) in enumerate(sorted(ps.items(), key=lambda kv: kv[1])):
        if p > alpha / (len(ps) - i):
            break
        out.add(k)
    return out


def workers(r):
    names = [p.partition("=")[0] for p in str(r.get("observed_threads") or "").split()]
    return sorted(n for n in names if re.fullmatch(r"mibz2-\d+", n))


def machine(lscpu, env, errors):
    try:
        models = set(re.findall(r"^\s*Model name:\s*(.+?)\s*$", open(lscpu).read(), re.M))
    except OSError:
        models = set()
    if len(models) != 1 or "i9-10900K" not in next(iter(models)):
        errors.append(f"not the registered machine: lscpu names {sorted(models)}, not one i9-10900K")
    try:
        e = json.load(open(env))
        if e.get("cores") != 10 or e.get("cpus") != 20:
            errors.append(f"not the registered profile: {e.get('cores')} cores, {e.get('cpus')} CPUs, not 10 and 20")
    except (OSError, ValueError):
        errors.append(f"{env}: unreadable")
    return next(iter(models)) if len(models) == 1 else None


def load(path, counts, rounds, errors, check_workers=True):
    """{(corpus, threads, arm, round): record}, the reference arm canonical as `ref`"""
    rows, outs, refs, corpora = {}, {}, set(), set()
    if not os.path.exists(path):
        errors.append(f"{os.path.basename(path)}: missing")
        return rows, []
    for ln, line in enumerate(open(path), 1):
        if not line.strip():
            continue
        try:
            r = json.loads(line)
            arm = r["arm"]
            key = (r["corpus"], int(r["threads"]), CAND if arm == CAND else "ref", int(r["round"]))
        except (ValueError, KeyError, TypeError):
            errors.append(f"{os.path.basename(path)} line {ln}: not a record")
            continue
        if arm != CAND:
            if not arm.startswith(CAND + "@"):
                errors.append(f"{os.path.basename(path)} line {ln}: arm {arm!r} is neither {CAND} nor {CAND}@<v0.5.6>")
                continue
            refs.add(arm)
        if key in rows:
            errors.append(f"{key} duplicated")
            continue
        wall = r.get("wall")
        if r.get("termination") != "success" or r.get("err") or isinstance(wall, bool) \
                or not isinstance(wall, (int, float)) or not wall > 0 or not isinstance(r.get("rss_kb"), (int, float)):
            errors.append(f"{key}: failed, or no wall time or peak memory")
            continue
        t = key[1]
        if check_workers and t in WORKERS and workers(r) != sorted(f"mibz2-{k}" for k in range(WORKERS[t])):
            errors.append(f"{key}: observed workers {workers(r)}, not the shipped {WORKERS[t]} at {t} threads")
        outs.setdefault(key[0], set()).add((r.get("out"), r.get("sha256") or r.get("hash")))
        corpora.add(key[0])
        rows[key] = r
    if len(refs) > 1:
        errors.append(f"more than one reference arm: {sorted(refs)}")
    roster = {(c, t, a, i) for c in corpora for t in counts for a in (CAND, "ref") for i in range(rounds)}
    missing = sorted(roster - set(rows))
    extra = sorted(set(rows) - roster)
    if missing:
        errors.append(f"{len(missing)} of {len(roster)} records missing, e.g. {missing[:3]}")
    if extra:
        errors.append(f"{len(extra)} records outside the roster, e.g. {extra[:3]}")
    for c, o in sorted(outs.items()):
        if len(o) != 1:
            errors.append(f"{c}: the outputs differ: {sorted(o, key=str)[:3]}")
    return rows, sorted(corpora)


def cell(rows, corpus, t, rounds):
    xs = [math.log(rows[(corpus, t, CAND, i)]["wall"] / rows[(corpus, t, "ref", i)]["wall"]) for i in range(rounds)]
    m, se = statistics.mean(xs), statistics.stdev(xs) / math.sqrt(len(xs))
    h = t975(len(xs) - 1) * se
    mem = statistics.median(rows[(corpus, t, CAND, i)]["rss_kb"] for i in range(rounds)) / \
        statistics.median(rows[(corpus, t, "ref", i)]["rss_kb"] for i in range(rounds))
    return math.exp(m), math.exp(m - h), math.exp(m + h), p_gain(xs), mem


def label(lo, hi):
    return "FASTER" if hi < 1 else "SLOWER" if lo > 1 else "INCONCLUSIVE"


def judge(a):
    counts = [int(t) for t in a.counts.split(",")]
    errors = []
    if os.path.exists(os.path.join(os.path.dirname(os.path.abspath(a.runs)), "DRY-RUN.txt")):
        errors.append("a dry run (DRY-RUN.txt beside the records): not evidence")
    model = machine(a.lscpu, a.env, errors)
    rows, corpora = load(a.runs, counts, a.rounds, errors)
    srows, scorp = load(a.sustained, [10], a.sustained_rounds, errors)
    for need in (MAIN, HELDOUT):
        if rows and need not in corpora:
            errors.append(f"{need}: not in the run")
    if srows and scorp != [SUSTAINED]:
        errors.append(f"the sustained run's corpora are {scorp}, not [{SUSTAINED}]")
    print(f"records {len(rows)} and {len(srows)} sustained; counts {counts}; {a.rounds} rounds "
          f"({a.sustained_rounds} sustained); corpora {corpora}" + (f"; the machine: {model}" if model else ""))
    if errors:
        print(f"VOID: the records do not make the registered run ({len(errors)} problems); no verdict")
        for e in errors[:25]:
            print(f"  {e}")
        return 1
    fails, ps, res = [], {}, {}
    print("\nthe port over v0.5.6: wall, the rounds' paired ratios, geometric mean [95% interval]; "
          "one-sided p (gain); peak RSS, median over median")
    for t in counts:
        print(f"  {t} threads")
        for c in corpora:
            g, lo, hi, p, mem = res[(c, t)] = cell(rows, c, t, a.rounds)
            role = "primary" if c == MAIN else "held-out replication" if c == HELDOUT else "guard"
            print(f"    {c} ({role}): {g:.4f} [{lo:.4f}, {hi:.4f}] {label(lo, hi)}; p {p:.2e}; memory {mem:.3f}")
            if c == MAIN:
                ps[t] = p
    rejected = holm(ps, ALPHA)
    print(f"\nP1, wall on {MAIN}: Holm at α {ALPHA} over {len(ps)} cells: "
          + ", ".join(f"{t} threads p {ps[t]:.2e} {'rejects' if t in rejected else 'does not'}" for t in counts))
    for t in counts:
        if t not in rejected:
            fails.append(f"P1: no gain on {MAIN} at {t} threads")
        g, lo, hi, p, mem = res[(MAIN, t)]
        if mem > 1.0:
            fails.append(f"P2: peak RSS on {MAIN} at {t} threads {mem:.3f}, above 1.00")
        g, lo, hi, p, mem = res[(HELDOUT, t)]
        if not hi < 1:
            fails.append(f"H: {HELDOUT} at {t} threads {g:.4f} [{lo:.4f}, {hi:.4f}], not FASTER")
        for c in corpora:
            if c in (MAIN, HELDOUT):
                continue
            g, lo, hi, p, mem = res[(c, t)]
            if lo > 1:
                fails.append(f"G: {c} at {t} threads SLOWER, {g:.4f} [{lo:.4f}, {hi:.4f}]")
            elif g > GUARD_MAX:
                fails.append(f"G: {c} at {t} threads {g:.4f}, above the bound {GUARD_MAX}")
            if mem > RSS_GATE:
                fails.append(f"G: {c} at {t} threads peak RSS {mem:.3f}, above {RSS_GATE}")
    g, lo, hi, p, mem = cell(srows, SUSTAINED, 10, a.sustained_rounds)
    print(f"\nS, sustained: 10 threads back to back on {SUSTAINED}: {g:.4f} [{lo:.4f}, {hi:.4f}] {label(lo, hi)}; "
          f"memory {mem:.3f}")
    if lo > 1:
        fails.append(f"S: sustained SLOWER, {g:.4f} [{lo:.4f}, {hi:.4f}]")
    elif g > GUARD_MAX:
        fails.append(f"S: sustained {g:.4f}, above the bound {GUARD_MAX}")
    if fails:
        print(f"\nVERDICT: NOT ADOPTED. {len(fails)} conditions fail:")
        for f in fails:
            print(f"  {f}")
    else:
        print("\nVERDICT: ADOPTS. A wall gain on Wikidata at every count (Holm), peak RSS not above "
              "v0.5.6's, the gain replicated on held-out Wikidata, every guard within its bound, and "
              "the sustained cell within its bound.")
    return 0


def pairs(a):
    errors = []
    recs = {}
    for ln, line in enumerate(open(a.runs), 1):
        if line.strip():
            r = json.loads(line)
            if r.get("termination") != "success":
                errors.append(f"line {ln}: failed")
            recs[(r["corpus"], int(r["threads"]), r["arm"], int(r["round"]))] = r
    if errors:
        print(f"VOID: {len(errors)} records failed")
        return 1
    for spec in a.pair:
        x, y = spec.split("/")
        print(f"{x} over {y}: wall, geometric mean [95% interval]; peak RSS, median over median")
        for (c, t) in sorted({(k[0], k[1]) for k in recs if k[2] == x}, key=lambda k: (k[1], k[0])):
            n = sorted(i for (cc, tt, arm, i) in recs if (cc, tt, arm) == (c, t, x) and (c, t, y, i) in recs)
            if len(n) < 2:
                continue
            xs = [math.log(recs[(c, t, x, i)]["wall"] / recs[(c, t, y, i)]["wall"]) for i in n]
            m, h = statistics.mean(xs), t975(len(xs) - 1) * statistics.stdev(xs) / math.sqrt(len(xs))
            mem = statistics.median(recs[(c, t, x, i)]["rss_kb"] for i in n) / \
                statistics.median(recs[(c, t, y, i)]["rss_kb"] for i in n)
            print(f"  W {t:2d} {c}: {math.exp(m):.4f} [{math.exp(m - h):.4f}, {math.exp(m + h):.4f}] "
                  f"{label(math.exp(m - h), math.exp(m + h))}; memory {mem:.3f}")
    return 0


def main(argv):
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="mode", required=True)
    j = sub.add_parser("judge")
    j.add_argument("runs")
    j.add_argument("sustained")
    j.add_argument("--lscpu", required=True)
    j.add_argument("--env", required=True)
    j.add_argument("--counts", default="8,10,20")
    j.add_argument("--rounds", type=int, default=12)
    j.add_argument("--sustained-rounds", type=int, default=8)
    q = sub.add_parser("pairs")
    q.add_argument("runs")
    q.add_argument("--pair", action="append", required=True)
    a = ap.parse_args(argv[1:])
    return judge(a) if a.mode == "judge" else pairs(a)


if __name__ == "__main__":
    sys.exit(main(sys.argv))
