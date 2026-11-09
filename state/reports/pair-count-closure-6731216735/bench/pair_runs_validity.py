"""The validity of a `sweep.yml` `tool: pair_runs` run (docs/journal/2026-11-09-pair-from-runs.md
§5), and its registered readings, recomputed from the raw paired timings.

    python3 bench/pair_runs_validity.py DIR

DIR holds the run's `plan.json`, written by the workflow before any copy starts, and every
copy's `sweep-ID.txt`, `status-ID.txt` and `raw-ID.csv`. The run is VALID only if all hold:

- the plan: the tool, rounds 2-31, a finite positive clock (GHZ), the CPUs and the exact copy
  set they imply (`1`, then `N-cpuC` for each of the N CPUs); the head 40 hex digits and the
  frozen head named at dispatch, the same; the corpus's SHA-256 the spec's and the plan's;
- the inventory: for each planned copy exactly its three files, and no file of a copy not
  planned (a missing child is found from the plan, not from what survived); each status `0`;
  each output with its VALID line and no NOT VALID;
- each raw CSV: its header; the exact corpus x round x arm matrix (the files below, every
  round once, the six arms), nothing missing, duplicated or extra; every total finite and
  positive, but a build arm's (row, run, one) exactly 0 on a file with no paired block; the
  file's paired blocks, paired rows and output bytes the same in every row and every copy,
  and Wikidata with paired blocks;
- the counts (each file's paired blocks, paired rows and output bytes, which the readings
  divide by): in their domains (whole numbers, paired rows 0 exactly when paired blocks are,
  each paired block 2^16 to 900,000 rows, output positive); the same in every planned copy;
  equal to the census each copy logged for the file (its blocks, output, rows, paired blocks
  and paired rows); and, for Wikidata, its output the spec's decoded bytes, its blocks the
  spec's windows' blocks, and its logged corpus identity the spec's. So no denominator can
  differ between copies, from the copy's own log, or (for Wikidata) from the pinned spec;
- the readings: from the single copy's raw timings, every registered interval and difference
  present with finite, ordered endpoints.

Only then are the readings printed, recomputed here as the harness computes them; otherwise
NOT VALID and the problems, and no reading. Exits 0 when VALID, 1 when not.
"""

import csv
import hashlib
import json
import math
import re
import sys
from pathlib import Path

T95 = [
    12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179, 2.160,
    2.145, 2.131, 2.120, 2.110, 2.101, 2.093, 2.086, 2.080, 2.074, 2.069, 2.064, 2.060, 2.056,
    2.052, 2.048, 2.045, 2.042,
]
HEADER = ["file", "round", "arm", "seconds", "paired_blocks", "paired_rows", "out_bytes"]
ARMS = ["row", "run", "one", "P", "R", "O"]
BUILD_ARMS = {"row", "run", "one"}
# examples/pair_runs.rs's files: the corpus it is given, then tests/common's synthetic ones
SYNTHETIC = ["text", "random", "one byte", "pattern", "runs", "records"]
HEX40 = re.compile(r"[0-9a-f]{40}")
# bwt::MIN_SEGMENTED, and bzip2's largest block: a paired (segmented) block's rows lie between
MIN_SEGMENTED = 1 << 16
MAX_BLOCK = 900_000
# examples/pair_runs.rs's census line, one a file
CENSUS = re.compile(
    r"^(?P<file>.+): (?P<blocks>\d+) blocks, (?P<mb>\d+\.\d) MB of output, (?P<rows>\d+) rows; "
    r"paired (?P<paired>\d+) blocks \((?P<paired_rows>\d+) rows\)",
    re.M,
)
IDENTITY = re.compile(r"^corpus identity: (?P<bytes>\d+) bytes, (?P<decoded>\d+) decoded", re.M)


def copies_for(nproc):
    return ["1"] + [f"{nproc}-cpu{c}" for c in range(nproc)]


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def ratio(rounds, num, den):
    """The paired geometric mean of num / den and its 95% t-interval, or None"""
    try:
        logs = [math.log(r[num] / r[den]) for r in rounds]
    except (ValueError, ZeroDivisionError):
        return None
    if len(logs) < 2 or not all(math.isfinite(x) for x in logs):
        return None
    m = sum(logs) / len(logs)
    var = sum((x - m) ** 2 for x in logs) / (len(logs) - 1)
    h = T95[len(logs) - 2] * math.sqrt(var / len(logs))
    iv = (math.exp(m), math.exp(m - h), math.exp(m + h))
    return iv if all(math.isfinite(x) for x in iv) and iv[1] <= iv[2] else None


