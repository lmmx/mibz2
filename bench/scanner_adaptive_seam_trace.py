"""The registered traced in_maint gate for bench-scanner-adaptive-seam
(docs/journal/2026-11-27-scanner-adaptive-seam-window/registration.md §4-§5): median scanner
window maintenance (`InputGot`/`WindowResume` -> `WindowWorkEnd`) over the timeline rounds of
`examples/occupancy` at 10 threads, both builds dispatching production's 8 workers and 32
slots, and the window's peak reserved capacity. Writes a JSON summary the verdict reads.

    scanner_adaptive_seam_trace.py BASE.jsonl HEAD.jsonl ROUNDS OUT.json [--dry]

Raises (exit 1, so the verdict is VOID) on any missing, duplicated or incomplete round, any
dispatch other than 8 workers / 32 slots (unless --dry), or base and head dispatching
different plans.
"""
import argparse
import json
import math
import statistics
from pathlib import Path

GATE_DROP = 0.85  # in_maint head/base <= 0.15
CAP_GATE = 1.00  # window_cap_max_mb head/base <= 1.00


def fields(line):
    return dict(item.split("=", 1) for item in line.split() if "=" in item)


def maintenance(path, rounds, dry):
    report = json.loads(path.read_text())
    expected = {(r, mode) for r in range(rounds) for mode in ("off", "timeline", "full")}
    seen, times, spans, caps, plans = set(), [], [], [], set()
    if report["parity"] is not True:
        raise ValueError(f"{path}: output parity failed")
    for sample in report["runs"]:
        key = (sample["round"], sample["mode"])
        if key not in expected or key in seen or sample["threads"] != 10 or sample["placement"]:
            raise ValueError(f"{path}: invalid sample {key}")
        seen.add(key)
        f = fields(sample["line"])
        workers, slots = float(f["t.dispatch_workers"]), float(f["t.dispatch_slots"])
        plans.add((workers, slots))
        if not dry and (workers, slots) != (8, 32):
            raise ValueError(f"{path}: production dispatched {workers:g} workers, {slots:g} slots")
        if sample["mode"] != "off":
            ms = float(f["t.in_maint_ms"])
            if f["t.complete"] != "yes" or not math.isfinite(ms) or ms <= 0:
                raise ValueError(f"{path}: incomplete scanner trace {key}")
            if sample["mode"] == "timeline":
                times.append(ms)
                spans.append(float(f["t.in_maint_spans"]))
            cap = float(f["t.f.window_cap_max_mb"])
            if not math.isfinite(cap) or cap <= 0:
                raise ValueError(f"{path}: no window capacity in {key}")
            caps.append(cap)
    if seen != expected or len(plans) != 1:
        raise ValueError(f"{path}: missing rounds or varying dispatch")
    return {
        "in_maint_ms": statistics.median(times),
        "in_maint_spans": statistics.median(spans),
        "window_cap_max_mb": max(caps),
        "workers": plans.copy().pop()[0],
        "slots": plans.pop()[1],
    }


def summarise(base, head):
    if (base["workers"], base["slots"]) != (head["workers"], head["slots"]):
        raise ValueError("base and head dispatched different plans")
    ratio = head["in_maint_ms"] / base["in_maint_ms"]
    cap = head["window_cap_max_mb"] / base["window_cap_max_mb"]
    return {
        "base": base,
        "head": head,
        "in_maint_ratio": ratio,
        "in_maint_drop": 1 - ratio,
        "window_cap_ratio": cap,
        "passes": ratio <= 1 - GATE_DROP and cap <= CAP_GATE,
    }


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("base", type=Path)
    parser.add_argument("head", type=Path)
    parser.add_argument("rounds", type=int)
    parser.add_argument("out", type=Path)
    parser.add_argument("--dry", action="store_true")
    args = parser.parse_args()
    s = summarise(maintenance(args.base, args.rounds, args.dry), maintenance(args.head, args.rounds, args.dry))
    args.out.write_text(json.dumps(s, indent=2) + "\n")
    b, h = s["base"], s["head"]
    print(f"traced in_maint over {args.rounds} timeline rounds, {b['workers']:g} workers/{b['slots']:g} slots: "
          f"base {b['in_maint_ms']:.3f} ms ({b['in_maint_spans']:g} spans), head {h['in_maint_ms']:.3f} ms "
          f"({h['in_maint_spans']:g} spans), drop {100 * s['in_maint_drop']:.2f}% (gate >= 85%); "
          f"window cap base {b['window_cap_max_mb']:.2f} MB, head {h['window_cap_max_mb']:.2f} MB "
          f"({s['window_cap_ratio']:.4f}, gate <= 1.00)")
