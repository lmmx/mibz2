"""The level-aware plan's A/B, scored against its pre-registered gates
(docs/journal/2026-10-29-level-aware-plan.md).

Reads bench/coop_decoder.py's raw runs (one .jsonl a corpus) and, for each corpus, pairs
the default decoder (`mibz2`: `BzReader::parallel`, which plans for the input's first
stream's level) against today's plan (`mibz2-auto`: `Plan::for_threads`, blind to the
level) by round, at each thread count: throughput (the geometric mean of the rounds'
paired ratios and its 95% t-interval), and peak RSS and CPU-seconds as ratios of the arms'
medians. Each corpus is scored by its role, from its name:

- `bz1`, `bz2`, `bz3` (the rule's levels): at all CPUs, a geometric mean >= 1.05 with its
  lower bound > 1.00, RSS <= 1.15x, CPU-seconds <= 1.5x;
- `bz6`, the native corpus, `mix93` (-9 then -3: its first stream plans as today):
  unchanged, the lower bound >= 0.97 and RSS <= 1.05x;
- `mix39` (-3 then -9: the rule engaged, the gate bounding the -9 stream's blocks): no
  slower, the lower bound >= 0.97, RSS <= 1.15x, CPU-seconds <= 1.5x.

Every run must give its corpus's one output hash, with no error.

With `--pool`, a corpus given in several files (a repeat of a run) is also scored pooled:
every file's paired rounds together, one statistic over them all, each run's verdict
printed as well and none dropped.

    python3 bench/level_aware.py coop-decoder-*.jsonl [--new mibz2] [--old mibz2-auto] [--pool]
"""

import argparse
import json
import math
import statistics
import sys

# Student's t, 97.5% quantile, df 1-30
T975 = [12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179,
        2.160, 2.145, 2.131, 2.120, 2.110, 2.101, 2.093, 2.086, 2.080, 2.074, 2.069, 2.064,
        2.060, 2.056, 2.052, 2.048, 2.045, 2.042]  # fmt: skip

RULE = ("bz1", "bz2", "bz3")
UNCHANGED = ("bz6", "native", "mix93")
MIXED = ("mix39",)


def role(corpus):
    """The corpus's role, from its name (`wikidata-bz3` -> `bz3`; `wikidata` -> native)"""
    tag = corpus.split("-", 1)[1] if "-" in corpus else "native"
    if tag in RULE:
        return tag, "rule"
    if tag in UNCHANGED:
        return tag, "unchanged"
    if tag in MIXED:
        return tag, "mixed"
    return tag, None


def paired(new, old):
    """The geometric mean of old's wall over new's, paired by round, and its 95% interval"""
    logs = [math.log(old[r]["wall"] / new[r]["wall"]) for r in new if r in old]
    m = statistics.mean(logs)
    if len(logs) < 2:
        return math.exp(m), float("nan"), float("nan"), len(logs)
    h = T975[min(len(logs) - 1, 30) - 1] * statistics.stdev(logs) / math.sqrt(len(logs))
    return math.exp(m), math.exp(m - h), math.exp(m + h), len(logs)


def ratio(new, old, key):
    return statistics.median(r[key] for r in new.values()) / statistics.median(
        r[key] for r in old.values()
    )


