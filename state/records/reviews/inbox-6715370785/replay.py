#!/usr/bin/env python3
"""Replay archived release rows, not a decoder or a timing harness.
Usage: python replay.py runs.jsonl
Input pinned by evidence.json. CI is Student t on paired log ratios, 10 rounds.
"""
import json, math, statistics, sys
rows=[json.loads(line) for line in open(sys.argv[1]) if line.strip()]
assert len(rows)==240
assert len({r["out"] for r in rows})==len({r["hash"] for r in rows})==1
assert not any(r["err"] for r in rows)
result={"rows":len(rows),"outputs":sorted({r["out"] for r in rows}),"hashes":sorted({r["hash"] for r in rows}),"threads":{}}
for threads in (1,2,4,8,10,12,16,20):
    cells=[r for r in rows if r["threads"]==threads]
    arms=sorted({r["arm"] for r in cells})
    assert len(arms)==3
    for arm in arms:
        assert sorted(r["round"] for r in cells if r["arm"]==arm)==list(range(10))
    cur={r["round"]:r for r in cells if r["arm"]=="mibz2"}
    old={r["round"]:r for r in cells if r["arm"].startswith("mibz2@")}
    logs=[math.log(old[i]["wall"]/cur[i]["wall"]) for i in range(10)]
    mean=statistics.mean(logs)
    half=2.262157*statistics.stdev(logs)/math.sqrt(10)
    result["threads"][threads]={"speed":math.exp(mean),"ci":[math.exp(mean-half),math.exp(mean+half)],"cpu_multiplier":math.exp(statistics.mean(math.log(cur[i]["cpu"]/old[i]["cpu"]) for i in range(10))),"wall_median":statistics.median(r["wall"] for r in cur.values())}
print(json.dumps(result,indent=2))
