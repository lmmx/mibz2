"""P74 Gate 1's figures and its registered rules (docs/journal/2026-10-30-p74-gate1-
preregistration.md), from bench/stages.py's runs.

    p74.py WIKIDATA.jsonl RANDOM.jsonl RECORDS.jsonl --md OUT.md --json OUT.json

WIKIDATA.jsonl is the Stages step's run (the base's and the head's phases, round by round),
RANDOM and RECORDS the head's `--mtf-arms` runs. Each round is one observation: each P74
arm's time over the shipped kernel's in the same run of the same binary (`arms`), a ratio
per round, its mean and a 95% t-interval over the rounds. The layout control is the
head's "MTF + RLE2" stage (production's unchanged `mtf_avx2`) over the base's, paired by
round. The rules are applied as registered; the verdict printed is that mechanical
reading, for the maintainer to confirm.
"""

import argparse
import json
import math
import statistics
from pathlib import Path

KERNELS = ["plain", "checkpoints"]
# the first measurement's arms, then the selector (the mixed-result step: the
# pre-registration's "The selector arm", frozen before its code)
FIRST = ["P74 branch", "P74 free"]
SELECTOR = "P74 selector"
ARMS = FIRST + [SELECTOR]
# t's two-sided 95% quantile by degrees of freedom
T95 = {
    1: 12.706,
    2: 4.303,
    3: 3.182,
    4: 2.776,
    5: 2.571,
    6: 2.447,
    7: 2.365,
    8: 2.306,
    9: 2.262,
    10: 2.228,
    11: 2.201,
    12: 2.179,
    13: 2.160,
    14: 2.145,
    15: 2.131,
    16: 2.120,
    17: 2.110,
    18: 2.101,
    19: 2.093,
    20: 2.086,
}

GAIN = 0.05  # the better arm's registered minimum gain on Wikidata's checkpoint kernel
CONTROL = 0.03  # the controls' registered maximum regression (upper bound)


def interval(xs: list[float]) -> dict:
    n = len(xs)
    m = statistics.fmean(xs)
    if n < 2:
        return {"n": n, "mean": m, "lo": math.nan, "hi": math.nan}
    h = T95.get(n - 1, 1.96) * statistics.stdev(xs) / math.sqrt(n)
    return {"n": n, "mean": m, "lo": m - h, "hi": m + h}


def records(path: str) -> list[dict]:
    return [json.loads(l) for l in Path(path).read_text().splitlines() if l.strip()]


def ratios(recs: list[dict]) -> dict:
    """Each kernel's and arm's per-round ratio over the shipped kernel, from the head's
    runs without errors"""
    out = {k: {a: [] for a in ARMS} for k in KERNELS}
    for r in recs:
        if r.get("arm") != "head" or "err" in r or len(r.get("arms", {})) != 8:
            continue
        for k in KERNELS:
            for a in ARMS:
                out[k][a].append(r["arms"][f"{k}/{a}"] / r["arms"][f"{k}/shipped"])
    return {k: {a: interval(v) for a, v in arms.items() if v} for k, arms in out.items()}


def shipped_cb(recs: list[dict]) -> dict:
    """Each kernel's shipped arm, its median picoseconds a byte over the rounds"""
    return {
        k: statistics.median(
            r["arms"][f"{k}/shipped"] for r in recs if r.get("arm") == "head" and "arms" in r
        )
        for k in KERNELS
    }


def layout(recs: list[dict]) -> dict | None:
    """The head's MTF + RLE2 stage over the base's, paired by round"""
    by = {}
    for r in recs:
        if "err" not in r and "mtf" in r.get("ps_per_byte", {}):
            by.setdefault(r["round"], {})[r["arm"]] = r["ps_per_byte"]["mtf"]
    xs = [v["head"] / v["base"] for v in by.values() if {"head", "base"} <= set(v)]
    return interval(xs) if xs else None


def selections(recs: list[dict]) -> dict | None:
    """The selector's choices on a corpus (the same in every run): blocks and L bytes"""
    return next((r["selection"] for r in recs if "selection" in r), None)


def selector_verdict(fig: dict) -> dict:
    """The selector arm's frozen rule, read mechanically: positive or not (P74 closes)"""
    wd = fig["wikidata"]["checkpoints"][SELECTOR]
    gain = 1 - wd["mean"]
    lay = fig["layout"]
    noise = abs(1 - lay["mean"]) if lay else math.nan
    wiki_ok = gain >= GAIN and wd["hi"] < 1 and (lay is None or gain > 2 * noise)
    controls = {
        f"{c}/{k}": fig[c][k][SELECTOR]["hi"] for c in ("random", "records") for k in KERNELS
    }
    controls_ok = all(hi <= 1 + CONTROL for hi in controls.values())
    return {
        "gain": gain,
        "layout_noise": noise,
        "wikidata_rule": wiki_ok,
        "controls_upper": controls,
        "controls_rule": controls_ok,
        "verdict": "positive" if wiki_ok and controls_ok else "negative (P74 closes)",
    }


