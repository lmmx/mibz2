"""run.py OUT.jsonl WORKLOAD FILE THREADS DECODER... : time each decoder on FILE.

Each decoder is a command writing the decoded bytes to stdout, read here through a pipe
(1 MiB, F_SETPIPE_SZ). Run 0 hashes the output (sha256, here, so the hash's CPU is not the
child's); runs 1..REPS discard it (no hashing to compete for the cores). For each run: wall
(spawn to exit), time to first output byte, the child's user+sys CPU and peak RSS
(wait4), exit code. One-thread runs are pinned to CPU 1 (taskset), so a decoder's reader
or writer threads share its one core; THREADS > 1 runs are not pinned.
The 1-minute load average is recorded before every run. Rounds are interleaved.

Env: BAKEOFF_BIN (the harness binary), BAKEOFF_PY (the venv's python, for indexed_bzip2),
BAKEOFF_BIN_NATIVE (the harness built with -C target-cpu=native, for DECODER-native),
REPS (default 3).
"""
import fcntl, hashlib, json, os, subprocess, sys, time

HERE = os.path.dirname(os.path.abspath(__file__))
BIN = os.environ["BAKEOFF_BIN"]
PY = os.environ.get("BAKEOFF_PY", sys.executable)
REPS = int(os.environ.get("REPS", "3"))


def command(dec: str, threads: int, path: str) -> list[str]:
    if dec == "bzip2":
        return ["bzip2", "-dc", path]
    if dec == "lbzip2":
        return ["lbzip2", "-dc", "-n", str(threads), path]
    if dec.endswith("-native"):  # the same harness built with RUSTFLAGS="-C target-cpu=native"
        return [os.environ["BAKEOFF_BIN_NATIVE"], dec.removesuffix("-native"), str(threads), path]
    if dec == "indexed_bzip2":
        return [PY, "-I", os.path.join(HERE, "ib.py"), str(threads), path]
    return [BIN, dec, str(threads), path]


def cpu_ticks() -> tuple[int, int]:
    """(steal, total) jiffies over all CPUs, from /proc/stat"""
    f = [int(x) for x in open("/proc/stat").readline().split()[1:]]
    return f[7], sum(f[:8])


def once(cmd: list[str], hash_it: bool) -> dict:
    st0, tot0 = cpu_ticks()
    r, w = os.pipe()
    fcntl.fcntl(r, 1031, 1 << 20)  # F_SETPIPE_SZ
    t0 = time.perf_counter()
    p = subprocess.Popen(cmd, stdout=w, stderr=subprocess.PIPE)
    os.close(w)
    buf = bytearray(1 << 20)
    mv = memoryview(buf)
    h = hashlib.sha256() if hash_it else None
    n, first = 0, None
    while True:
        k = os.readv(r, [buf])
        if k == 0:
            break
        if first is None:
            first = time.perf_counter() - t0
        n += k
        if h:
            h.update(mv[:k])
    os.close(r)
    err = p.stderr.read().decode(errors="replace")
    _, status, ru = os.wait4(p.pid, 0)
    wall = time.perf_counter() - t0
    p.returncode = os.waitstatus_to_exitcode(status)
    st1, tot1 = cpu_ticks()
    return {
        "steal_pct": round(100 * (st1 - st0) / max(1, tot1 - tot0), 1),
        "wall_s": round(wall, 3),
        "ttfo_s": None if first is None else round(first, 3),
        "cpu_s": round(ru.ru_utime + ru.ru_stime, 3),
        "user_s": round(ru.ru_utime, 3),
        "sys_s": round(ru.ru_stime, 3),
        "rss_mb": round(ru.ru_maxrss / 1024, 1),
        "out_bytes": n,
        "exit": p.returncode,
        "sha256": h.hexdigest() if h else None,
        "stderr": err[-300:],
    }


def main() -> None:
    out, workload, path, threads, decs = sys.argv[1], sys.argv[2], sys.argv[3], int(sys.argv[4]), sys.argv[5:]
    cmds = {}
    for dec in decs:
        cmd = command(dec, threads, path)
        cmds[dec] = ["taskset", "-c", "1"] + cmd if threads == 1 else cmd
    # Rounds interleaved (round 0 hashes): a drift in the machine's speed (this is a shared
    # VM) falls on every decoder alike rather than on whichever ran last.
    runs = {d: [] for d in decs}
    loads = {d: [] for d in decs}
    for i in range(REPS + 1):
        for dec in decs:
            loads[dec].append(round(os.getloadavg()[0], 2))
            runs[dec].append(once(cmds[dec], i == 0))
    for dec in decs:
        rs = runs[dec]
        rec = {"workload": workload, "file": os.path.basename(path), "in_bytes": os.path.getsize(path),
               "threads": threads, "decoder": dec, "load1_before": loads[dec], "cmd": cmds[dec], "runs": rs}
        with open(out, "a") as f:
            f.write(json.dumps(rec) + "\n")
        t = sorted(r["wall_s"] for r in rs[1:]) or [rs[0]["wall_s"]]
        print(f"{workload} t={threads} {dec:14s} load={max(loads[dec]):.2f} best={t[0]:.2f} med={t[len(t)//2]:.2f} "
              f"cpu={min(r['cpu_s'] for r in rs):.2f} rss={max(r['rss_mb'] for r in rs):.0f}MB out={rs[0]['out_bytes']} "
              f"exit={rs[0]['exit']} sha={(rs[0]['sha256'] or '')[:12]}", flush=True)


if __name__ == "__main__":
    main()
