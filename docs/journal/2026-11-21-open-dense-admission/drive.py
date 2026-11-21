"""In-process pairs of each harness binary against master's kernel, over rounds.

Usage: drive.py MODE SET ROUNDS BIN... (after #692's drive2.py). The binaries' order
alternates by round. MODE t1 (SET: six, rejected or dense) or t4 (SET ignored; ADMIT_WORKERS
workers over the whole file).
"""

import json, os, subprocess, sys, math, statistics as st

mode, sset, rounds, bins = sys.argv[1], sys.argv[2], int(sys.argv[3]), sys.argv[4:]
S = os.path.dirname(os.path.abspath(__file__))
env = dict(
    os.environ,
    ADMIT_BZ2=os.environ.get("ADMIT_BZ2", S + "/../moonshot/corpora/wikidata.bz2"),
    MIBZ2_ADMITTED_BWT=os.environ.get("MIBZ2_ADMITTED_BWT", S + "/../wt-data/admitted-bwt"),
    ADMIT_MODE=mode,
    ADMIT_SET=sset,
)
cpu = os.environ.get("CPU", "2" if mode == "t1" else "0-3")
res = {b: [] for b in bins}
for r in range(rounds):
    for b in bins if r % 2 == 0 else bins[::-1]:
        o = subprocess.run(
            [
                "taskset",
                "-c",
                cpu,
                S + "/" + b,
                "open::admit_harness::admit_harness",
                "--ignored",
                "--exact",
                "--nocapture",
                "--test-threads=1",
            ],
            env=env,
            capture_output=True,
            text=True,
        ).stdout
        d = {}
        for line in o.replace("test open::admit_harness::admit_harness ... ", "").splitlines():
            p = line.split()
            if p and p[0] in ("T1", "T4"):
                d[p[1]] = (float(p[2]), float(p[3]))
        assert d, o
        res[b].append(d)
tag = "-".join(x.split(".")[0] for x in bins)
json.dump(res, open(f"{S}/res-{mode}-{sset}-{tag}.json", "w"))
T = {3: 3.182, 5: 2.571, 7: 2.365, 9: 2.262, 11: 2.201, 15: 2.131, 19: 2.093}
for b in bins:
    print(
        f"== {b} / master kernel, in-process pairs ({mode} {sset}, {rounds} rounds; us, medians over rounds)"
    )
    for k in res[b][0]:
        lr = [math.log(x[k][1] / x[k][0]) for x in res[b]]
        m = st.mean(lr)
        se = st.stdev(lr) / math.sqrt(len(lr))
        t = T.get(len(lr) - 1, 2.0)
        a = st.median(x[k][0] for x in res[b])
        c = st.median(x[k][1] for x in res[b])
        print(
            f"{k:12} master {a:9.1f}  cand {c:9.1f}  ratio {math.exp(m):.3f} [{math.exp(m - t * se):.3f}, {math.exp(m + t * se):.3f}]"
        )