def verdict(fig: dict) -> dict:
    """The registered rules, read mechanically (the first measurement's two arms)"""
    wd = fig["wikidata"]["checkpoints"]
    best = min(FIRST, key=lambda a: wd[a]["mean"])
    gain = 1 - wd[best]["mean"]
    lay = fig["layout"]
    noise = abs(1 - lay["mean"]) if lay else math.nan
    wiki_ok = gain >= GAIN and wd[best]["hi"] < 1 and (lay is None or gain > 2 * noise)
    controls = {f"{c}/{k}": fig[c][k][best]["hi"] for c in ("random", "records") for k in KERNELS}
    controls_ok = all(hi <= 1 + CONTROL for hi in controls.values())
    if wiki_ok and controls_ok:
        v = "positive"
    elif wiki_ok:
        v = "mixed"
    else:
        v = "negative"
    return {
        "better_arm": best,
        "gain": gain,
        "layout_noise": noise,
        "wikidata_rule": wiki_ok,
        "controls_upper": controls,
        "controls_rule": controls_ok,
        "verdict": v,
    }


def pct(x: float) -> str:
    return f"{100 * (x - 1):+.1f}%"


def md(fig: dict, v: dict, sv: dict) -> str:
    lines = [
        "### P74 Gate 1: the MTF kernels, same binary (pre-registered)",
        "",
        "Each P74 arm against the shipped kernel in the same run of the same binary, one ratio a "
        "round; mean and 95% t-interval over the rounds (negative: faster). "
        "`docs/journal/2026-10-30-p74-gate1-preregistration.md`.",
        "",
        "| corpus | kernel | shipped, ns/B | P74 branch | P74 free | P74 selector |",
        "|---|---|--:|--:|--:|--:|",
    ]
    for c in ("wikidata", "random", "records"):
        for k in KERNELS:
            cells = []
            for a in ARMS:
                x = fig[c][k].get(a)
                cells.append(
                    f"{pct(x['mean'])} ({pct(x['lo'])} to {pct(x['hi'])}), n={x['n']}" if x else "–"
                )
            lines.append(
                f"| {c} | {k} | {fig['shipped'][c][k] / 1000:.3f} | " + " | ".join(cells) + " |"
            )
    lay = fig["layout"]
    lines += [
        "",
        "**Layout control** (the head's unchanged `MTF + RLE2` over the base's, paired by round): "
        + (
            f"{pct(lay['mean'])} ({pct(lay['lo'])} to {pct(lay['hi'])}), n={lay['n']}"
            if lay
            else "not available"
        ),
        "",
        f"**The registered rules, read mechanically** (to be confirmed): the better arm on Wikidata's "
        f"checkpoint kernel is **{v['better_arm']}**, gain {100 * v['gain']:.1f}% "
        f"(rule: ≥ {100 * GAIN:.0f}%, interval excluding zero, more than twice the layout noise "
        f"{100 * v['layout_noise']:.1f}%): **{'met' if v['wikidata_rule'] else 'not met'}**; "
        f"controls' upper bounds "
        + ", ".join(f"{k} {pct(hi)}" for k, hi in v["controls_upper"].items())
        + f" (rule: ≤ +{100 * CONTROL:.0f}%): **{'met' if v['controls_rule'] else 'not met'}**. "
        f"Verdict: **{v['verdict']}**.",
        "",
        "**The selector's choices** (the frozen rule: P74 free if A ≤ 32, or E = 0, or "
        "10·F ≤ E; else the shipped kernel; its count timed in the arm):",
        "",
        "| corpus | blocks to P74 | L bytes to P74 |",
        "|---|--:|--:|",
    ]
    for c in ("wikidata", "random", "records"):
        sel = fig["selections"].get(c)
        if sel:
            lines.append(
                f"| {c} | {sel['blocks']} of {sel['of_blocks']} | "
                f"{100 * sel['bytes'] / max(1, sel['of_bytes']):.1f}% |"
            )
    lines += [
        "",
        f"**The selector arm's frozen rule, read mechanically** (to be confirmed): Wikidata's "
        f"checkpoint kernel gain {100 * sv['gain']:.1f}% (≥ {100 * GAIN:.0f}%, interval excluding "
        f"zero, more than twice the layout noise {100 * sv['layout_noise']:.1f}%): "
        f"**{'met' if sv['wikidata_rule'] else 'not met'}**; controls' upper bounds "
        + ", ".join(f"{k} {pct(hi)}" for k, hi in sv["controls_upper"].items())
        + f" (≤ +{100 * CONTROL:.0f}%): **{'met' if sv['controls_rule'] else 'not met'}**. "
        f"Verdict: **{sv['verdict']}**.",
        "",
    ]
    return "\n".join(lines)


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("wikidata")
    ap.add_argument("random")
    ap.add_argument("records")
    ap.add_argument("--md", required=True)
    ap.add_argument("--json", required=True)
    a = ap.parse_args()
    runs = {
        "wikidata": records(a.wikidata),
        "random": records(a.random),
        "records": records(a.records),
    }
    fig = {c: ratios(r) for c, r in runs.items()}
    fig["shipped"] = {c: shipped_cb(r) for c, r in runs.items()}
    fig["layout"] = layout(runs["wikidata"])
    fig["selections"] = {c: selections(r) for c, r in runs.items()}
    v = verdict(fig)
    sv = selector_verdict(fig)
    Path(a.json).write_text(
        json.dumps({"figures": fig, "rules": v, "selector_rules": sv}, indent=1)
    )
    Path(a.md).write_text(md(fig, v, sv))
    print(md(fig, v, sv))


if __name__ == "__main__":
    main()
