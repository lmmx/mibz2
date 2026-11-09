"""The service core's revision-2 diagnostics (docs/journal/2026-11-09-service-2x3-r2-registration.md):
the machine state beside the records, in run order. Diagnostic only: it judges nothing, feeds
nothing back to bench/service_2x3.py's report, and computes no ratio, interval or mean within a
clock state.

    python3 bench/service_2x3_diag.py RUNS.jsonl --out STEM      (STEM.md and STEM.json)

1. **The chronological table:** every record in the order it ran (runs.jsonl is appended and
   flushed run by run), with its round, its position in the round, the arm before it, the gap
   since that arm's child exited, its wall and CPU, and its telemetry as recorded.
2. **Each registered comparison's pairs, round by round:** the pair's log ratios (the reference's
   wall over the arm's, as the report forms them), each member's clock state, and the pair's
   state (arm, then reference): HH, HL, LH, LL or unknown. Only counts are given per state,
   never a ratio, interval or mean.

A record's clock state is registered here, fixed before any run and independent of any
outcome:
- **L:** its counted cycles over its counted reference cycles below 1, which is below the
  reference rate (the counters' scope matched, since both are opened alike);
- **H:** that ratio 1 or above;
- **unknown:** either counter missing, failed (`perf_ctl_errors`) or multiplexed
  (`perf_multiplexed`). Unknown is never imputed.
It is a description of the counted threads' on-CPU time, aggregated over them. It is not a
critical-path frequency, not a cause, and not a quality rule.
"""

import argparse
import json
import math
import os
import sys
from collections import Counter, defaultdict

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from service_2x3 import CONTROL, JUDGED, SECONDARY  # noqa: E402

STATES = ["HH", "HL", "LH", "LL", "unknown"]


def cell(r):
    return f"{r['arm']}@{r['threads']}"


def clock_state(r):
    """H, L or unknown (never imputed), and why where unknown"""
    bad = {e.get("counter") for e in r.get("perf_ctl_errors") or []}
    bad |= set(r.get("perf_multiplexed") or [])
    cy, rc = r.get("cycles"), r.get("ref_cycles")
    if cy is None or rc is None:
        return "unknown", "cycles or reference cycles not recorded"
    if bad & {"cycles", "ref_cycles"}:
        return "unknown", "cycles or reference cycles failed or multiplexed"
    if not rc:
        return "unknown", "no reference cycles counted"
    return ("L" if cy / rc < 1 else "H"), None


def chronological(runs):
    rows, prev = [], None
    for i, r in enumerate(runs):
        if prev is None or prev["round"] != r["round"]:
            pos = 0
        else:
            pos += 1
        gap = None
        if prev is not None and r.get("t_spawn_unix_ns") and prev.get("t_exit_unix_ns"):
            gap = (r["t_spawn_unix_ns"] - prev["t_exit_unix_ns"]) / 1e9
        st, why = clock_state(r)
        pre, post = r.get("machine_pre") or {}, r.get("machine_post") or {}
        rows.append({
            "index": i,
            "round": r.get("round"),
            "position": pos,
            "cell": cell(r),
            "predecessor": cell(prev) if prev is not None else None,
            "gap_s": gap,
            "wall": r.get("wall"),
            "cpu": r.get("cpu"),
            "cycles": r.get("cycles"),
            "ref_cycles": r.get("ref_cycles"),
            "instructions": r.get("instructions"),
            "task_clock_s": r.get("task_clock_s"),
            "eff_ghz": r.get("eff_ghz"),
            "state": st,
            "state_why": why,
            "freq_mhz_pre": pre.get("freq_mhz"),
            "freq_mhz_post": post.get("freq_mhz"),
            "pkg_temp_c_pre": pre.get("pkg_temp_c"),
            "pkg_temp_c_post": post.get("pkg_temp_c"),
            "ctl_errors": len(r.get("perf_ctl_errors") or []),
            "multiplexed": r.get("perf_multiplexed"),
        })
        prev = r
    return rows


