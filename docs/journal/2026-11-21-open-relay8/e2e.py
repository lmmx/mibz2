# e2e.py ROUNDS THREADS: bzcat-base and bzcat-cand on the corpus, fresh processes alternated,
# output to /dev/null: wall, CPU and peak RSS (getrusage of the child), with paired ratios
import os, sys, subprocess, time, resource, math, statistics as st
rounds, threads = int(sys.argv[1]), sys.argv[2]
S = os.path.dirname(os.path.abspath(__file__))
C = S + "/../moonshot/corpora/wikidata.bz2"
res = {"base": [], "cand": []}
for r in range(rounds):
    for arm in (["base", "cand"] if r % 2 == 0 else ["cand", "base"]):
        with open(C, "rb") as f, open(os.devnull, "wb") as o:
            t = time.perf_counter()
            p = subprocess.Popen([S + "/bzcat-" + {"base": "base", "cand": os.environ.get("CAND", "cand")}[arm], threads], stdin=f, stdout=o)
            _, status, ru = os.wait4(p.pid, 0)
            wall = time.perf_counter() - t
        assert status == 0
        res[arm].append((wall, ru.ru_utime + ru.ru_stime, ru.ru_maxrss))
        time.sleep(0.5)
T = {5: 2.571, 7: 2.365, 9: 2.262, 11: 2.201, 15: 2.131, 19: 2.093}
for i, name in enumerate(["wall s", "cpu s", "rss KiB"]):
    lr = [math.log(c[i] / b[i]) for b, c in zip(res["base"], res["cand"])]
    m = st.mean(lr); se = st.stdev(lr) / math.sqrt(len(lr)); t = T.get(len(lr) - 1, 2.0)
    print(f"T={threads} {name:8} base {st.median(x[i] for x in res['base']):10.3f} cand {st.median(x[i] for x in res['cand']):10.3f}  ratio {math.exp(m):.3f} [{math.exp(m-t*se):.3f}, {math.exp(m+t*se):.3f}]")
