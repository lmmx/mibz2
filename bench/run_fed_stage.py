"""The run-fed diagnostic's admission verifier, for C31's direct placement
(docs/journal/2026-11-09-run-fed-direct.md; #269's, docs/journal/2026-11-09-run-fed-stage.md,
was schema 1, at fd60fffa).

examples/run_fed_stage.rs writes a record of its run (RUN_FED_RECORD): its identity, its
corpora's, and every round's raw times, arm by arm. This judges that record on its own:
VALID only if every registered figure is there, whole and finite, at the registered head,
rounds, kernels and corpus; then the readings, recomputed here from the raw times, not
taken from the binary's printout.

    python3 bench/run_fed_stage.py verify RECORD --head SHA [--wikidata FILE]
        [--sources DIR] [--unjudged]

Exit 0 for VALID, 1 for INVALID. `--unjudged` labels the readings so: for a run that is
not the registered one (any run before the freeze).

The corpus evidence is the verifier's own, never the record's: the Wikidata corpus's
identity, sizes and blocks from its pinned spec (bench/wikidata.json), and from the file
itself where given (its SHA-256, size, and blocks counted from their headers); the six
synthetic corpora regenerated here from the head's sources (`--sources`, the checkout:
tests/common's `corpora(1)` ported, then compressed at level 9), their SHA-256, size,
decoded size and blocks. Every count the record makes must also fit the representation's
relations between them (`count_relations`). All of it is decided before any reading.
"""

import argparse
import bz2
import hashlib
import json
import math
import os
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent

ROUNDS = 15
WIKIDATA_SHA256 = "82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915"
SYNTHETIC = [
    "synthetic text",
    "synthetic random",
    "synthetic one byte",
    "synthetic pattern",
    "synthetic runs",
    "synthetic records",
]
SCHEMA = "run_fed_stage/2"
ARMS = ["A column", "B records", "D direct"]
PHASES = ["front", "sort", "build", "fallback"]
ENTRY_PATHS = ["records", "fallback"]
NARMS = 3
# bzip2's block limit: at most 900,000 rows (BWT symbols) a block, at level 9
MAX_ROWS = 900_000
# the sampled clock's plausible range, GHz (`sweep.yml` passes the reported MHz / 1000)
GHZ_RANGE = (0.1, 10.0)
# #269's phase-sum gap less its sort, nominal c/B (4.228 - 3.693 - 0.317): the budget's
# fixed term (docs/journal/2026-11-09-run-fed-direct.md, section 3)
BUDGET_269 = 0.218
HEX40 = re.compile(r"[0-9a-f]{40}")
HEX64 = re.compile(r"[0-9a-f]{64}")
T95 = [
    12.706,
    4.303,
    3.182,
    2.776,
    2.571,
    2.447,
    2.365,
    2.306,
    2.262,
    2.228,
    2.201,
    2.179,
    2.160,
    2.145,
    2.131,
    2.120,
    2.110,
    2.101,
    2.093,
    2.086,
    2.080,
    2.074,
    2.069,
    2.064,
    2.060,
    2.056,
    2.052,
    2.048,
    2.045,
    2.042,
]
# phases[arm][phase] that must be timed (positive) on a corpus with records' blocks (D's
# "build" is its placement); A's and D's sort is never timed (exactly 0)
REQUIRED_PHASES = [(0, 0), (0, 2), (1, 0), (1, 1), (1, 2), (2, 0), (2, 2)]
NEVER_TIMED = [(0, 1), (2, 1)]


def _reject_constant(name):
    raise ValueError(f"not a finite number: {name}")


def load(path):
    """The record, refusing NaN and infinities (which JSON has no word for)"""
    with open(path) as f:
        return json.load(f, parse_constant=_reject_constant)


def _is_int(x):
    return isinstance(x, int) and not isinstance(x, bool)


def _is_time(x):
    return isinstance(x, (int, float)) and not isinstance(x, bool) and math.isfinite(x) and x >= 0


