"""Observation cost and safe-point latency under a bounded active-work fixture.

Isolated temporary bare remote only. The work loop runs fixed-length batches and
observes at the first safe point (batch boundary) once `interval` seconds have
passed since the last observation; a protocol change is pushed part-way through.
Time is scaled down (interval 0.5 s stands in for the 5-minute target), so the
latency bound, not the absolute number, is what transfers.
"""
import argparse
import json
import pathlib
import statistics
import tempfile
import threading
import time

from checkpoint import CLIENT, observe
from test_checkpoint import git


def fixture(root):
    remote, seed, agent = root / "remote.git", root / "seed", root / "agent"
    git(root, "init", "--bare", "-b", "master", str(remote))
    git(root, "clone", str(remote), str(seed))
    git(seed, "config", "user.name", "Test")
    git(seed, "config", "user.email", "test@example.invalid")
    (seed / CLIENT).parent.mkdir(parents=True)
    (seed / CLIENT).write_text("client v1\n")
    claims = seed / "frontier/claims"
    claims.mkdir(parents=True)
    for i in range(40):  # a frontier of realistic size: forty claims, twenty flags
        (claims / f"c{i}.toml").write_text(f'id = "c{i}"\npremises = ["research/c{i}/"]\nowned_paths = []\n')
    flags = seed / "frontier/flags"
    flags.mkdir(parents=True)
    for i in range(20):
        (flags / f"f{i}.md").write_text(f"flags: c{i}\ninvariant: x\nby: t\nstatus: resolved\n")
    git(seed, "add", "-A")
    git(seed, "commit", "-qm", "initial")
    git(seed, "push", "-q", "origin", "master")
    git(root, "clone", "-q", str(remote), str(agent))
    (agent / "dirty-research.txt").write_text("unfinished\n")
    return seed, agent


def cost(agent, claim, n):
    samples = []
    for _ in range(n):
        t = time.perf_counter()
        observe(agent, claim=claim, cursor=agent / ".git" / "cursor-cost")
        samples.append(time.perf_counter() - t)
    return {"median_s": round(statistics.median(samples), 4), "max_s": round(max(samples), 4)}


def latency(seed, agent, batch, interval, push_at, batches):
    cursor = agent / ".git" / "cursor-latency"
    observe(agent, cursor=cursor)
    pushed = {}

    def change():
        time.sleep(push_at)
        (seed / CLIENT).write_text("client v2\n")
        git(seed, "commit", "-qam", "protocol change")
        git(seed, "push", "-q", "origin", "master")
        pushed["t"] = time.perf_counter()

    thread = threading.Thread(target=change)
    thread.start()
    last, observations = time.perf_counter(), 0
    for _ in range(batches):
        time.sleep(batch)  # one indivisible unit of active work
        if time.perf_counter() - last >= interval:
            result = observe(agent, cursor=cursor)
            last, observations = time.perf_counter(), observations + 1
            if not result["client_current"] and "t" in pushed:
                thread.join()
                return {"latency_s": round(last - pushed["t"], 3), "observations": observations}
    thread.join()
    return {"latency_s": None, "observations": observations}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--batch", type=float, default=0.2)
    parser.add_argument("--interval", type=float, default=0.5)
    parser.add_argument("--repeats", type=int, default=5)
    args = parser.parse_args()
    runs = []
    for r in range(args.repeats):
        with tempfile.TemporaryDirectory() as tmp:
            seed, agent = fixture(pathlib.Path(tmp))
            if r == 0:
                unchanged = cost(agent, None, 20)
                scoped = cost(agent, "c3", 20)
            runs.append(latency(seed, agent, args.batch, args.interval, 0.3 + 0.17 * r, 40))
    lat = [x["latency_s"] for x in runs if x["latency_s"] is not None]
    print(json.dumps({
        "batch_s": args.batch, "interval_s": args.interval,
        "bound_s": round(args.interval + args.batch, 3),
        "cost_unchanged": unchanged, "cost_scoped_claim": scoped,
        "latency_runs": runs, "latency_max_s": max(lat) if lat else None,
    }, indent=2))


if __name__ == "__main__":
    main()
