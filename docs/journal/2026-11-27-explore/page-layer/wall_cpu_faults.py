import os, sys, time, resource, subprocess, statistics, json
binp, inp = sys.argv[1], sys.argv[2]
threads = [int(x) for x in sys.argv[3].split(",")]
reps = int(sys.argv[4])
arms = {"default": {}, "hugetlb1": {"GLIBC_TUNABLES": "glibc.malloc.hugetlb=1"},
        "nommap": {"GLIBC_TUNABLES": "glibc.malloc.mmap_threshold=4294967295:glibc.malloc.trim_threshold=4294967295"}}
sel = sys.argv[5].split(",") if len(sys.argv) > 5 else list(arms)
res = {}
for t in threads:
    for r in range(reps):
        for a in sel:   # interleaved
            env = dict(os.environ, **arms[a])
            with open(inp, "rb") as f:
                r0 = resource.getrusage(resource.RUSAGE_CHILDREN)
                t0 = time.perf_counter()
                p = subprocess.run([binp, str(t)], stdin=f, stdout=subprocess.DEVNULL, env=env)
                wall = time.perf_counter() - t0
                r1 = resource.getrusage(resource.RUSAGE_CHILDREN)
            assert p.returncode == 0
            cpu = (r1.ru_utime + r1.ru_stime) - (r0.ru_utime + r0.ru_stime)
            sys_ = r1.ru_stime - r0.ru_stime
            res.setdefault((t, a), []).append((wall, cpu, sys_, r1.ru_minflt - r0.ru_minflt, r1.ru_maxrss))
for (t, a), v in sorted(res.items()):
    med = lambda i: statistics.median(x[i] for x in v)
    print(f"t={t} {a:9s} wall {med(0)*1e3:8.1f} ms  cpu {med(1)*1e3:8.1f} ms  sys {med(2)*1e3:7.1f} ms  minflt {med(3):9.0f}  maxrss(cum) {max(x[4] for x in v)/1024:.0f} MiB")