def interval(logs):
    """The geometric mean and its 95% t-interval, from the paired rounds' log ratios"""
    n = len(logs)
    m = sum(logs) / n
    var = sum((x - m) ** 2 for x in logs) / (n - 1)
    h = T95[n - 2] * math.sqrt(var / n)
    return (math.exp(m), math.exp(m - h), math.exp(m + h))


def ratio(rounds, num, den):
    return interval([math.log(num(r) / den(r)) for r in rounds])


def check(rec, head, corpora, wikidata_file=None, primary=True):
    """The record's problems, each a reason it is INVALID (none: VALID).

    `corpora` is the verifier's own evidence a corpus, in the registered order: a dict of
    its sha256, bytes, output (decoded bytes) and blocks (`expected_corpora`).
    `wikidata_file`, where the file was given, is the same measured from it
    (`file_evidence`), and must agree with both the record and the pinned spec.
    """
    p = []
    if not isinstance(rec, dict):
        return ["the record is not an object"]
    if rec.get("schema") != SCHEMA:
        p.append(f"schema {rec.get('schema')!r}, not {SCHEMA}")
    if not (isinstance(head, str) and HEX40.fullmatch(head)):
        p.append(f"the registered head {head!r} is not a full 40-character commit")
    rh = rec.get("head")
    if not (isinstance(rh, str) and HEX40.fullmatch(rh)):
        p.append(f"the record's head {rh!r} is not a full 40-character commit")
    elif rh != head:
        p.append(f"the record's head {rh} is not the registered {head}")
    if not (_is_int(rec.get("rounds")) and rec["rounds"] == ROUNDS):
        p.append(f"rounds {rec.get('rounds')!r}, not {ROUNDS}")
    ghz = rec.get("ghz")
    if not (
        isinstance(ghz, (int, float))
        and not isinstance(ghz, bool)
        and math.isfinite(ghz)
        and GHZ_RANGE[0] <= ghz <= GHZ_RANGE[1]
    ):
        p.append(
            f"ghz {ghz!r}: the sampled clock must be a finite number in "
            f"[{GHZ_RANGE[0]}, {GHZ_RANGE[1]}] GHz"
        )
    k = rec.get("kernels")
    if not (isinstance(k, dict) and k.get("avx2") is True):
        p.append(f"kernels {k!r}: AVX2 not selected")
    if rec.get("correctness") != "passed":
        p.append(f"correctness {rec.get('correctness')!r}, not passed")
    host = rec.get("host")
    if not (isinstance(host, str) and host and host != "unknown"):
        p.append(f"host {host!r} not recorded")
    aff = rec.get("affinity")
    if not isinstance(aff, str) or (primary and not re.fullmatch(r"\d+", aff)):
        p.append(f"affinity {aff!r}: the primary condition is one copy pinned to one CPU")
    for key, want in (("arms", ARMS), ("phases", PHASES), ("entry_paths", ENTRY_PATHS)):
        if rec.get(key) != want:
            p.append(f"{key} {rec.get(key)!r}, not {want}")
    files = rec.get("files")
    if not isinstance(files, list) or len(files) != 1 + len(SYNTHETIC):
        n = len(files) if isinstance(files, list) else None
        p.append(f"{n} files, not the registered {1 + len(SYNTHETIC)}")
        return p
    if not (isinstance(corpora, list) and len(corpora) == len(files)):
        p.append(
            "no corpus evidence: the verifier's own evidence for the seven corpora is required"
        )
        return p
    if wikidata_file is not None:
        for key in ("sha256", "bytes", "blocks"):
            if wikidata_file.get(key) != corpora[0].get(key):
                p.append(
                    f"the Wikidata file's {key} {wikidata_file.get(key)!r} is not the pinned "
                    f"spec's {corpora[0].get(key)!r}"
                )
    rounds = rec.get("rounds") if _is_int(rec.get("rounds")) else ROUNDS
    for i, f in enumerate(files):
        p += check_file(i, f, rounds, corpora[i])
    return p


