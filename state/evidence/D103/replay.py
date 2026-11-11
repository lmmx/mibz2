#!/usr/bin/env python3
"""Untimed independent validation of archived D103 ordinary/team traces."""
import collections, hashlib, json, sys, zipfile
raw = open(sys.argv[1], "rb").read()
digest = hashlib.sha256(raw).hexdigest()
assert digest == "f2e7c1d0d421a858e8efde116aa9bf3d2e06650552746a036022589d251af2e7"
z = zipfile.ZipFile(sys.argv[1])
assert sorted(z.namelist()) == ["occupancy-teams.json", "occupancy-teams.txt", "occupancy.json", "occupancy.txt"]
result = {"artifact_sha256": digest, "files": {}}
def byte_bound(q):
    return max(8, 2*q-1) if q else 0
for filename in ("occupancy.json", "occupancy-teams.json"):
    d = json.loads(z.read(filename))
    assert d["worker_ledger"].startswith("route-history-v2")
    assert d["parity"] is True
    traced = histories = samples = old_tt_failures = 0
    plans = set()
    routes = collections.Counter()
    outputs = set()
    for run in d["runs"]:
        kv = dict(w.split("=", 1) for w in run["line"].split() if "=" in w)
        plans.add(kv["plan"])
        assert kv["err"] == "-"
        outputs.add((kv["out"], kv["hash"]))
        if run["mode"] == "off":
            continue
        traced += 1
        assert kv["t.complete"] == "yes" and kv["t.problem"] == "-"
        assert float(kv["t.f.w.ledger_violations"]) == 0
        keys = [k for k in kv if k.endswith(".history")]
        assert len(keys) == int(float(kv["t.f.w.recorded"]))
        run_samples = 0
        for key in keys:
            assert kv[key.replace(".history", ".errors")] == "-"
            histories += 1
            maxrows = llq = rleq = ttq = 0
            prev = dict(ll=0, rle=0, tt=0)
            for value in kv[key].split("|"):
                route, coop, avx2, ns, bs, buffers = value.split(":", 5)
                n, bound = int(ns), int(bs)
                assert 0 < n <= bound <= 900000 and bound % 100000 == 0
                bufs = {p.split(":")[0]: tuple(map(int, p.split(":")[1:])) for p in buffers.split(",")}
                maxrows = max(maxrows, n)
                llq = max(llq, n if route == "Runs" else bound)
                if route == "ColumnSeparate":
                    rleq = max(rleq, n)
                ttq = max(ttq, n + (8 if route == "Runs" and avx2 == "true" else 0))
                bounds = {"ll": (llq, byte_bound(llq)), "rle": (rleq, byte_bound(rleq)),
                          "tt": (4*maxrows, 4*max(4, 2*ttq-1))}
                for name, (lo, hi) in bounds.items():
                    length, cap = bufs[name]
                    assert length <= cap and prev[name] <= cap and lo <= cap <= hi, (filename, run["round"], key, name, cap, lo, hi)
                    prev[name] = cap
                assert bufs["counts"] == (1024, 1024)
                old_tt_failures += bufs["tt"][1] > 4*max(4, 2*maxrows-1)
                routes[(route, coop, avx2)] += 1
                samples += 1
                run_samples += 1
        assert run_samples == int(float(kv["t.blocks"])) == 431
    assert len(outputs) == 1
    overhead = []
    for r in d["results"]:
        for mode in ("timeline", "full"):
            assert r[mode]["complete_rounds"] == d["settings"]["rounds"]
            v = r["perturbation"][mode]["throughput_ratio"]
            assert v["lo"] <= 1 <= v["hi"]
            overhead.append(dict(threads=r["threads"], mode=mode, **v))
    if filename == "occupancy.json":
        assert traced == 36 and plans == {"workers1", "workers2", "workers4"}
    else:
        assert traced == 4 and plans == {"teams2x2"}
    result["files"][filename] = dict(traced_rounds=traced, worker_histories=histories,
        samples=samples, old_tt_upper_failures_on_new_samples=old_tt_failures,
        corrected_ll_rle_tt_errors=0, plans=sorted(plans), output=list(outputs)[0],
        routes={str(k):v for k,v in routes.items()}, overhead=overhead)
# Independent request-growth witness; scalar versus selected AVX2 fallback.
for slack in (0, 8):
    cap = qmax = 0
    for n in (99992, 100000):
        q = n + slack
        if q > cap:
            cap = max(q, 2*cap, 4)
        qmax = max(qmax, q)
        assert n <= cap <= max(4, 2*qmax-1)
    if slack:
        assert 4*cap == 800000 > 4*(2*100000-1)
        assert 4*cap <= 4*(2*(100000+8)-1) == 800060
result["boundary"] = "AVX2 cap800000 > old799996, <= corrected800060; scalar remains bounded"
print(json.dumps(result, indent=2))
