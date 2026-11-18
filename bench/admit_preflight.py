#!/usr/bin/env python3
"""`just bench-admit`'s pre-flight, before any timing (#145, the Researcher's REVIEW 6974376850):
the run happens only on the registered machine, plans, inputs and census. Standard library only.

    admit_preflight.py --built BUILT.jsonl --census CENSUS.jsonl --inputs INPUTS.sha256

**What it checks** (any failure: exit 1, the recipe stops before timing and keeps the bundle):
- *the machine and the plans* (`ab built mibz2,mibz2-wide 8,10`): at each count, both arms on
  the measured i9 (`measured_i9`) with this process's affinity every CPU (0-19); at 8 threads
  eight workers with 32 in flight, at 10 threads seven with 28, the same in both arms; the
  kernels the lazy kernel (`CellsVariant<1>`) in `mibz2` and the wide arm's (`CellsVariant<4>`)
  in `mibz2-wide`;
- *the census on the exact inputs* (`ab census wikidata.bz2 text.bz2 random.bz2`): the Wikidata
  sample's blocks as registered (196 both admit, 32 the wide arm alone, 155 neither, 48 the
  column path, none the lazy kernel alone); text and random with no block admitted by either;
  no block's two outputs differing;
- *the guard inputs* (`sha256sum` of `text.bz2` and `random.bz2`, generated afresh by `ab gen` at
  scale 16 for this run): the registered digests.
"""
import argparse
import json
import sys

PLANS = {8: (8, 32), 10: (7, 28)}
KERNELS = {"mibz2": "mibz2::cells::CellsVariant<1>", "mibz2-wide": "mibz2::cells::CellsVariant<4>"}
AFFINITY = "0-19"
CENSUS = {
    "wikidata.bz2": {"blocks": 431, "column": 48, "both": 196, "wide_only": 32, "lazy_only": 0, "neither": 155,
                     "differ": 0},
    "text.bz2": {"column": 0, "both": 0, "wide_only": 0, "lazy_only": 0, "differ": 0},
    "random.bz2": {"both": 0, "wide_only": 0, "lazy_only": 0, "differ": 0},
}
# `ab gen DIR 16` at this branch's sources (the text corpus is the crate's own `src`, `tests`
# and `examples`): the guards the registration names
DIGESTS = {
    "text.bz2": "088478a5c9980c3ad26706aa7a3e2d287148b3b0860bcd0e6f23fe28e9dc9262",
    "random.bz2": "f24831b171e595ef5e93d9f3f56fd267ec3122cfa482f55cf6c73547c0af20e1",
}


def lines(path, errors):
    try:
        with open(path) as f:
            return [json.loads(line) for line in f if line.strip()]
    except (OSError, ValueError) as e:
        errors.append(f"{path}: unreadable ({e})")
        return []


def check_built(rows, errors):
    seen = set()
    for r in rows:
        try:
            arm, t = r["arm"], int(r["threads"])
        except (KeyError, TypeError, ValueError):
            errors.append(f"built: a malformed record {r!r}")
            continue
        seen.add((arm, t))
        if arm not in KERNELS or t not in PLANS:
            errors.append(f"built: {arm} at {t} threads, not registered")
            continue
        if r.get("measured_i9") is not True:
            errors.append(f"built: {arm} at {t}: not the measured i9 ({r.get('measured_i9')!r})")
        if r.get("cpus_allowed") != AFFINITY:
            errors.append(f"built: {arm} at {t}: the affinity {r.get('cpus_allowed')!r}, not {AFFINITY}")
        if (r.get("workers"), r.get("window")) != PLANS[t]:
            errors.append(f"built: {arm} at {t}: {r.get('workers')} workers and {r.get('window')} in flight, "
                          f"not {PLANS[t][0]} and {PLANS[t][1]}")
        if r.get("kernel") != KERNELS[arm]:
            errors.append(f"built: {arm} at {t}: the kernel {r.get('kernel')!r}, not {KERNELS[arm]}")
    for arm in KERNELS:
        for t in PLANS:
            if (arm, t) not in seen:
                errors.append(f"built: {arm} at {t} threads missing")


def check_census(rows, errors):
    by = {r.get("file"): r for r in rows}
    for f, want in CENSUS.items():
        r = by.get(f)
        if r is None:
            errors.append(f"census: {f} missing")
            continue
        for k, v in want.items():
            if r.get(k) != v:
                errors.append(f"census: {f} {k} {r.get(k)!r}, not {v}")


def check_inputs(path, errors):
    got = {}
    try:
        with open(path) as f:
            for line in f:
                parts = line.split()
                if len(parts) == 2:
                    got[parts[1].rsplit("/", 1)[-1]] = parts[0]
    except OSError as e:
        errors.append(f"{path}: unreadable ({e})")
        return
    for f, digest in DIGESTS.items():
        if got.get(f) != digest:
            errors.append(f"inputs: {f} {got.get(f)!r}, not the registered {digest}")


def main(argv):
    ap = argparse.ArgumentParser()
    ap.add_argument("--built", required=True)
    ap.add_argument("--census", required=True)
    ap.add_argument("--inputs", required=True)
    a = ap.parse_args(argv[1:])
    errors = []
    check_built(lines(a.built, errors), errors)
    check_census(lines(a.census, errors), errors)
    check_inputs(a.inputs, errors)
    if errors:
        print(f"PRE-FLIGHT FAILED: not the registered run ({len(errors)} problems); nothing timed")
        for e in errors:
            print(f"  {e}")
        return 1
    print("pre-flight: the measured i9, every CPU, 8/32 and 7/28 in both arms, the kernels, the census and "
          "the guards' digests as registered")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