def count_relations(f):
    """The relations a corpus's counts must keep, as the representation fixes them (each
    a block's sum): a block has 1 to MAX_ROWS rows; a block by records has 1 to its rows'
    runs (records), its cells are at least its runs and at most 2r - 1, B's fill stores 8
    for each started 8 of a cell's rows (so a multiple of 8, from its rows to its rows plus
    7 a cell, and one 8-slot store at least a cell); D's stores write its rows exactly or
    more, no more than B's fill, with one store at least a cell and at most 7 one-slot rows
    a destination bucket, and 1 to 256 nonempty buckets a block, no more than its runs;
    the fallback's blocks hold 1 to MAX_ROWS rows each; RLE1 turns 5 rows
    into 4 to 259 bytes, and each block's last 4 rows at most may be unpaired. Returns the
    problems"""
    p = []
    b, br, bf = f["blocks"], f["blocks_runs"], f["blocks_fallback"]
    rows, rr, r, c, sl = f["rows"], f["rows_runs"], f["records"], f["cells"], f["slots"]
    dv, ds, dst = f["d_vector"], f["d_scalar"], f["d_streams"]
    rel = [
        ("blocks_runs + blocks_fallback = blocks", br + bf == b),
        ("blocks <= rows <= 900,000 a block", b <= rows <= MAX_ROWS * b),
        ("blocks_runs <= rows_runs <= 900,000 a block by records", br <= rr <= MAX_ROWS * br),
        (
            "the fallback's rows: blocks_fallback <= rows - rows_runs <= 900,000 a fallback block",
            bf <= rows - rr <= MAX_ROWS * bf,
        ),
        ("blocks_runs <= records <= rows_runs", br <= r <= rr),
        ("records <= cells <= 2 records - blocks_runs", r <= c <= 2 * r - br),
        ("rows_runs <= slots <= rows_runs + 7 cells", rr <= sl <= rr + 7 * c),
        ("8 cells <= slots (B stores 8 slots at least a cell)", 8 * c <= sl),
        ("slots a multiple of 8", sl % 8 == 0),
        ("blocks_runs <= d_streams <= 256 blocks_runs", br <= dst <= 256 * br),
        ("d_streams <= records (a nonempty bucket holds a run at least)", dst <= r),
        ("rows_runs <= 8 d_vector + d_scalar <= slots", rr <= 8 * dv + ds <= sl),
        ("cells <= d_vector + d_scalar (D stores once at least a cell)", c <= dv + ds),
        ("d_scalar <= 7 d_streams", ds <= 7 * dst),
        ("RLE1: 5 output >= 4 (rows - 4 blocks)", 5 * f["output"] >= 4 * (rows - 4 * b)),
        ("RLE1: 5 output <= 259 rows", 5 * f["output"] <= 259 * rows),
    ]
    for what, ok in rel:
        if not ok:
            p.append(f"the counts break: {what}")
    return p


