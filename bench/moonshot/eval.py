#!/usr/bin/env python3
"""The moonshot scorer's retrospective (bench/moonshot/README.md, "The retrospective"): every
candidate's local forecast beside every later independent result for the same frozen source.

    python3 bench/moonshot/eval.py [DIR ...]          (default research/astra-e)

Read-only over the records; it fits nothing.

What a local forecast is (each over 0.5.6's LazyCells, at one worker count W):
- a score's `summary.json`: the local wall ratio with its interval, and the proxy's predicted ratio
  where the score ran it (v0's and v1's layouts both read);
- a register line (`--record`'s JSONL, or any `*register*.jsonl`): the same, with the candidate
  source's SHA-256.

What an independent result is: any JSON object under a key ending `ci_run` or `i9_run` (an
`assessment.json`'s, for instance) holding `machine`, `candidate_source_sha256` (or `selected`, a
path) and `candidate_over_reference` -> "master LazyCells" -> W -> {geomean, lo, hi}.

A forecast and a result are joined by the candidate source's SHA-256, else by sharing a results
directory, else by the candidate's path (flagged: a path's source may have changed). An error is
scored as calibration only when the result is on the host the forecast is for, at the same W:
- the local wall ratio forecasts the local host;
- the proxy forecasts the i9 (the target).

Every other pairing is printed with its flag and left out of the accuracy summary.
"""

from __future__ import annotations

import json
import math
import sys
from pathlib import Path

LAZY = "master LazyCells"


def load_json(p: Path):
    try:
        return json.loads(p.read_text())
    except (OSError, ValueError):
        return None


def proxy_ratio(s: dict, w: str) -> float | None:
    """the proxy's predicted candidate/lazy at W: v1's {arm: {W: {ratio}}} or v0's {W: {arm: x}}"""
    p = (s.get("proxy") or {}).get("predicted_over_lazy") or {}
    c = p.get("candidate")
    if isinstance(c, dict) and isinstance(c.get(w), dict):
        return c[w].get("ratio")
    x = p.get(w)
    if isinstance(x, dict) and isinstance(x.get("candidate"), (int, float)):
        return x["candidate"]
    return None


def forecasts(root: Path) -> list[dict]:
    """every local forecast under `root`: one row per candidate, W"""
    rows = []
    for f in sorted(root.rglob("summary.json")):
        s = load_json(f)
        if not isinstance(s, dict) or "info" not in s or "gate" not in s:
            continue
        sc = s.get("scorer") or {}
        ws = set((s.get("timing") or {}).keys())
        p = (s.get("proxy") or {}).get("predicted_over_lazy") or {}
        ws |= (
            set((p.get("candidate") or {}).keys())
            if isinstance(p.get("candidate"), dict)
            else set(k for k in p if k.isdigit())
        )
        rec = f.parent.parent if f.parent.name == "score" else f.parent
        for w in ws:
            t = ((s.get("timing") or {}).get(w) or {}).get("candidate") or {}
            rows.append(
                {
                    "source": str(f),
                    "dir": str(rec),
                    "name": rec.name,
                    "candidate": s["info"].get("candidate"),
                    "sha256": None,
                    "commit": None,
                    "version": sc.get("version", "v0"),
                    "scorer_commit": sc.get("commit"),
                    "gate": (s.get("gate") or {}).get("verdict"),
                    "w": w,
                    "local": t.get("geomean"),
                    "lo": t.get("lo"),
                    "hi": t.get("hi"),
                    "proxy": proxy_ratio(s, w),
                }
            )
    for f in sorted(root.rglob("*register*.jsonl")):
        for line in f.read_text().splitlines():
            try:
                r = json.loads(line)
            except ValueError:
                continue
            rec = f.parent
            for w, x in (r.get("walls") or {}).items():
                rows.append(
                    {
                        "source": str(f),
                        "dir": str(rec),
                        "name": rec.name,
                        "candidate": r.get("candidate"),
                        "sha256": r.get("candidate_sha256"),
                        "commit": r.get("candidate_commit"),
                        "version": r.get("scorer_version", "unversioned"),
                        "scorer_commit": r.get("scorer_commit"),
                        "gate": r.get("gate"),
                        "w": w,
                        "local": x.get("ratio"),
                        "lo": x.get("lo"),
                        "hi": x.get("hi"),
                        "proxy": ((r.get("proxy_advisory") or {}).get(w) or {}).get("ratio"),
                    }
                )
    # (a summary and its register line for the same score are one forecast: keep the register's,
    # which carries the source's SHA-256)
    seen, out = set(), []
    for r in sorted(rows, key=lambda r: r["sha256"] is None):
        k = (r["dir"], r["w"], round(r["local"] or 0, 6))
        if k not in seen:
            seen.add(k)
            out.append(r)
    return out


def results(root: Path) -> list[dict]:
    """every independent result under `root`: one row per run, W"""
    rows = []
    for f in sorted(root.rglob("*.json")):
        j = load_json(f)
        if not isinstance(j, dict):
            continue
        for key, run in j.items():
            if not (key.endswith("ci_run") or key.endswith("i9_run")) or not isinstance(run, dict):
                continue
            ref = (run.get("candidate_over_reference") or {}).get(LAZY) or {}
            rec = f.parent
            for w, x in ref.items():
                rows.append(
                    {
                        "source": f"{f}:{key}",
                        "dir": str(rec),
                        "sha256": run.get("candidate_source_sha256"),
                        "path": run.get("selected"),
                        "head": run.get("head"),
                        "host": run.get("machine") or "unknown",
                        "i9": "i9-10900K" in (run.get("machine") or "") or key.endswith("i9_run"),
                        "w": w,
                        "ratio": x.get("geomean"),
                        "lo": x.get("lo"),
                        "hi": x.get("hi"),
                    }
                )
    return rows


