"""The target-gated noninferiority workflow's verdict gate (.github/workflows/target-gated-ni.yml,
its `aggregate` job's last step; the Guardian's D078): the job's status follows the judge's
verdict, after the evidence is uploaded.

    python3 bench/tg_ni_gate.py --label LABEL --verdict ni.json --exit JUDGE_EXIT [--summary FILE]

- **qualification** (one dispatch of the replicated programme, registration §2.11): never a
  verdict. Passes (exit 0) only if the judge (`service_tg.py ni`) exited 0 and its dispatch
  verdict is admitted; the summary says it is one of k and not a verdict. INVALID (exit 2), a
  missing or unreadable verdict, or any disagreement between the exit and the verdict fails.
- **replicated** (the programme's verdict, `service_tg.py replicated`, offline after all k):
  passes only if the judge exited 0, its verdict is valid and passes, and this gate's own
  recomputation agrees. The gate recomputes, apart from the judge: exactly k = 12 distinct
  integer run ids; for each of the 42 (21 cells × wall, CPU) exactly 12 finite dispatch
  estimates; U = exp(mean + t(0.95, 11) · sd / sqrt(12)) ≤ 1.02, t(0.95, 11) = 1.7958848187.
  Its failing set must equal the judge's. NOT SHOWN (exit 1) and INVALID (exit 2) fail.
- **smoke** and **pilot** (the registration's §2.8 design pilot): never evidence. The judge must have refused it as smoke: its
  verdict present, not valid, and among its problems the label's refusal. And the run must
  pass the structural admission (`service_tg.py structure`, `--structure`): every shard present,
  its artifacts listed and downloaded, the manifest valid, the history reconciled, the files
  whole. Only then is it success (exit 0), and the summary says "smoke: not evidence" (or
  "pilot: not evidence"). A refused label with nothing behind it fails (exit 1). The pilot's own analysis is bench/tg_ni_pilot.py's, offline.
- Any other label fails.

It writes one line to `--summary` (the step summary) either way.
"""

import argparse
import json
import math
import sys

# the registered programme (§2.11), restated here so that the bound is computed twice, apart
CELLS = [f"{c}/{t}" for c in ("onebyte", "pattern", "random", "records", "runs", "text", "wikidata") for t in (1, 2, 4)]
METRICS = ("wall", "cpu")
K = 12
T95 = 1.7958848187  # t(0.95, K - 1)
MARGIN = 1.02


def recompute(verdict):
    """(failing as 'cell metric', problems): the programme's bound from the verdict's own
    dispatch estimates"""
    problems, failing = [], []
    runs = verdict.get("runs")
    if not (isinstance(runs, list) and len(runs) == K and all(isinstance(r, int) and not isinstance(r, bool) for r in runs)
            and len(set(runs)) == K):
        problems.append(f"not {K} distinct run ids: {runs!r}")
    cells = verdict.get("cells") if isinstance(verdict.get("cells"), dict) else {}
    for key in CELLS:
        for metric in METRICS:
            cell = cells.get(key)
            st = cell.get(metric) if isinstance(cell, dict) else None
            ds = st.get("d") if isinstance(st, dict) else None
            if not (isinstance(ds, list) and len(ds) == K and all(
                    isinstance(x, (int, float)) and not isinstance(x, bool) and math.isfinite(x) for x in ds)):
                problems.append(f"{key} {metric}: not {K} finite dispatch estimates")
                continue
            m = sum(ds) / K
            sd = math.sqrt(sum((x - m) ** 2 for x in ds) / (K - 1))
            if math.exp(m + T95 * sd / math.sqrt(K)) > MARGIN:
                failing.append(f"{key} {metric}")
    return failing, problems