def check_file(i, f, rounds, want):
    p = []
    if not isinstance(f, dict):
        return [f"file {i} is not an object"]
    name = f.get("name")
    where = f"file {i} ({name!r})"
    if i == 0:
        if f.get("sha256") != WIKIDATA_SHA256:
            p.append(f"{where}: SHA-256 {f.get('sha256')!r}, not the registered {WIKIDATA_SHA256}")
    else:
        if name != SYNTHETIC[i - 1]:
            p.append(f"{where}: not the registered {SYNTHETIC[i - 1]!r}")
        if not (isinstance(f.get("sha256"), str) and HEX64.fullmatch(f["sha256"])):
            p.append(f"{where}: SHA-256 {f.get('sha256')!r} not recorded")
    # the corpus's identity and sizes against the verifier's own evidence
    for key in ("sha256", "bytes", "output", "blocks"):
        if f.get(key) != want.get(key):
            p.append(f"{where}: {key} {f.get(key)!r}, not the evidence's {want.get(key)!r}")
    counts = [
        "bytes",
        "output",
        "blocks",
        "rows",
        "blocks_runs",
        "rows_runs",
        "records",
        "blocks_fallback",
        "cells",
        "slots",
        "d_vector",
        "d_scalar",
        "d_streams",
    ]
    if not all(_is_int(f.get(c)) and f[c] >= 0 for c in counts):
        p.append(f"{where}: path counts missing or not whole: {[f.get(c) for c in counts]}")
        return p
    if f["bytes"] == 0 or f["output"] == 0 or f["blocks"] == 0 or f["rows"] == 0:
        p.append(f"{where}: an empty corpus")
    if f["blocks_runs"] + f["blocks_fallback"] != f["blocks"]:
        p.append(f"{where}: {f['blocks_runs']} + {f['blocks_fallback']} blocks, not {f['blocks']}")
    p += [f"{where}: {x}" for x in count_relations(f)]
    order = f.get("order")
    want = [[(j + r) % NARMS for j in range(NARMS)] for r in range(rounds)]
    if order != want:
        p.append(f"{where}: the arms' order is not the registered rotation")
    has = {0: f["blocks_runs"] > 0, 1: f["blocks_fallback"] > 0}
    entry = f.get("entry")
    if not (isinstance(entry, list) and len(entry) == rounds):
        p.append(
            f"{where}: entry times for {len(entry) if isinstance(entry, list) else None} "
            f"rounds, not {rounds}"
        )
    else:
        for r, row in enumerate(entry):
            if not (
                isinstance(row, list)
                and len(row) == NARMS
                and all(isinstance(a, list) and len(a) == 2 and all(map(_is_time, a)) for a in row)
            ):
                p.append(f"{where}: round {r}'s entry times are not {NARMS} arms x 2 finite times")
                continue
            for a in range(NARMS):
                for path in (0, 1):
                    t = row[a][path]
                    if has[path] and t <= 0:
                        p.append(
                            f"{where}: round {r}, {ARMS[a]}, {ENTRY_PATHS[path]}: no "
                            f"time for a path with blocks"
                        )
                    if not has[path] and t != 0:
                        p.append(
                            f"{where}: round {r}, {ARMS[a]}, {ENTRY_PATHS[path]}: a "
                            f"time for a path with no blocks"
                        )
    phases = f.get("phases")
    if not (isinstance(phases, list) and len(phases) == rounds):
        p.append(
            f"{where}: phase times for "
            f"{len(phases) if isinstance(phases, list) else None} rounds, not {rounds}"
        )
    else:
        for r, row in enumerate(phases):
            if not (
                isinstance(row, list)
                and len(row) == NARMS
                and all(isinstance(a, list) and len(a) == 4 and all(map(_is_time, a)) for a in row)
            ):
                p.append(f"{where}: round {r}'s phase times are not {NARMS} arms x 4 finite times")
                continue
            for a, ph in REQUIRED_PHASES:
                if has[0] and row[a][ph] <= 0:
                    p.append(f"{where}: round {r}, {ARMS[a]} {PHASES[ph]}: not timed")
            for a, ph in NEVER_TIMED + ([] if has[0] else REQUIRED_PHASES):
                if row[a][ph] != 0:
                    p.append(
                        f"{where}: round {r}, {ARMS[a]} {PHASES[ph]}: a time where none is taken"
                    )
            for a in range(NARMS):
                if has[1] != (row[a][3] > 0):
                    p.append(
                        f"{where}: round {r}, {ARMS[a]} fallback: its time and its "
                        f"{f['blocks_fallback']} blocks disagree"
                    )
    return p


