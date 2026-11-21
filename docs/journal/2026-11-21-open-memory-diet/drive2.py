# drive2.py MODE ROUNDS BIN...: each BIN's in-process pairs against master's kernel, over rounds
import json, os, subprocess, sys, math, statistics as st
mode, rounds, bins = sys.argv[1], int(sys.argv[2]), sys.argv[3:]
S = os.path.dirname(os.path.abspath(__file__))
env = dict(os.environ, MIBZ2_ADMITTED_BWT=S + "/../wt-data/admitted-bwt", DIET_MODE=mode)
cpu = os.environ.get("CPU", "2" if mode == "t1" else "0-3")
res = {b: [] for b in bins}
for r in range(rounds):
    for b in (bins if r % 2 == 0 else bins[::-1]):
        o = subprocess.run(["taskset", "-c", cpu, S + "/" + b, "open::diet_harness::diet_harness",
                            "--ignored", "--exact", "--nocapture", "--test-threads=1"],
                           env=env, capture_output=True, text=True).stdout
        d = {}
        for line in o.replace("test open::diet_harness::diet_harness ... ", "").splitlines():
            p = line.split()
            if p and p[0] in ("T1", "T4"):
                d[p[1]] = (float(p[2]), float(p[3]))
        assert d, o
        res[b].append(d)
tag = "-".join(x.split(".")[0] for x in bins)
json.dump(res, open(f"{S}/res2-{mode}-{tag}.json", "w"))
T = {3: 3.182, 5: 2.571, 7: 2.365, 9: 2.262, 11: 2.201, 15: 2.131, 19: 2.093}
for b in bins:
    print(f"== {b} / master kernel, in-process pairs ({mode}, {rounds} rounds; times in us, medians over rounds)")
    for k in res[b][0]:
        lr = [math.log(x[k][1] / x[k][0]) for x in res[b]]
        m = st.mean(lr); se = st.stdev(lr) / math.sqrt(len(lr)); t = T.get(len(lr) - 1, 2.0)
        a = st.median(x[k][0] for x in res[b]); c = st.median(x[k][1] for x in res[b])
        print(f"{k:12} master {a:9.1f}  cand {c:9.1f}  ratio {math.exp(m):.3f} [{math.exp(m - t*se):.3f}, {math.exp(m + t*se):.3f}]")
