import os, resource, subprocess, sys, time, statistics, math

sp, w = sys.argv[1], sys.argv[2]
rounds = int(sys.argv[3])
rows = []
for t in (1, 2):
    cpus = "1" if t == 1 else "1,2"
    for r in range(rounds):
        order = ["base", "branch"] if r % 2 == 0 else ["branch", "base"]
        for v in order:
            pid = os.fork()
            if pid == 0:
                os.sched_setaffinity(0, {int(c) for c in cpus.split(",")})
                fd = os.open(os.devnull, os.O_WRONLY)
                os.dup2(fd, 1)
                os.execv(f"{sp}/{v}/target/release/c30cg", ["c30cg", "time", w, str(t), "1"])
            t0 = time.perf_counter()
            _, status, ru = os.wait4(pid, 0)
            wall = time.perf_counter() - t0
            assert status == 0
            rows.append((t, r, v, wall, ru.ru_utime + ru.ru_stime, ru.ru_maxrss / 1024))
for t in (1, 2):
    for k, name in ((3, "wall"), (4, "cpu"), (5, "rss_mb")):
        b = [x[k] for x in rows if x[0] == t and x[2] == "base"]
        c = [x[k] for x in rows if x[0] == t and x[2] == "branch"]
        ratios = [math.log(ci / bi) for bi, ci in zip(b, c)]
        m = statistics.mean(ratios)
        s = statistics.stdev(ratios) / math.sqrt(len(ratios))
        print(
            f"threads {t} {name}: base median {statistics.median(b):.3f} branch median {statistics.median(c):.3f} ratio {math.exp(m):.4f} (approx 95% [{math.exp(m - 2.57 * s):.4f}, {math.exp(m + 2.57 * s):.4f}], n={len(b)})"
        )