def diff(rounds, a, b, scale):
    """The paired mean of (a - b) x scale and its 95% t-interval, or None"""
    if not (math.isfinite(scale) and scale > 0):
        return None
    d = [(r[a] - r[b]) * scale for r in rounds]
    if len(d) < 2 or not all(math.isfinite(x) for x in d):
        return None
    m = sum(d) / len(d)
    var = sum((x - m) ** 2 for x in d) / (len(d) - 1)
    h = T95[len(d) - 2] * math.sqrt(var / len(d))
    iv = (m, m - h, m + h)
    return iv if all(math.isfinite(x) for x in iv) and iv[1] <= iv[2] else None


def check_plan(d, problems):
    try:
        plan = json.loads((d / "plan.json").read_text())
    except (OSError, ValueError) as e:
        problems.append(f"plan.json: {e}")
        return None
    if plan.get("tool") != "pair_runs":
        problems.append(f"plan: tool {plan.get('tool')!r}")
    rounds = plan.get("rounds")
    if not (isinstance(rounds, int) and 2 <= rounds <= 31):
        problems.append(f"plan: rounds {rounds!r}, not 2 to 31")
    ghz = plan.get("ghz")
    if not (isinstance(ghz, (int, float)) and math.isfinite(ghz) and ghz > 0):
        problems.append(f"plan: GHZ {ghz!r} is not finite and positive")
    nproc = plan.get("nproc")
    if not (isinstance(nproc, int) and nproc >= 1):
        problems.append(f"plan: nproc {nproc!r}")
    elif plan.get("copies") != copies_for(nproc):
        problems.append(f"plan: copies {plan.get('copies')!r}, not {copies_for(nproc)} for {nproc} CPUs")
    head, frozen = plan.get("head", ""), plan.get("frozen", "")
    if not HEX40.fullmatch(str(head)):
        problems.append(f"plan: head {head!r} is not 40 hex digits")
    if not HEX40.fullmatch(str(frozen)):
        problems.append(f"plan: frozen head {frozen!r} is not 40 hex digits (none named at dispatch)")
    elif frozen != head:
        problems.append(f"plan: head {head} is not the frozen head {frozen}")
    try:
        spec = json.loads((d / plan["spec"]).read_text())
        got = sha256(d / plan["corpus"])
        if got != spec.get("corpus_sha256"):
            problems.append(f"corpus SHA-256 {got}, not the spec's {spec.get('corpus_sha256')}")
        if got != plan.get("corpus_sha256"):
            problems.append(f"corpus SHA-256 {got}, not the plan's {plan.get('corpus_sha256')}")
    except (KeyError, OSError, ValueError) as e:
        problems.append(f"corpus identity: {e!r}")
    return plan


def check_inventory(d, copies, problems):
    planned = set(copies)
    for kind, pattern in [("sweep", "sweep-*.txt"), ("status", "status-*.txt"), ("raw", "raw-*.csv")]:
        for f in d.glob(pattern):
            ident = f.name[len(kind) + 1 :].rsplit(".", 1)[0]
            if ident not in planned:
                problems.append(f"{f.name}: not a planned copy")
    for c in copies:
        for name in [f"sweep-{c}.txt", f"status-{c}.txt", f"raw-{c}.csv"]:
            if not (d / name).is_file():
                problems.append(f"copy {c}: {name} missing")
        st = d / f"status-{c}.txt"
        if st.is_file() and st.read_text().strip() != "0":
            problems.append(f"copy {c}: exit status {st.read_text().strip()!r}")
        sw = d / f"sweep-{c}.txt"
        if sw.is_file():
            text = sw.read_text()
            if not re.search(r"^VALID \(this process", text, re.M) or "NOT VALID" in text:
                problems.append(f"copy {c}: no VALID line")


