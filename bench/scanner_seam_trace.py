"""Report the registered scanner maintenance diagnostic from complete occupancy rounds."""
import argparse
import json
import math
import statistics
from pathlib import Path


def maintenance(path, rounds, dry):
    report = json.loads(path.read_text())
    expected = {(r, mode) for r in range(rounds) for mode in ("off", "timeline", "full")}
    seen, times, plans = set(), [], set()
    if report["parity"] is not True:
        raise ValueError(f"{path}: output parity failed")
    for sample in report["runs"]:
        key = (sample["round"], sample["mode"])
        if key not in expected or key in seen or sample["threads"] != 10 or sample["placement"]:
            raise ValueError(f"{path}: invalid sample {key}")
        seen.add(key)
        fields = dict(item.split("=", 1) for item in sample["line"].split() if "=" in item)
        workers, slots = float(fields["t.dispatch_workers"]), float(fields["t.dispatch_slots"])
        plans.add((workers, slots))
        if not dry and (workers, slots) != (8, 32):
            raise ValueError(f"{path}: production dispatched {workers:g} workers, {slots:g} slots")
        if sample["mode"] != "off":
            ms = float(fields["t.in_maint_ms"])
            if fields["t.complete"] != "yes" or not math.isfinite(ms) or ms <= 0:
                raise ValueError(f"{path}: incomplete scanner trace {key}")
            if sample["mode"] == "timeline":
                times.append(ms)
    if seen != expected or len(plans) != 1:
        raise ValueError(f"{path}: missing rounds or varying dispatch")
    return statistics.median(times), plans.pop()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("base", type=Path)
    parser.add_argument("head", type=Path)
    parser.add_argument("rounds", type=int)
    parser.add_argument("--dry", action="store_true")
    args = parser.parse_args()
    base, base_plan = maintenance(args.base, args.rounds, args.dry)
    head, head_plan = maintenance(args.head, args.rounds, args.dry)
    if head_plan != base_plan:
        raise ValueError("base and head dispatched different plans")
    print(f"Diagnostic only: median in_maint over {args.rounds} timeline rounds, "
          f"{base_plan[0]:g} workers/{base_plan[1]:g} slots: "
          f"base {base:.3f} ms, head {head:.3f} ms, reduction {100 * (1 - head / base):.2f}% "
          "(expected >=80%)")
