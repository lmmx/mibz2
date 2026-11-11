#!/usr/bin/env python3
"""Untimed independent replay of PR368 Occupancy artifact; no decoder execution.
Usage: python replay.py occupancy.zip
SHA-256 and the original bounds are asserted before the proposed finite slack envelope.
The latter is an analytical diagnostic, NOT a patch or resumption verdict.
"""
import collections, hashlib, json, re, sys, zipfile
path = sys.argv[1]
raw = open(path, "rb").read()
digest = hashlib.sha256(raw).hexdigest()
assert digest == "62d397b01c7f6f8c60db7c61f4694097a0b36a83ce765e1d41edf2733f2d5417"
z = zipfile.ZipFile(path)
assert sorted(z.namelist()) == ["occupancy.json", "occupancy.txt"]
d = json.loads(z.read("occupancy.json"))
assert d["parity"] is True
counts = collections.Counter()
bad_rounds = collections.Counter()
replayed = corrected = histories = samples = 0
first = None
for run in d["runs"]:
    if run["mode"] == "off":
        continue
    kv = dict(w.split("=", 1) for w in run["line"].split() if "=" in w)
    assert kv["t.complete"] == "yes"
    errors = []
    for key, value in kv.items():
        if not key.endswith(".errors") or value == "-":
            continue
        for error in value.split(";"):
            m = re.match(r"sample_(\d+):_([a-z]+)_", error)
            assert m, error
            counts[m[2]] += 1
            errors.append(error)
            if first is None:
                first = dict(round=run["round"], threads=run["threads"],
                             mode=run["mode"], key=key, error=error)
    assert len(errors) == int(float(kv["t.f.w.ledger_violations"]))
    if errors:
        bad_rounds[(run["threads"], run["mode"])] += 1
    for key, value in kv.items():
        if not key.endswith(".history"):
            continue
        histories += 1
        rows_max = request_max = previous = 0
        parsed = []
        for sample in value.split("|"):
            route, coop, rows, bound, buffers = sample.split(":", 4)
            rows, bound = int(rows), int(bound)
            bufs = {p.split(":")[0]: tuple(map(int, p.split(":")[1:])) for p in buffers.split(",")}
            parsed.append((route, rows, bufs["tt"]))
            samples += 1
        # Source validates all samples, then final buffers. Final ll/rle/tt are
        # checked for equality with the last snapshot by the original instrument.
        for route, rows, (length, capacity) in parsed + parsed[-1:]:
            rows_max = max(rows_max, rows)
            request_max = max(request_max, rows + (8 if route == "Runs" else 0))
            old_upper = 4 * max(4, 2 * rows_max - 1)
            slack_upper = 4 * max(4, 2 * request_max - 1)
            common = length > capacity or capacity < previous or capacity < 4 * rows_max
            replayed += common or capacity > old_upper
            corrected += common or capacity > slack_upper
            previous = capacity
assert counts == {"tt": 3973}, counts
assert replayed == 3973 and corrected == 0, (replayed, corrected)
# Native AVX2 reserve sequence: 899992+8 fills capacity 900000; 900000+8 grows it.
cap = 0
witness = []
for n in (899992, 900000):
    q = n + 8
    if q > cap:
        cap = max(q, 2 * cap, 4)
    witness.append(dict(rows=n, request=q, capacity_words=cap, capacity_bytes=4*cap))
assert witness[-1]["capacity_bytes"] == 7200000 > 4*(2*900000-1)
print(json.dumps(dict(artifact_sha256=digest, traced_rounds=36, histories=histories,
    samples=samples, original_errors=dict(counts), replayed_tt_errors=replayed,
    slack_envelope_errors=corrected, bad_rounds={str(k):v for k,v in bad_rounds.items()},
    first=first, witness=witness,
    interpretation="Original tt upper bound omits AVX2 n+8 reserve. Finite conservative request envelope explains raw errors; no native repair or resumption certified."), indent=2))