def read_raw(path, rounds, corpus, problems):
    """The copy's matrix: {file: {round: [seconds x 6]}} and its per-file counts, or None"""
    label = path.name
    try:
        with path.open(newline="") as f:
            rows = list(csv.reader(f))
    except OSError as e:
        problems.append(f"{label}: {e}")
        return None
    if not rows or rows[0] != HEADER:
        problems.append(f"{label}: header {rows[0] if rows else None!r}")
        return None
    files = [corpus] + [f"synthetic {s}" for s in SYNTHETIC]
    m = {f: {} for f in files}
    meta = {}
    bad = 0
    for row in rows[1:]:
        if len(row) != len(HEADER):
            problems.append(f"{label}: a row of {len(row)} fields: {row!r}")
            bad += 1
            continue
        f, r, arm, sec, pb, pr, ob = row
        try:
            r, sec, counts = int(r), float(sec), (int(pb), int(pr), int(ob))
        except ValueError:
            problems.append(f"{label}: an unparsable row {row!r}")
            bad += 1
            continue
        if f not in m:
            problems.append(f"{label}: file {f!r} not registered")
            continue
        if not (0 <= r < rounds) or arm not in ARMS:
            problems.append(f"{label}: {f}: round {r} arm {arm!r} outside the matrix")
            continue
        slot = m[f].setdefault(r, [None] * 6)
        a = ARMS.index(arm)
        if slot[a] is not None:
            problems.append(f"{label}: {f}: round {r} arm {arm} twice")
        slot[a] = sec
        if meta.setdefault(f, counts) != counts:
            problems.append(f"{label}: {f}: its paired blocks, rows or output change between rows")
    for f in files:
        for r in range(rounds):
            slot = m[f].get(r)
            for a, arm in enumerate(ARMS):
                sec = slot[a] if slot else None
                if sec is None:
                    problems.append(f"{label}: {f}: round {r} arm {arm} missing")
                    continue
                paired = meta.get(f, (0, 0, 0))[0]
                if arm in BUILD_ARMS and paired == 0:
                    if sec != 0:
                        problems.append(f"{label}: {f}: build arm {arm} {sec} with no paired block")
                elif not (math.isfinite(sec) and sec > 0):
                    problems.append(f"{label}: {f}: round {r} arm {arm} total {sec}")
    if meta.get(corpus, (0, 0, 0))[0] == 0:
        problems.append(f"{label}: {corpus}: no paired block")
    for f in files:
        if f not in meta:
            continue
        pb, pr, ob = meta[f]
        if pb < 0 or pr < 0 or ob <= 0:
            problems.append(f"{label}: {f}: counts {meta[f]} outside their domain")
        elif (pb == 0) != (pr == 0):
            problems.append(f"{label}: {f}: {pb} paired blocks but {pr} paired rows")
        elif not (pb * MIN_SEGMENTED <= pr <= pb * MAX_BLOCK):
            problems.append(f"{label}: {f}: {pr} paired rows over {pb} paired blocks, not 2^16 to 900,000 a block")
    if bad:
        return None
    return {f: [m[f][r] for r in sorted(m[f])] for f in files}, meta


def readings(matrix, meta, corpus, ghz, problems):
    """The registered intervals of the single copy, required present, and the readings"""
    times = matrix[corpus]
    paired_rows = meta[corpus][1]
    iv = {
        "Q1": ratio(times, 1, 0),
        "M1 ratio": ratio(times, 1, 2),
        "M1 difference": diff(times, 1, 2, ghz * 1e9 / paired_rows) if paired_rows else None,
        "Q2": ratio(times, 4, 3),
        "O/P": ratio(times, 5, 3),
        "R/O": ratio(times, 4, 5),
    }
    for k, v in iv.items():
        if v is None:
            problems.append(f"{corpus}: interval {k} missing")
    worst = None
    for s in SYNTHETIC:
        f = f"synthetic {s}"
        q = ratio(matrix[f], 4, 3)
        if q is None:
            problems.append(f"{f}: interval Q2 missing")
        elif worst is None or q[0] > worst[1]:
            worst = (f, q[0])
    if problems:
        return []
    out = []
    g, lo, hi = iv["Q1"]
    r = ("POSITIVE" if g <= 0.50 and hi < 0.60 else "NEGATIVE STOP (closes this registered candidate)" if lo > 0.80 else "INCONCLUSIVE")
    out.append(f"Q1 R_build {g:.4} ({lo:.4}, {hi:.4}): {r}")
    m, lo, hi = iv["M1 difference"]
    r = "MET" if hi <= 1.0 else "NOT MET" if lo > 1.0 else "UNRESOLVED"
    out.append(f"M1 run - one-hop {m:.3} nominal c/B at {ghz} GHz ({lo:.3}, {hi:.3}): {r} (exploratory; clock-scaled wall time, not cycles)")
    g, lo, hi = iv["Q2"]
    if g <= 0.95 and hi < 1.0 and worst[1] <= 1.05:
        r = "POSITIVE"
    elif lo > 1.0:
        r = "NEGATIVE STOP (pauses this candidate)"
    elif g <= 0.95 and hi < 1.0:
        r = "STOP (a synthetic point estimate above 1.05: a screen)"
    else:
        r = "INCONCLUSIVE"
    out.append(f"Q2 R_whole {g:.4} ({lo:.4}, {hi:.4}); worst synthetic {worst[1]:.4} ({worst[0]}): {r}")
    for k in ["O/P", "R/O"]:
        g, lo, hi = iv[k]
        out.append(f"ref {k} {g:.4} ({lo:.4}, {hi:.4})")
    return out


