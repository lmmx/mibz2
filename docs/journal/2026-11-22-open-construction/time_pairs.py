from pathlib import Path
import json, os, sys, subprocess, time

exe, name, armstr, modes, rounds = sys.argv[1:]
modes = modes.split(",")
arms = armstr.split(",")
output = Path(name + ".jsonl")
already = (
    {tuple(json.loads(l)[:4]) for l in output.read_text().splitlines()}
    if output.exists()
    else set()
)
for r in range(int(rounds)):
    for conf in modes:
        dataset = conf[0]
        mode = conf[1:]
        w = 4 if mode in ("t4", "stage4", "corpus4", "cs4") else 1
        if mode.startswith(("corpus", "cs")):
            mode = mode[:-1]
        if mode == "t2":
            w, mode = 2, "t4"
        reps = {"t1": 41, "t4": 30, "stage": 15, "stage4": 15}.get(mode, 1)
        for arm in arms if r % 2 == 0 else arms[::-1]:
            key = (r, conf, w, arm)
            if key in already:
                continue
            env = dict(
                os.environ,
                DIET_MODE=mode,
                DIET_REPS=str(reps),
                REVERSE=str(r % 2),
                WORKERS=str(w),
            )
            if dataset == "d":
                env["MIBZ2_ADMITTED_BWT"] = env["DENSE_DIR"]
            cpu = (
                "2"
                if w == 1 and mode not in ("corpus", "cs")
                else "0-1"
                if w == 2
                else "0-3"
            )
            cmd = [
                "taskset",
                "-c",
                cpu,
                exe,
                f"timing::arm_{arm}::diet_harness",
                "--ignored",
                "--exact",
                "--nocapture",
                "--test-threads=1",
            ]
            result = subprocess.run(
                cmd,
                env=env,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                text=True,
            )
            rows = {}
            for line in result.stdout.splitlines():
                cols = line.split(" ... ")[-1].split()
                if cols and cols[0] in ("ST", "CORP", "T1", "T4", "MEM"):
                    rows[cols[1]] = list(map(float, cols[2:]))
            if result.returncode or not rows:
                sys.exit(result.stdout)
            with output.open("a") as f:
                f.write(json.dumps([*key, rows]) + "\n")
    print(name, r + 1, time.asctime(), flush=True)
