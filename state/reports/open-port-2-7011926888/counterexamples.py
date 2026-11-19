"""Deterministic checks for the G2 estimand and split-bundle admission."""
from contextlib import redirect_stdout
from io import StringIO
import itertools
import json
import math
import shutil
import statistics
from pathlib import Path
from types import SimpleNamespace

import open_port as op
import open_port2 as new
import test_open_port2 as fixtures

xs = [-0.001] * 57 + [0.5] * 3
print(f"skew witness: exp(mean)={math.exp(statistics.mean(xs)):.8f}, signed-rank p={new.signed_rank_p(xs):.12g}")
assert math.exp(statistics.mean(xs)) > 1.02 and new.signed_rank_p(xs) > new.ALPHA2
print(f"shared-sign null: p(60 positives)={new.signed_rank_p([.02] * 60):.12g}, rejection probability 1/2")
assert new.signed_rank_p([.02] * 60) < new.ALPHA2 and new.signed_rank_p([-.02] * 60) == 1
for short in ([0, 0], [0, 1, -1, 4], [-3, 1, 3, 3, -1], [1, -2, 1, -1, -1, 2]):
    assert new.signed_rank_p(short) == fixtures.brute(short)

def run(label, edit):
    root = fixtures.build2()
    try:
        edit(root)
        with redirect_stdout(buf := StringIO()):
            rc = new.judge(SimpleNamespace(bundle=str(root), port=fixtures.PORT_SHA))
        verdict = next(line for line in buf.getvalue().splitlines() if line.startswith("VERDICT:"))
        print(label, rc, verdict.split(".")[0])
        assert verdict.startswith("VERDICT: ADOPTS")
    finally:
        shutil.rmtree(root)

def rewrite(root, name, change):
    p = root / name
    rows = [json.loads(line) for line in p.read_text().splitlines()]
    for r in rows:
        change(r)
    p.write_text("".join(json.dumps(r) + "\n" for r in rows))

def change_digest(r):
    if r["corpus"] == "text":
        r["sha256"] = "f" * 64

def change_candidate(r):
    if "/build-port/" in r["arm"]:
        r["arm"] = r["arm"].replace("/build-port/", "/other/build-port/")

def set_skew(first, r):
    if (r["corpus"], r["threads"]) == ("text", 8):
        x = xs[r["round"] if first else r["round"] + op.ROUNDS]
        r["wall"] = .05 * (math.exp(x) if "/build-port/" in r["arm"] else 1)

def skew_guard(root):
    rewrite(root, "runs.jsonl", lambda r: set_skew(True, r))
    rewrite(root, "guards-b.jsonl", lambda r: set_skew(False, r))

run("skewed guard, 2.435% geometric-mean slowdown", skew_guard)
run("extension reassigns text output digest", lambda root: rewrite(root, "guards-b.jsonl", change_digest))
run("extension reassigns candidate binary", lambda root: rewrite(root, "guards-a.jsonl", change_candidate))