def pairs(runs):
    by = defaultdict(dict)
    for r in runs:
        by[cell(r)][r["round"]] = r
    out = {}
    for name, arm, ref in [*JUDGED, *SECONDARY, CONTROL]:
        rs = []
        for rnd in sorted(set(by[arm]) & set(by[ref])):
            a, b = by[arm][rnd], by[ref][rnd]
            sa, sb = clock_state(a)[0], clock_state(b)[0]
            pair = sa + sb if "unknown" not in (sa, sb) else "unknown"
            lw = math.log(b["wall"] / a["wall"]) if a.get("wall") and b.get("wall") else None
            lc = math.log(b["cpu"] / a["cpu"]) if a.get("cpu") and b.get("cpu") else None
            rs.append({"round": rnd, "log_speed": lw, "log_cpu": lc, "arm_state": sa, "ref_state": sb, "pair": pair})
        counts = Counter(x["pair"] for x in rs)
        out[name] = {"arm": arm, "ref": ref, "rounds": rs, "counts": {s: counts.get(s, 0) for s in STATES}}
    return out


def markdown(chron, prs):
    n = lambda v, f="{:.3f}": "–" if v is None else f.format(v)
    lines = [
        "# The machine state beside the records (diagnostic, never judged)",
        "",
        "Every record in run order, and every registered comparison's pairs round by round. Nothing here "
        "feeds the registered report. No ratio, interval or mean is computed within a clock state. A clock "
        "state is cycles over reference cycles (L below 1, H at or above it, unknown where either is missing, "
        "failed or multiplexed). It describes the counted threads' on-CPU time, not a cause.",
        "",
        "## In run order",
        "",
        "| # | round | pos | cell | after | gap s | wall s | CPU s | cycles/ref | state | GHz (cyc/task) | MHz pre → post | °C pre → post | faults |",
        "|--:|--:|--:|---|---|--:|--:|--:|--:|---|--:|--:|--:|---|",
    ]
    for r in chron:
        ratio = r["cycles"] / r["ref_cycles"] if r["cycles"] is not None and r["ref_cycles"] else None
        faults = []
        if r["ctl_errors"]:
            faults.append(f"{r['ctl_errors']} control failures")
        if r["multiplexed"]:
            faults.append("multiplexed: " + ", ".join(r["multiplexed"]))
        lines.append(
            f"| {r['index']} | {r['round']} | {r['position']} | {r['cell']} | {r['predecessor'] or '–'} | "
            f"{n(r['gap_s'], '{:.2f}')} | {n(r['wall'])} | {n(r['cpu'])} | {n(ratio)} | {r['state']} | "
            f"{n(r['eff_ghz'], '{:.2f}')} | {n(r['freq_mhz_pre'], '{:.0f}')} → {n(r['freq_mhz_post'], '{:.0f}')} | "
            f"{n(r['pkg_temp_c_pre'], '{:.0f}')} → {n(r['pkg_temp_c_post'], '{:.0f}')} | {'; '.join(faults) or '–'} |"
        )
    lines += ["", "## Pairs, round by round (arm state, then reference state)", ""]
    for name, p in prs.items():
        c = p["counts"]
        lines += [
            f"### {name}: {p['arm']} against {p['ref']}",
            "",
            "Pairs by state (counts only): " + ", ".join(f"{s} {c[s]}" for s in STATES) + ".",
            "",
            "| round | log speed | log CPU | arm | ref | pair |",
            "|--:|--:|--:|---|---|---|",
        ]
        for x in p["rounds"]:
            lines.append(
                f"| {x['round']} | {n(x['log_speed'], '{:+.4f}')} | {n(x['log_cpu'], '{:+.4f}')} | "
                f"{x['arm_state']} | {x['ref_state']} | {x['pair']} |"
            )
        lines.append("")
    return "\n".join(lines) + "\n"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("runs")
    ap.add_argument("--out", required=True)
    a = ap.parse_args()
    with open(a.runs) as f:
        runs = [json.loads(l) for l in f if l.strip()]
    chron, prs = chronological(runs), pairs(runs)
    with open(a.out + ".json", "w") as f:
        json.dump({"diagnostic": True, "judged": False, "runs": chron, "pairs": prs}, f, indent=1)
    with open(a.out + ".md", "w") as f:
        f.write(markdown(chron, prs))


if __name__ == "__main__":
    main()