def join(fc: list[dict], res: list[dict]) -> list[dict]:
    """each forecast beside each result for the same source (and each result with no forecast)"""
    out, used = [], set()
    for r in fc:
        cand = r.get("candidate") or ""
        matched = []
        for i, x in enumerate(res):
            if r["sha256"] and x["sha256"]:
                how = "sha256" if r["sha256"] == x["sha256"] else None
            elif r["dir"] == x["dir"]:
                how = "directory"
            elif x["path"] and cand.endswith(x["path"]):
                how = "path only"
            else:
                how = None
            if how:
                matched.append((i, x, how))
        same_w = [(i, x, how) for i, x, how in matched if x["w"] == r["w"]]
        if not same_w:
            out.append(
                dict(r, indep=None, flags=["no independent result at this W"] if matched else [])
            )
        for i, x, how in same_w:
            used.add(i)
            flags = [] if how == "sha256" else [f"joined by {how}"]
            row = dict(r, indep=x, how=how, flags=flags)
            row["proxy_scored"] = x["i9"] and r["proxy"] is not None and how != "path only"
            if not x["i9"]:
                row["flags"].append(
                    f"host {x['host']}, not the local host or the i9: not calibration"
                )
            out.append(row)
    for i, x in enumerate(res):
        if i not in used:
            out.append(
                {
                    "name": Path(x["dir"]).name,
                    "w": x["w"],
                    "version": "-",
                    "indep": x,
                    "local": None,
                    "proxy": None,
                    "flags": ["no local forecast at this W"],
                }
            )
    return out


def err(pred: float | None, meas: float | None) -> float | None:
    return None if pred is None or meas is None else math.log(pred / meas)


def accuracy(rows: list[dict], key: str) -> dict:
    """mean absolute log error and pairwise rank agreement (same W and host) of `key` forecasts"""
    e = [abs(err(r[key], r["indep"]["ratio"])) for r in rows]
    pairs = right = 0
    for i, a in enumerate(rows):
        for b in rows[i + 1 :]:
            if a["w"] != b["w"] or a["indep"]["host"] != b["indep"]["host"]:
                continue
            pairs += 1
            right += (a[key] < b[key]) == (a["indep"]["ratio"] < b["indep"]["ratio"])
    return {"n": len(e), "male": sum(e) / len(e) if e else None, "pairs": pairs, "right": right}


def f(x, nd=3) -> str:
    return "-" if x is None else f"{x:.{nd}f}"


def main(argv: list[str]) -> int:
    roots = [Path(a) for a in argv] or [Path("research/astra-e")]
    fc, res = [], []
    for root in roots:
        if not root.exists():
            print(f"moonshot eval: {root} does not exist", file=sys.stderr)
            return 2
        fc += forecasts(root)
        res += results(root)
    rows = sorted(join(fc, res), key=lambda r: (r["name"], int(r["w"])))
    print(
        f"{len(fc)} local forecasts, {len(res)} independent results; ratios over 0.5.6's LazyCells"
    )
    print(
        "record | commit | scorer | W | local [95%] | proxy | independent [95%] (host) | log err local / proxy | flags"
    )
    for r in rows:
        x = r.get("indep")
        ind = f"{f(x['ratio'])} [{f(x['lo'])}, {f(x['hi'])}] ({x['host']})" if x else "-"
        el = f(err(r.get("local"), x and x["ratio"]), 3) if x else "-"
        ep = f(err(r.get("proxy"), x and x["ratio"]), 3) if x else "-"
        loc = f"{f(r.get('local'))} [{f(r.get('lo'))}, {f(r.get('hi'))}]" if r.get("local") else "-"
        commit = (r.get("commit") or (x or {}).get("head") or "-")[:10]
        sc = r.get("version", "-") + (
            "@" + r["scorer_commit"][:7] if r.get("scorer_commit") else ""
        )
        print(
            f"{r['name']} | {commit} | {sc} | {r['w']} | {loc} | {f(r.get('proxy'))} | {ind} | {el} / {ep} | {'; '.join(r.get('flags', []))}"
        )
    print(
        "\nACCURACY, by scorer version: only forecasts whose host and W match their independent result"
    )
    for v in sorted({r["version"] for r in rows if r.get("indep") and r.get("version") != "-"}):
        vr = [r for r in rows if r.get("version") == v and r.get("indep")]
        sp = [r for r in vr if r.get("proxy_scored")]
        a = accuracy(sp, "proxy")
        print(
            f"  {v}: proxy against the i9: {a['n']} cells, mean |log err| {f(a['male'])}, ranks {a['right']}/{a['pairs']}"
            + ("  (no i9 result yet)" if not sp else "")
        )
        for key, what in (("local", "local wall"), ("proxy", "proxy")):
            cross = [r for r in vr if r.get(key) is not None and not r.get("proxy_scored")]
            c = accuracy(cross, key)
            if cross:
                print(
                    f"  {v}: {what} against other hosts (NOT calibration: host differs): {c['n']} cells,"
                    f" mean |log err| {f(c['male'])}, ranks {c['right']}/{c['pairs']}"
                )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