def gate(label, verdict, judge_exit, structure=None):
    """(exit status, one-line summary)"""
    if label in ("smoke", "pilot"):
        if not isinstance(verdict, dict):
            return 1, f"{label}: not evidence; FAILED: the judge wrote no verdict (the plumbing is broken)"
        if verdict.get("valid"):
            return 1, f"{label}: not evidence; FAILED: the judge admitted a {label} run"
        if not any(f"labelled '{label}'" in str(p) for p in verdict.get("problems", [])):
            return 1, f"{label}: not evidence; FAILED: the judge did not refuse it as {label}"
        if judge_exit != 2:
            return 1, f"{label}: not evidence; FAILED: the judge exited {judge_exit}, not 2 (refused)"
        # (the refusal alone shows only that the label is refused: the plumbing is shown by the
        # structural admission, every shard present, listed, downloaded, reconciled and whole)
        if not isinstance(structure, dict) or structure.get("label") != label:
            return 1, f"{label}: not evidence; FAILED: no structural admission for a {label} run"
        if not structure.get("valid"):
            why = "; ".join(map(str, structure.get("problems", [])[:3])) or "not admitted"
            return 1, f"{label}: not evidence; FAILED: the structural admission failed: {why}"
        return 0, (f"{label}: not evidence (the judge refused it as a {label} run, as it must); "
                   f"structurally admitted: all {len(structure.get('shards', []))} shards, R {structure.get('rounds')}")
    if label == "qualification":
        if not isinstance(verdict, dict):
            return 1, "qualification dispatch: FAILED: no readable verdict (INVALID)"
        valid = bool(verdict.get("valid"))
        if verdict.get("schema") != "mibz2.tg_ni_dispatch/1":
            return 1, f"qualification dispatch: FAILED: not a dispatch verdict ({verdict.get('schema')!r})"
        if judge_exit == 0 and valid:
            return 0, (f"qualification dispatch admitted (run {verdict.get('run')}): one of {K}, not a verdict; "
                       "condition 2 is judged only from all of them (service_tg.py replicated)")
        if judge_exit == 2 and not valid:
            return 1, "qualification dispatch: INVALID (the programme fails): " + "; ".join(map(str, verdict.get("problems", [])[:5]))
        return 1, f"qualification dispatch: FAILED: the judge's exit {judge_exit} disagrees with its verdict (valid {valid})"
    if label == "replicated":
        if not isinstance(verdict, dict):
            return 1, "replicated: FAILED: no readable verdict (INVALID)"
        valid, passes = bool(verdict.get("valid")), bool(verdict.get("passes"))
        if judge_exit == 2 and not valid:
            return 1, "replicated: INVALID: " + "; ".join(map(str, verdict.get("problems", [])[:5]))
        if not valid:
            return 1, f"replicated: FAILED: the judge's exit {judge_exit} disagrees with its verdict (valid False)"
        failing, problems = recompute(verdict)
        if problems:
            return 1, "replicated: INVALID: the verdict lacks what the bound needs: " + "; ".join(problems[:3])
        claimed = sorted(" ".join(str(x).split()[:2]) for x in verdict.get("failing") or [])
        if sorted(failing) != claimed:
            return 1, (f"replicated: FAILED: the gate's bound (failing {', '.join(failing) or 'none'}) disagrees "
                       f"with the judge's ({', '.join(claimed) or 'none'})")
        if judge_exit == 0 and passes and not failing:
            return 0, f"replicated: condition 2 met at every cell over {K} dispatches (PASS)"
        if judge_exit == 1 and not passes:
            why = "; ".join(verdict.get("failing", []) + [f"RSS {c}" for c in verdict.get("rss_regression", [])])
            return 1, f"replicated: condition 2 NOT SHOWN: {why}"
        return 1, f"replicated: FAILED: the judge's exit {judge_exit} disagrees with its verdict (passes {passes})"
    return 1, f"FAILED: unknown label {label!r}"


def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    p.add_argument("--label", required=True)
    p.add_argument("--verdict", required=True)
    p.add_argument("--exit", type=int, required=True)
    p.add_argument("--structure", help="smoke and pilot: service_tg.py structure's JSON")
    p.add_argument("--summary")
    a = p.parse_args(argv)
    try:
        with open(a.verdict) as f:
            verdict = json.load(f)
    except (OSError, json.JSONDecodeError):
        verdict = None
    structure = None
    if a.structure:
        try:
            with open(a.structure) as f:
                structure = json.load(f)
        except (OSError, json.JSONDecodeError):
            structure = None
    status, line = gate(a.label, verdict, a.exit, structure)
    print(line)
    if a.summary:
        with open(a.summary, "a") as f:
            f.write(f"\n**{line}**\n")
    return status


if __name__ == "__main__":
    sys.exit(main())
