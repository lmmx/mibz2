"""On complete, valid calibration the repaired analyser's results equal c0f8d38's, except
for the added S9 entry; and the old analyser's verdict on report 178's deletion case"""

import importlib.util, json, sys, copy

sys.path.insert(0, sys.argv[1])


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    return m


old = load("old", sys.argv[2])
new = load("new", sys.argv[1] + "/c30_traversal.py")
import test_c30_traversal as T

variants = [
    dict(c_cost=x, noise=n, c_imbalance=i)
    for x in (0.6, 0.8, 0.865, 0.95, 1.05)
    for n in (0.0, 0.002, 0.03)
    for i in (1.0, 1.275, 1.3125)
]
variants += [
    dict(p_drift=0.01),
    dict(skew=0.2),
    dict(clock_ns=6000),
    dict(c_desched=1.1, noise=0.002),
    dict(rounds=2, dry=True),
]
same = 0
for v in variants:
    m, w, c = T.make(**v)
    a = old.analyse(copy.deepcopy(m), copy.deepcopy(w), True, copy.deepcopy(c), [])
    b = new.analyse(copy.deepcopy(m), copy.deepcopy(w), True, copy.deepcopy(c), [])
    assert b["schedule"].pop("S9_calibration") is True, v
    assert json.dumps(a, sort_keys=True) == json.dumps(b, sort_keys=True), v
    same += 1
print(f"{same} valid-calibration variants: identical results, decisions included")
decisions = {}
for v in variants:
    m, w, c = T.make(**v)
    decisions.setdefault(new.analyse(m, w, True, c, []).get("decision", "dry"), 0)
    decisions[new.analyse(m, w, True, c, []).get("decision", "dry")] += 1
print("decisions covered:", decisions)
m, w, c = T.make(c_cost=0.8, noise=0.002)
c = T.expensive_cpu(c)
for k in c:
    k["cpus"] = [e for e in k["cpus"] if e["cpu"] != 4]
a = old.analyse(m, w, True, c, [])
b = new.analyse(m, w, True, c, [])
print(
    "expensive CPU deleted: c0f8d38 ->",
    a["decision"],
    "G5",
    a["gates"].get("G5_clock_overhead"),
    "| repaired ->",
    b["decision"],
)
m, w, c = T.make(c_cost=0.8, noise=0.002)
c = T.expensive_cpu(c)
print(
    "complete with the expensive CPU: both ->",
    old.analyse(m, w, True, c, [])["decision"],
    "/",
    new.analyse(m, w, True, c, [])["decision"],
)