def gate(kind, g, lo, rss, cpu, clean):
    if not clean:
        return False
    if kind == "rule":
        return g >= 1.05 and lo > 1.00 and rss <= 1.15 and cpu <= 1.5
    if kind == "unchanged":
        return lo >= 0.97 and rss <= 1.05
    if kind == "mixed":
        return lo >= 0.97 and rss <= 1.15 and cpu <= 1.5
    return False


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("files", nargs="+")
    ap.add_argument("--new", default="mibz2")
    ap.add_argument("--old", default="mibz2-auto")
    ap.add_argument("--pool", action="store_true", help="also score each corpus's files pooled")
    a = ap.parse_args()
    verdicts = []
    loaded = []
    for i, f in enumerate(a.files):
        runs, hashes, errors, corpus = {}, set(), 0, None
        for line in open(f):
            r = json.loads(line)
            corpus = r.get("corpus", corpus)
            if r.get("err"):
                errors += 1
                continue
            hashes.add(r["hash"])
            runs.setdefault((r["arm"], r["threads"]), {})[r["round"]] = r
        loaded.append((i, corpus, runs, hashes, errors))
        tag, kind = role(corpus or "?")
        threads = sorted({t for (arm, t) in runs if arm == a.new and (a.old, t) in runs})
        if not threads:
            print(f"{f}: no {a.new} and {a.old} runs at the same thread count", file=sys.stderr)
            continue
        clean = len(hashes) == 1 and errors == 0
        print(
            f"## {corpus} ({tag}: {kind or 'not scored'}); {len(hashes)} hash(es), {errors} error(s)"
        )
        print()
        print(f"| threads | {a.new} / {a.old} | rounds | peak RSS | CPU-s |")
        print("|--:|--:|--:|--:|--:|")
        for t in threads:
            new, old = runs[(a.new, t)], runs[(a.old, t)]
            g, lo, hi, n = paired(new, old)
            rss, cpu = ratio(new, old, "rss_kb"), ratio(new, old, "cpu")
            print(f"| {t} | {g:.3f} ({lo:.3f}-{hi:.3f}) | {n} | {rss:.2f} | {cpu:.2f} |")
        top = threads[-1]
        new, old = runs[(a.new, top)], runs[(a.old, top)]
        g, lo, _, _ = paired(new, old)
        ok = gate(kind, g, lo, ratio(new, old, "rss_kb"), ratio(new, old, "cpu"), clean)
        if kind:
            verdicts.append((corpus, kind, ok))
            print(f"\nAt {top} threads, the {kind} gate: **{'met' if ok else 'not met'}**\n")
        else:
            print()
    if a.pool:
        by_corpus = {}
        for i, corpus, runs, hashes, errors in loaded:
            by_corpus.setdefault(corpus, []).append((i, runs, hashes, errors))
        for corpus, parts in by_corpus.items():
            if len(parts) < 2:
                continue
            tag, kind = role(corpus or "?")
            # every file's rounds, keyed by (file, round): pairs never cross files
            new, old, hashes, errors = {}, {}, set(), 0
            top = min(
                max(t for (arm, t) in runs if arm == a.new and (a.old, t) in runs)
                for _, runs, _, _ in parts
            )
            for i, runs, h, e in parts:
                new.update({(i, r): v for r, v in runs[(a.new, top)].items()})
                old.update({(i, r): v for r, v in runs[(a.old, top)].items()})
                hashes |= h
                errors += e
            g, lo, hi, n = paired(new, old)
            rss, cpu = ratio(new, old, "rss_kb"), ratio(new, old, "cpu")
            clean = len(hashes) == 1 and errors == 0
            ok = gate(kind, g, lo, rss, cpu, clean)
            print(f"## {corpus}, pooled over {len(parts)} runs ({tag}: {kind or 'not scored'})\n")
            print(
                f"At {top} threads: {g:.3f} ({lo:.3f}-{hi:.3f}), {n} rounds; RSS {rss:.2f}, CPU {cpu:.2f}; {len(hashes)} hash(es), {errors} error(s)"
            )
            if kind:
                print(
                    f"\nThe {kind} gate, pooled: **{'met' if ok else 'not met'}** (the runs' own verdicts above stand)\n"
                )
                verdicts.append((f"{corpus} (pooled)", kind, ok))
    if verdicts:
        print("## Verdict\n")
        for corpus, kind, ok in verdicts:
            print(f"- {corpus} ({kind}): {'met' if ok else 'not met'}")
        print(f"\nAll met: **{'yes' if all(ok for _, _, ok in verdicts) else 'no'}**")


if __name__ == "__main__":
    main()