def check_counts(d, plan, metas, files, problems):
    """Each file's counts the same in every copy, equal to each copy's logged census, and
    Wikidata's to the pinned spec"""
    corpus = plan.get("corpus", "")
    try:
        spec = json.loads((d / plan["spec"]).read_text())
    except (KeyError, OSError, ValueError) as e:
        problems.append(f"spec: {e!r}")
        spec = {}
    first = None
    for c, meta in metas.items():
        if first is None:
            first = (c, meta)
        for f in files:
            if meta.get(f) != first[1].get(f):
                problems.append(f"{f}: counts {meta.get(f)} in copy {c}, {first[1].get(f)} in copy {first[0]}")
        sw = d / f"sweep-{c}.txt"
        text = sw.read_text() if sw.is_file() else ""
        census = {}
        for m in CENSUS.finditer(text):
            if m["file"] in census:
                problems.append(f"copy {c}: {m['file']}: two census lines")
            census[m["file"]] = m
        for f in files:
            m = census.get(f)
            if m is None:
                problems.append(f"copy {c}: {f}: no census line in its log")
                continue
            if f not in meta:
                continue
            pb, pr, ob = meta[f]
            logged = (int(m["paired"]), int(m["paired_rows"]))
            if (pb, pr) != logged:
                problems.append(f"copy {c}: {f}: raw counts {(pb, pr)}, its log {logged}")
            if abs(ob / 1e6 - float(m["mb"])) > 0.05 + 1e-9:
                problems.append(f"copy {c}: {f}: raw output {ob} bytes, its log {m['mb']} MB")
            if pb > int(m["blocks"]) or pr > int(m["rows"]):
                problems.append(f"copy {c}: {f}: more paired than logged blocks or rows")
        ident = IDENTITY.search(text)
        want = (spec.get("corpus_bytes"), spec.get("decoded_bytes"))
        if ident is None or (int(ident["bytes"]), int(ident["decoded"])) != want:
            problems.append(f"copy {c}: logged corpus identity {ident and ident.group(0)!r}, the spec's {want}")
        if corpus in meta and meta[corpus][2] != spec.get("decoded_bytes"):
            problems.append(f"copy {c}: {corpus}: raw output {meta[corpus][2]}, the spec's {spec.get('decoded_bytes')}")
        blocks = sum(w.get("blocks", 0) for w in spec.get("windows", []))
        if corpus in census and int(census[corpus]["blocks"]) != blocks:
            problems.append(f"copy {c}: {corpus}: {census[corpus]['blocks']} blocks logged, the spec's windows {blocks}")


def validate(d):
    """(VALID, the lines to print)"""
    d = Path(d)
    problems = []
    plan = check_plan(d, problems)
    if plan is None:
        return False, ["NOT VALID (no readings):"] + [f"  - {p}" for p in problems]
    copies = plan.get("copies") if isinstance(plan.get("copies"), list) else []
    check_inventory(d, copies, problems)
    rounds = plan.get("rounds") if isinstance(plan.get("rounds"), int) else 0
    corpus = plan.get("corpus", "")
    single = None
    metas = {}
    for c in copies:
        p = d / f"raw-{c}.csv"
        if p.is_file():
            got = read_raw(p, rounds, corpus, problems)
            if got is not None:
                metas[c] = got[1]
            if c == "1":
                single = got
    files = [corpus] + [f"synthetic {s}" for s in SYNTHETIC]
    check_counts(d, plan, metas, files, problems)
    lines = [f"plan: head {plan.get('head')}, frozen {plan.get('frozen')}, copies {copies}, GHZ {plan.get('ghz')}"]
    out = []
    if single is not None and not problems:
        out = readings(single[0], single[1], corpus, plan["ghz"], problems)
    if problems:
        return False, lines + ["NOT VALID (no readings):"] + [f"  - {p}" for p in problems]
    return True, lines + ["VALID", "readings (UNJUDGED unless this is the registered run):"] + [f"  {x}" for x in out]


def main():
    ok, lines = validate(sys.argv[1] if len(sys.argv) > 1 else ".")
    print("\n".join(lines))
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