def _median(xs):
    return sorted(xs)[len(xs) // 2]


def readings(rec):
    """The registered readings, from a VALID record's raw times"""
    files = rec["files"]
    out = {"files": []}
    for f in files:
        e, ph = f["entry"], f["phases"]
        row = {"name": f["name"], "blocks_fallback": f["blocks_fallback"]}
        tot = lambda a: lambda r: r[a][0] + r[a][1]  # noqa: E731
        row["R_direct"] = ratio(e, tot(2), tot(0))
        row["R_stage"] = ratio(e, tot(1), tot(0))
        row["R_DB"] = ratio(e, tot(2), tot(1))
        if f["blocks_runs"]:
            row["R_acc_D"] = ratio(e, lambda r: r[2][0], lambda r: r[0][0])
            row["R_acc"] = ratio(e, lambda r: r[1][0], lambda r: r[0][0])
        else:
            row["R_acc_D"] = row["R_acc"] = None
        if f["blocks_fallback"]:
            row["R_fb_D"] = ratio(e, lambda r: r[2][1], lambda r: r[0][1])
            row["R_fb"] = ratio(e, lambda r: r[1][1], lambda r: r[0][1])
        else:
            row["R_fb_D"] = row["R_fb"] = None
        if f["blocks_runs"]:
            row["R_phases_D"] = ratio(ph, lambda r: sum(r[2]), lambda r: sum(r[0]))
            row["R_place"] = ratio(ph, lambda r: r[2][2], lambda r: r[0][2])
            row["R_pf"] = ratio(ph, lambda r: r[2][2], lambda r: r[1][2])
            row["R_psf"] = ratio(ph, lambda r: r[2][2], lambda r: r[1][1] + r[1][2])
            # the budget's terms, nominal c/B of output, each round's own: K_i, delta_i,
            # e_i, f_i; then (D - A)_i = K_i + delta_i - e_i - f_i reconstructed a round at
            # a time (D's sort is 0) and its largest residual kept, before any summary. The
            # medians below are each term's own: their sum is not the total's median
            cb = rec["ghz"] * 1e9 / f["output"]
            per = lambda g: [g(r) * cb for r in ph]  # noqa: E731
            k_i = per(lambda r: sum(r[1]) - sum(r[0]) - r[1][1])
            d_i = per(lambda r: r[2][2] - r[1][2])
            e_i = per(lambda r: r[1][0] - r[2][0])
            f_i = per(lambda r: r[1][3] - r[2][3])
            da_i = per(lambda r: sum(r[2]) - sum(r[0]))
            row["identity_residual"] = max(
                abs(da - (k + d - e - fb)) for da, k, d, e, fb in zip(da_i, k_i, d_i, e_i, f_i)
            )
            row["delta_direct"] = _median(d_i)
            row["e"] = _median(e_i)
            row["f"] = _median(f_i)
            row["D_minus_A"] = _median(da_i)
            row["B_minus_A_less_sort"] = _median(k_i)
            row["budget"] = _median([BUDGET_269 + d - e - fb for d, e, fb in zip(d_i, e_i, f_i)])
            # D's predicted table stores, from the record's census (reported, exploratory)
            rr = f["rows_runs"]
            row["d_stores_per_row"] = (f["d_vector"] + f["d_scalar"]) / rr
            row["d_slots_per_row"] = (8 * f["d_vector"] + f["d_scalar"]) / rr
            row["b_slots_per_row"] = f["slots"] / rr
            row["d_streams_per_block"] = f["d_streams"] / f["blocks_runs"]
        out["files"].append(row)
    w = out["files"][0]
    g, lo, hi = w["R_direct"]
    syn = max(out["files"][1:], key=lambda r: r["R_direct"][0])
    fbs = [r for r in out["files"] if r["R_fb_D"] is not None]
    fb = max(fbs, key=lambda r: r["R_fb_D"][0]) if fbs else None
    s_syn = syn["R_direct"][0] <= 1.02
    s_fb = fb is None or fb["R_fb_D"][0] <= 1.02
    out["screen_synthetic"] = {"worst": syn["name"], "geomean": syn["R_direct"][0], "pass": s_syn}
    out["screen_fallback"] = (
        {"worst": fb["name"], "geomean": fb["R_fb_D"][0], "pass": s_fb}
        if fb
        else {"worst": None, "geomean": None, "pass": True, "vacuous": True}
    )
    gate = g <= 0.95 and hi < 1.0
    if gate and s_syn and s_fb:
        q1 = "FEASIBLE"
    elif lo > 0.95:
        q1 = "STOP: no saving of 5% or more"
    elif gate:
        q1 = "STOP: a screen failed"
    else:
        q1 = "STOP, INCONCLUSIVE"
    out["Q1"] = q1
    dg, dlo, dhi = w["R_DB"]
    out["sort_removal"] = (
        "D CHEAPER THAN B"
        if dhi < 1.0
        else "D NOT CHEAPER THAN B"
        if dlo >= 1.0
        else "D AGAINST B, INCONCLUSIVE"
    )
    return out


NOTES = [
    "Q1's screens are point-estimate screens, not a noninferiority claim: a passing screen "
    "does not show that no corpus or fallback regresses by more than 2%.",
    "STOP and INCONCLUSIVE are operational pauses for this implementation on this runner, "
    "not proof of no effect; a STOP on this arm does not close the run-fed line (PROTOCOL 11).",
    "R_DB, the phase ratios and the budget are exploratory. The budget is a sensitivity on "
    "the phase sweep (nominal c/B of output: wall time x the sampled clock), not a "
    "reconstructed paired verdict: the phase sum is not the entry point's cost.",
]


BLOCK_MAGIC = bytes.fromhex("314159265359")


def count_blocks(data):
    """A bzip2 file's blocks, counted from their 48-bit headers at any bit offset (the
    blocks are bit-aligned; the streams' own headers are byte-aligned and not counted)"""
    big, n, seen = int.from_bytes(data, "big"), len(data), set()
    for s in range(8):
        b = (big << s).to_bytes(n + 1, "big")
        j = b.find(BLOCK_MAGIC)
        while j >= 0:
            seen.add(8 * j - 8 + s)
            j = b.find(BLOCK_MAGIC, j + 1)
    return len(seen)


def file_evidence(data):
    return {
        "sha256": hashlib.sha256(data).hexdigest(),
        "bytes": len(data),
        "blocks": count_blocks(data),
    }


def wikidata_expected(spec_path=HERE / "wikidata.json"):
    """The Wikidata corpus's evidence from its pinned spec; its SHA-256 must be the
    registered one"""
    with open(spec_path) as f:
        spec = json.load(f)
    if spec.get("corpus_sha256") != WIKIDATA_SHA256:
        raise ValueError(f"{spec_path}: corpus_sha256 is not the registered {WIKIDATA_SHA256}")
    return {
        "name": "wikidata",
        "sha256": spec["corpus_sha256"],
        "bytes": spec["corpus_bytes"],
        "output": spec["decoded_bytes"],
        "blocks": sum(w["blocks"] for w in spec["windows"]),
    }


class _Rng:
    """tests/common's xorshift Rng"""

    M = (1 << 64) - 1

    def __init__(self, x):
        self.x = x

    def next(self):
        x = self.x
        x ^= (x << 13) & self.M
        x ^= x >> 7
        x ^= (x << 17) & self.M
        self.x = x
        return x

    def below(self, n):
        return self.next() % max(n, 1)

    def bytes(self, n):
        return bytes(self.next() & 0xFF for _ in range(n))


def synthetic_inputs(root):
    """tests/common/mod.rs's `corpora(1)`, ported: the six synthetic inputs, by name, the
    text corpus read from `root`'s sources as the binary reads them"""
    text = bytearray()
    for d in ("src", "tests", "examples"):
        for f in sorted(os.listdir(os.path.join(root, d))):
            path = os.path.join(root, d, f)
            if os.path.isfile(path):
                with open(path, "rb") as h:
                    text += h.read()
    with open(os.path.join(root, "README.md"), "rb") as h:
        text += h.read()
    one, i = bytes(text), 0
    while len(text) < 3_000_000:
        text += f"\n-- copy {i} --\n".encode() + one
        i += 1
    rng = _Rng(0xFEED_BEEF_DEAD_CAFE)
    records = b"".join(
        ((((i * 2_654_435_761) & 0xFFFFFFFF) >> 12) | ((i << 20) & 0xFFFFFFFF)).to_bytes(
            4, "little"
        )
        for i in range(600_000)
    )
    runs = bytearray()
    while len(runs) < 2_500_000:
        n = 1 + rng.below(600)
        runs += bytes([b"xyz"[rng.below(3)]]) * n
    return [
        ("synthetic text", bytes(text)),
        ("synthetic random", rng.bytes(2_000_000)),
        ("synthetic one byte", b"a" * 5_000_000),
        ("synthetic pattern", b"0123456" * 700_000),
        ("synthetic runs", bytes(runs)),
        ("synthetic records", records),
    ]


def synthetic_expected(root):
    """Each synthetic corpus's evidence: its input regenerated, compressed at level 9 as
    tests/common's `compress` (libbzip2), then its SHA-256, size, decoded size and blocks"""
    out = []
    for name, data in synthetic_inputs(root):
        c = bz2.compress(data, 9)
        out.append({"name": name, "output": len(data), **file_evidence(c)})
    return out


def expected_corpora(root):
    return [wikidata_expected()] + synthetic_expected(root)


def fmt(x):
    return "n/a" if x is None else f"{x[0]:.4f} ({x[1]:.4f}, {x[2]:.4f})"


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    v = sub.add_parser("verify")
    v.add_argument("record")
    v.add_argument("--head", required=True, help="the registered (frozen) 40-character head")
    v.add_argument("--wikidata", help="the Wikidata file, to hash, size and count")
    v.add_argument(
        "--sources",
        default=str(HERE.parent),
        help="the head's checkout, whose sources the synthetic text is made from (default: "
        "this verifier's own)",
    )
    v.add_argument("--unjudged", action="store_true", help="not the registered run")
    v.add_argument(
        "--contended",
        action="store_true",
        help="a contended copy: its affinity need not be one CPU (never judged)",
    )
    a = ap.parse_args(argv)
    try:
        rec = load(a.record)
    except (OSError, ValueError) as e:
        print(f"INVALID: the record does not load: {e}")
        return 1
    wfile = None
    if a.wikidata:
        with open(a.wikidata, "rb") as f:
            wfile = file_evidence(f.read())
    try:
        corpora = expected_corpora(a.sources)
    except (OSError, ValueError, KeyError) as e:
        print(f"INVALID: no corpus evidence: {e}")
        return 1
    problems = check(rec, a.head, corpora, wfile, primary=not a.contended)
    if problems:
        print(f"INVALID ({len(problems)} problems):")
        for x in problems:
            print(f"  - {x}")
        return 1
    label = "UNJUDGED" if a.unjudged or a.contended else "JUDGED"
    r = readings(rec)
    print(f"VALID; readings {label}")
    for f in r["files"]:
        print(
            f"  {f['name']}: R_direct {fmt(f['R_direct'])}, R_stage {fmt(f['R_stage'])}, "
            f"R_DB {fmt(f['R_DB'])}, R_acc_D {fmt(f['R_acc_D'])}, R_fb_D {fmt(f['R_fb_D'])}"
        )
    w = r["files"][0]
    for key in ("R_phases_D", "R_place", "R_pf", "R_psf"):
        print(f"  {key} {fmt(w.get(key))}")
    if "d_stores_per_row" in w:
        print(
            f"  D's predicted table stores a row (its source's store loops, not machine "
            f"code): {w['d_stores_per_row']:.4f}, {w['d_slots_per_row']:.4f} slots written "
            f"(B's fill {w['b_slots_per_row']:.4f}); {w['d_streams_per_block']:.1f} nonempty "
            f"destination buckets a block (not measured hardware streams)"
        )
    if "budget" in w:
        print(
            f"  budget (exploratory, nominal c/B, medians of each round's own terms): "
            f"delta_direct {w['delta_direct']:+.3f}, e {w['e']:+.3f}, f {w['f']:+.3f} "
            f"(measured controls); {BUDGET_269} + delta_i - e_i - f_i {w['budget']:+.3f}, "
            f"against D - A's phase sum {w['D_minus_A']:+.3f} (the identity's largest "
            f"residual a round {w['identity_residual']:.2e}); this run's own K "
            f"{w['B_minus_A_less_sort']:+.3f}"
        )
    print(f"  screen, synthetic whole stage: {r['screen_synthetic']}")
    print(f"  screen, fallback (entry point, selector charged): {r['screen_fallback']}")
    print(f"  Q1 ({label}): {r['Q1']}")
    print(f"  D against B (exploratory): {r['sort_removal']}")
    for n in NOTES:
        print(f"  note: {n}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
