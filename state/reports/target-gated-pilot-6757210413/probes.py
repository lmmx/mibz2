"""Independent synthetic probes of PR319 f5e3f0ad; no decoder timing."""
import copy
import json
import statistics
import tempfile
from pathlib import Path
import service_tg as tg
import test_service_tg as T
import tg_ni_pilot as pilot
import tg_ni_gate as gate

T.R = 600
out = {}
with tempfile.TemporaryDirectory() as root:
    man = T.build(root, label="pilot")
    def analyse(m=man):
        return pilot.analyse(root, m, T.HEAD, T.BASESHA,
                             T.REG["frozen"]["ni_rustc"], T.REG["frozen"]["corpus_text"], rounds=600)
    v = analyse()
    out["missing_timestamps"] = {"valid": v["valid"], "problems": v["problems"],
        "runs_shard": v["shards"]["runs"]}
    assert v["valid"] and v["shards"]["runs"]["timestamps"] == 0
    out["batch_menu_at_600"] = [b["m"] for b in v["cells"]["runs/1/wall"]["batches"]]
    assert out["batch_menu_at_600"] == [10, 20, 30, 60]
    a = next(a for a in man["artifacts"] if a["shard"] == "runs")
    path = Path(root) / a["name"] / "runs" / "runs.jsonl"
    original = path.read_text()
    rows = [json.loads(l) for l in original.splitlines()]
    rows[0]["wall"] *= 1.01
    path.write_text("".join(json.dumps(r) + "\n" for r in rows))
    assert tg.sha256_file(path) != a["files"]["runs.jsonl"]
    v = analyse()
    out["changed_rows_after_manifest"] = {"digest_mismatch": True, "valid": v["valid"], "problems": v["problems"]}
    assert v["valid"]
    path.write_text(original)
    bad = copy.deepcopy(man)
    bad["unlisted"] = ["tg-ni-surprise-a1"]
    v = analyse(bad)
    out["unlisted_artifact"] = {"valid": v["valid"], "problems": v["problems"]}
    assert v["valid"]
    # One timestamp per round, strictly backwards: partial and physically impossible.
    times_path = path.parent / "row-times.txt"
    times_path.write_text("".join(f"{6*r} {10000-r}\n" for r in range(600)))
    v = analyse()
    out["partial_backwards_timestamps"] = {"valid": v["valid"], "shard": v["shards"]["runs"]}
    assert v["valid"] and v["shards"]["runs"]["round_s"]["median"] == -1
    for label in ("smoke", "pilot"):
        refused = tg.ni_judge(root, {"label": label, "artifacts": []}, T.HEAD,
                              reg={"frozen": {}, "bench": None}, check_git=False)
        status, line = gate.gate(label, refused, 2)
        out[f"empty_{label}_gate"] = {"status": status, "summary": line,
                                      "problems": refused["problems"]}
        assert status == 0

# Equal-length contiguous batches; every batch has one slow round. No stochastic assumptions.
rt = ([1] * 29 + [100]) * 20
out["duration_proxy_counterexample"] = {
    "m": 30, "reported_m_times_median_round": 30 * statistics.median(rt),
    "actual_median_batch_sum": statistics.median([sum(rt[i:i+30]) for i in range(0,600,30)])}
print(json.dumps(out, indent=2))
