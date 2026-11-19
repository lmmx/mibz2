#!/usr/bin/env python3
"""The open port's adoption verdict (`just bench-open-port`; the C6 registration
docs/journal/2026-11-19-open-port-registration.md; #145 CHECKPOINT 7004759994): the port's
release build (`mibz2@…/build-port/…/ab`) against v0.5.6's (`mibz2@…/build-v0.5.6/…/ab`), each
its own binary run as `ab run`'s child, both through the same public entry point
(`BzReader::parallel`), unpinned on the i9 at the shipped thread counts.

    open_port.py judge BUNDLE --port PORT_COMMIT
    open_port.py guard TARGET --this RUN_DIR [--remote origin]   (the one attempt; see guard)
    open_port.py pairs RUNS.jsonl --pair X/Y [--pair X/Y ...]   (context: reported, not judged)

**The registered roster, fixed here** (the Researcher's correction, REVIEW 7007213171): the 15
corpora of `CORPORA` at 8, 10 and 20 threads, 12 paired rounds each (1,080 records), and
`wikidata-x16` at 10 threads, 8 rounds (16 records). The scorer never builds the roster from the
corpora a file holds.

**One confirmatory attempt.** The first timed attempt is this registration's only one, kept
whatever its outcome. A VOID gives no adoption evidence and an inconclusive required gain is
NOT ADOPTED; neither opens a retry.

**VOID, and nothing is read,** where:
- a record of the roster is missing, duplicated or failed, a record lies outside it, or a file is
  empty or missing;
- a record's corpus, thread count or round is not exactly one of the registered values of its
  type (threads 8.5 is not 8; "8" is not 8), its arm is not a string, or its output digest is not
  one (the Researcher's second correction, REVIEW 7008014645): checked before any is used;
- a wall time, peak RSS or output size is not a finite positive number (a NaN included);
- the registered identities do not hold: the bundle's `commits.txt` names other builds than the
  port's pinned commit and v0.5.6 (530ed404); or its `inputs.sha256` does not give each input of
  `INPUTS` exactly once, with a 64-hex-digit SHA-256 and no other input, or gives a judged copy
  of `FROZEN` another digest than the frozen one: the pinned sample and held-out windows, the
  primary and sustained corpora (4 and 16 copies of the sample), and the held-out replication
  (the windows verbatim);
- a corpus's outputs differ;
- the machine is not the registered one: `lscpu` names no single i9-10900K, or `ab env` records
  other than 10 cores and 20 CPUs;
- either arm observes other than the shipped workers at a count on the Wikidata corpora (8 at 8
  threads, 7 at 10 and 20: 0.5.6's plan and the port's are the same plan; only the kernel
  differs). Read there alone: a decode of a few milliseconds can end before the thread sampler
  sees every worker, and an arm builds the same decoder whatever the corpus;
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
import glob
import json
import math
import os
import re
import statistics
import subprocess
import sys

CAND_DIR, REF_DIR = "/build-port/", "/build-v0.5.6/"
MAIN, HELDOUT, SUSTAINED = "wikidata-x4", "ho-wikidata-l9", "wikidata-x16"
# the registered roster (the Researcher's correction, REVIEW 7007213171): exactly these corpora
# at exactly these counts and rounds, never the corpora a file happens to hold
CORPORA = (
    "wikidata-x4",
    "text",
    "random",
    "records",
    "runs",
    "pattern",
    "onebyte",
    "ho-wikidata-l9",
    "ho-wikidata-l1",
    "ho-prose-l1",
    "ho-prose-l9",
    "ho-code-l1",
    "ho-code-l9",
    "ho-structured-l1",
    "ho-structured-l9",
)
COUNTS, ROUNDS, SUSTAINED_ROUNDS = (8, 10, 20), 12, 8
# the registered identities: the two builds' commits, and the two pinned Wikidata corpora
V056 = "530ed404a014a414ef0b62adfd947fa14d4533e6"
WIKIDATA_SHA256 = "82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915"
HELDOUT_SHA256 = "7f5c9a0044e5a30589bf343b9b31e698849b2acc69461a55c97b550193128b45"
# the judged copies whose bytes are fixed by those two, with no compressor between (the
# Researcher's second correction, REVIEW 7008014645): the primary and sustained corpora are 4 and
# 16 copies of the sample, and the held-out replication is the held-out windows verbatim
FROZEN = {
    "wikidata.bz2": WIKIDATA_SHA256,
    "wikidata-heldout.bz2": HELDOUT_SHA256,
    "wikidata-x4.bz2": "0b820d14299af637bf7f644d172f5efe91ad59e169cf2a50bd0962f5c89ce08e",
    "wikidata-x16.bz2": "1aeac0d469a85ae21de32e97cddcae6a6f1b29ed950526bc664491c372dbf422",
    "ho-wikidata-l9.bz2": HELDOUT_SHA256,
}
# every input the receipt must name, each with a well-formed SHA-256
INPUTS = (*(f"{c}.bz2" for c in (*CORPORA, "wikidata-x16")), "wikidata.bz2", "wikidata-heldout.bz2")
CHECKED = (MAIN, HELDOUT, "ho-wikidata-l1", SUSTAINED)
WORKERS = {8: 8, 10: 7, 20: 7}
ALPHA, GUARD_MAX, RSS_GATE = 0.05, 1.02, 1.10
T975 = {
    1: 12.706,
    2: 4.303,
    3: 3.182,
    4: 2.776,
    5: 2.571,
    6: 2.447,
    7: 2.365,
    8: 2.306,
    9: 2.262,
    10: 2.228,
    11: 2.201,
    12: 2.179,
    13: 2.160,
    14: 2.145,
    15: 2.131,
    16: 2.120,
    17: 2.110,
    18: 2.101,
    19: 2.093,
    20: 2.086,
    21: 2.080,
    22: 2.074,
    23: 2.069,
    24: 2.064,
    25: 2.060,
}


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
        errors.append(
            f"not the registered machine: lscpu names {sorted(models)}, not one i9-10900K"
        )
    try:
        e = json.load(open(env))
        if e.get("cores") != 10 or e.get("cpus") != 20:
            errors.append(
                f"not the registered profile: {e.get('cores')} cores, {e.get('cpus')} CPUs, not 10 and 20"
            )
    except (OSError, ValueError):
        errors.append(f"{env}: unreadable")
    return next(iter(models)) if len(models) == 1 else None


def positive(v):
    """A finite, positive number (never a bool, a NaN or an infinity)"""
    return isinstance(v, (int, float)) and not isinstance(v, bool) and math.isfinite(v) and v > 0


def field(r, name, kind, allowed):
    """`r[name]` if it is exactly of type `kind` (never a bool) and one of `allowed`; else a
    ValueError naming it"""
    v = r.get(name)
    if type(v) is not kind or v not in allowed:
        raise ValueError(f"{name} {v!r} is not one of the registered {sorted(allowed)}")
    return v


def load(path, corpora, counts, rounds, errors, check_workers=True):
    """{(corpus, threads, side, round): record}, `side` the port's build (`cand`) or v0.5.6's
    (`ref`), against the registered roster `corpora` x `counts` x both sides x `rounds`"""
    rows, outs, refs = {}, {}, set()
    if not os.path.exists(path):
        errors.append(f"{os.path.basename(path)}: missing")
        return rows
    for ln, line in enumerate(open(path), 1):
        if not line.strip():
            continue
        # every field is checked for its type and value before it is used, so a record of the
        # wrong shape is VOID, never coerced into the roster and never an exception (the
        # Researcher's second correction: threads 8.5 is not 8; an arm that is not a string)
        try:
            r = json.loads(line)
            if not isinstance(r, dict):
                raise ValueError("not an object")
            arm = r.get("arm")
            if type(arm) is not str:
                raise ValueError(f"arm {arm!r} is not a string")
            side = (
                "cand"
                if arm.startswith("mibz2@") and CAND_DIR in arm
                else "ref"
                if arm.startswith("mibz2@") and REF_DIR in arm
                else None
            )
            key = (
                field(r, "corpus", str, corpora),
                field(r, "threads", int, counts),
                side,
                field(r, "round", int, range(rounds)),
            )
        except ValueError as e:
            errors.append(f"{os.path.basename(path)} line {ln}: not a record of the roster ({e})")
            continue
        if side is None:
            errors.append(
                f"{os.path.basename(path)} line {ln}: arm {arm!r} is neither the port's build "
                f"(mibz2@…{CAND_DIR}…) nor v0.5.6's (mibz2@…{REF_DIR}…)"
            )
            continue
        refs.add((side, arm))
        if key in rows:
            errors.append(f"{key} duplicated")
            continue
        if r.get("termination") != "success" or r.get("err"):
            errors.append(f"{key}: failed ({r.get('termination')!r}, {r.get('err')!r})")
            continue
        if not positive(r.get("wall")) or not positive(r.get("rss_kb")):
            errors.append(
                f"{key}: wall {r.get('wall')!r} or peak RSS {r.get('rss_kb')!r} not a finite positive number"
            )
            continue
        digest = r.get("sha256") or r.get("hash")
        if not positive(r.get("out")) or type(digest) is not str:
            errors.append(f"{key}: no output size or digest")
            continue
        t = key[1]
        if (
            check_workers
            and key[0] in CHECKED
            and t in WORKERS
            and workers(r) != sorted(f"mibz2-{k}" for k in range(WORKERS[t]))
        ):
            errors.append(
                f"{key}: observed workers {workers(r)}, not the shipped {WORKERS[t]} at {t} threads"
            )
        outs.setdefault(key[0], set()).add((r.get("out"), r.get("sha256") or r.get("hash")))
        rows[key] = r
    for side in ("cand", "ref"):
        arms = sorted(a for s, a in refs if s == side)
        if len(arms) > 1:
            errors.append(
                f"more than one {'port' if side == 'cand' else 'reference'} binary: {arms}"
            )
    roster = {
        (c, t, a, i)
        for c in corpora
        for t in counts
        for a in ("cand", "ref")
        for i in range(rounds)
    }
    if not rows:
        errors.append(f"{os.path.basename(path)}: no records (the roster is {len(roster)})")
    missing = sorted(roster - set(rows))
    extra = sorted(set(rows) - roster)
    if missing:
        errors.append(f"{len(missing)} of {len(roster)} records missing, e.g. {missing[:3]}")
    if extra:
        errors.append(f"{len(extra)} records outside the roster, e.g. {extra[:3]}")
    for c, o in sorted(outs.items()):
        if len(o) != 1:
            errors.append(f"{c}: the outputs differ: {sorted(o, key=str)[:3]}")
    return rows


def identities(bundle, port, errors):
    """The registered builds and inputs, from the bundle's own records: `commits.txt` names the
    port's pinned commit and v0.5.6's, and `inputs.sha256` the two pinned Wikidata corpora and a
    digest for each corpus of the roster"""
    try:
        commits = dict(
            line.split() for line in open(os.path.join(bundle, "commits.txt")) if line.strip()
        )
    except (OSError, ValueError):
        commits = {}
    if commits.get("port") != port or commits.get("v0.5.6") != V056:
        errors.append(f"commits.txt: the builds are {commits}, not port {port} and v0.5.6 {V056}")
    digests = {}
    try:
        for ln, line in enumerate(open(os.path.join(bundle, "inputs.sha256")), 1):
            if not line.strip():
                continue
            parts = line.split()
            name = os.path.basename(parts[-1])
            if len(parts) != 2:
                errors.append(f"inputs.sha256 line {ln}: not one digest and one path")
            elif name in digests:
                errors.append(f"inputs.sha256: {name} named twice")
            else:
                digests[name] = parts[0]
    except OSError:
        errors.append("inputs.sha256: missing")
    bad = [n for n in INPUTS if n in digests and not re.fullmatch(r"[0-9a-f]{64}", digests[n])]
    if bad:
        errors.append(f"inputs.sha256: not a SHA-256 for {bad}")
    missing = [n for n in INPUTS if n not in digests]
    if missing:
        errors.append(f"inputs.sha256: no digest for {missing}")
    extra = sorted(set(digests) - set(INPUTS))
    if extra:
        errors.append(f"inputs.sha256: inputs outside the registration: {extra}")
    for name, want in FROZEN.items():
        if name in digests and digests[name] != want:
            errors.append(f"inputs.sha256: {name} is {digests[name]!r}, not the frozen {want}")


def cell(rows, corpus, t, rounds):
    xs = [
        math.log(rows[(corpus, t, "cand", i)]["wall"] / rows[(corpus, t, "ref", i)]["wall"])
        for i in range(rounds)
    ]
    m, se = statistics.mean(xs), statistics.stdev(xs) / math.sqrt(len(xs))
    h = t975(len(xs) - 1) * se
    mem = statistics.median(
        rows[(corpus, t, "cand", i)]["rss_kb"] for i in range(rounds)
    ) / statistics.median(rows[(corpus, t, "ref", i)]["rss_kb"] for i in range(rounds))
    return math.exp(m), math.exp(m - h), math.exp(m + h), p_gain(xs), mem


def label(lo, hi):
    return "FASTER" if hi < 1 else "SLOWER" if lo > 1 else "INCONCLUSIVE"


def judge(a):
    counts, corpora = list(COUNTS), list(CORPORA)
    errors = []
    if os.path.exists(os.path.join(a.bundle, "DRY-RUN.txt")):
        errors.append("a dry run (DRY-RUN.txt in the bundle): not evidence")
    model = machine(os.path.join(a.bundle, "lscpu.txt"), os.path.join(a.bundle, "env.json"), errors)
    identities(a.bundle, a.port, errors)
    rows = load(os.path.join(a.bundle, "runs.jsonl"), corpora, counts, ROUNDS, errors)
    srows = load(
        os.path.join(a.bundle, "sustained.jsonl"), [SUSTAINED], [10], SUSTAINED_ROUNDS, errors
    )
    print(
        f"records {len(rows)} of {len(corpora) * len(counts) * 2 * ROUNDS} and {len(srows)} of "
        f"{2 * SUSTAINED_ROUNDS} sustained; the registered roster: {len(corpora)} corpora at {counts} threads, "
        f"{ROUNDS} rounds ({SUSTAINED_ROUNDS} sustained)"
        + (f"; the machine: {model}" if model else "")
    )
    if errors:
        print(
            f"VOID: the records do not make the registered run ({len(errors)} problems); no verdict. "
            "This registration's one confirmatory attempt gives no adoption evidence, and opens no retry."
        )
        for e in errors[:25]:
            print(f"  {e}")
        return 1
    fails, ps, res = [], {}, {}
    print(
        "\nthe port over v0.5.6: wall, the rounds' paired ratios, geometric mean [95% interval]; "
        "one-sided p (gain); peak RSS, median over median"
    )
    for t in counts:
        print(f"  {t} threads")
        for c in corpora:
            g, lo, hi, p, mem = res[(c, t)] = cell(rows, c, t, ROUNDS)
            role = "primary" if c == MAIN else "held-out replication" if c == HELDOUT else "guard"
            print(
                f"    {c} ({role}): {g:.4f} [{lo:.4f}, {hi:.4f}] {label(lo, hi)}; p {p:.2e}; memory {mem:.3f}"
            )
            if c == MAIN:
                ps[t] = p
    rejected = holm(ps, ALPHA)
    print(
        f"\nP1, wall on {MAIN}: Holm at α {ALPHA} over {len(ps)} cells: "
        + ", ".join(
            f"{t} threads p {ps[t]:.2e} {'rejects' if t in rejected else 'does not'}"
            for t in counts
        )
    )
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
    g, lo, hi, p, mem = cell(srows, SUSTAINED, 10, SUSTAINED_ROUNDS)
    print(
        f"\nS, sustained: 10 threads back to back on {SUSTAINED}: {g:.4f} [{lo:.4f}, {hi:.4f}] {label(lo, hi)}; "
        f"memory {mem:.3f}"
    )
    if lo > 1:
        fails.append(f"S: sustained SLOWER, {g:.4f} [{lo:.4f}, {hi:.4f}]")
    elif g > GUARD_MAX:
        fails.append(f"S: sustained {g:.4f}, above the bound {GUARD_MAX}")
    if fails:
        print(f"\nVERDICT: NOT ADOPTED. {len(fails)} conditions fail:")
        for f in fails:
            print(f"  {f}")
    else:
        print(
            "\nVERDICT: ADOPTS. A wall gain on Wikidata at every count (Holm), peak RSS not above "
            "v0.5.6's, the gain replicated on held-out Wikidata, every guard within its bound, and "
            "the sustained cell within its bound."
        )
    return 0


def pairs(a):
    errors = []
    recs = {}
    for ln, line in enumerate(open(a.runs), 1):
        if line.strip():
            r = json.loads(line)
            wall = r.get("wall")
            if (
                r.get("termination") != "success"
                or isinstance(wall, bool)
                or not isinstance(wall, (int, float))
                or not wall > 0
            ):
                errors.append(f"line {ln}: {r.get('corpus')} {r.get('arm')} failed")
                continue
            # (the arm's name before `@`: the context arms are one build's)
            recs[(r["corpus"], int(r["threads"]), r["arm"].split("@")[0], int(r["round"]))] = r
    if errors:
        print(f"VOID: {len(errors)} records failed, e.g. {errors[:3]}")
        return 1
    for spec in a.pair:
        x, y = spec.split("/")
        print(f"{x} over {y}: wall, geometric mean [95% interval]; peak RSS, median over median")
        for c, t in sorted({(k[0], k[1]) for k in recs if k[2] == x}, key=lambda k: (k[1], k[0])):
            n = sorted(
                i
                for (cc, tt, arm, i) in recs
                if (cc, tt, arm) == (c, t, x) and (c, t, y, i) in recs
            )
            if len(n) < 2:
                continue
            xs = [math.log(recs[(c, t, x, i)]["wall"] / recs[(c, t, y, i)]["wall"]) for i in n]
            m, h = (
                statistics.mean(xs),
                t975(len(xs) - 1) * statistics.stdev(xs) / math.sqrt(len(xs)),
            )
            mem = statistics.median(recs[(c, t, x, i)]["rss_kb"] for i in n) / statistics.median(
                recs[(c, t, y, i)]["rss_kb"] for i in n
            )
            print(
                f"  W {t:2d} {c}: {math.exp(m):.4f} [{math.exp(m - h):.4f}, {math.exp(m + h):.4f}] "
                f"{label(math.exp(m - h), math.exp(m + h))}; memory {mem:.3f}"
            )
    return 0


def guard(a):
    """The one confirmatory attempt (the Researcher's corrections, REVIEWs 7007213171 and
    7008014645): exit 0 only if no timed attempt has started in `target` (a run directory with
    `started.txt` and no `DRY-RUN.txt`, other than `--this`) and the evidence branch, fetched
    afresh, is readable and archives no `open-port/run-*.tar.gz`. Fails closed: an evidence branch
    that can't be fetched or read refuses, as an archived attempt does."""
    for started in sorted(glob.glob(os.path.join(a.target, "run-*", "started.txt"))):
        d = os.path.dirname(started)
        if a.this and os.path.realpath(d) == os.path.realpath(a.this):
            continue
        if not os.path.exists(os.path.join(d, "DRY-RUN.txt")):
            print(
                f"bench-open-port: a timed attempt already started here ({os.path.basename(d)}): "
                "this registration allows one"
            )
            return 1
    ref = f"refs/remotes/{a.remote}/evidence"

    def git(*args):
        return subprocess.run(["git", *args], capture_output=True, text=True)

    if git("fetch", "-q", a.remote, f"+refs/heads/evidence:{ref}").returncode != 0:
        print(
            f"bench-open-port: the evidence branch can't be fetched from {a.remote}, so a timed "
            "attempt archived there can't be ruled out: refusing (this registration allows one)"
        )
        return 1
    listing = git("ls-tree", "-r", "--name-only", ref, "--", "open-port")
    if listing.returncode != 0:
        print(
            f"bench-open-port: the evidence branch ({ref}) can't be read: refusing "
            "(this registration allows one)"
        )
        return 1
    runs = [n for n in listing.stdout.split() if re.fullmatch(r"open-port/run-[^/]*\.tar\.gz", n)]
    if runs:
        print(
            f"bench-open-port: a timed attempt is already archived on the evidence branch "
            f"({runs[0]}): this registration allows one"
        )
        return 1
    return 0


def main(argv):
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="mode", required=True)
    j = sub.add_parser("judge")
    j.add_argument(
        "bundle",
        help="the run's directory: runs.jsonl, sustained.jsonl, lscpu.txt, env.json, "
        "commits.txt, inputs.sha256",
    )
    j.add_argument("--port", required=True, help="the port's registered commit (the recipe's PORT)")
    g = sub.add_parser("guard")
    g.add_argument("target", help="the recipe's target directory (target/open-port)")
    g.add_argument("--this", help="this attempt's own run directory, already started")
    g.add_argument("--remote", default="origin")
    q = sub.add_parser("pairs")
    q.add_argument("runs")
    q.add_argument("--pair", action="append", required=True)
    a = ap.parse_args(argv[1:])
    return {"judge": judge, "guard": guard, "pairs": pairs}[a.mode](a)


if __name__ == "__main__":
    sys.exit(main(sys.argv))
