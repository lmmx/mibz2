"""Verify the full 960-run receipt against v1 and compare with its deposited reference arms."""
import csv
import hashlib
import json
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]


def main():
    root = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else ROOT
    meta = json.loads((HERE / "checkpoint/provenance.json").read_text())
    for path in ("bench/moonshot/moonshot.py", "bench/moonshot/calibration.json"):
        if hashlib.sha256((root / path).read_bytes()).hexdigest() != meta["source"][path]:
            raise ValueError("supply a checkout with the historical v1 scorer and calibration")
    sys.path.insert(0, str(root / "bench/moonshot"))
    import moonshot

    save = HERE / "score"
    cal = json.loads(moonshot.CALIBRATION.read_text())
    raw = [json.loads(line) for line in (save / "proxy-runs.jsonl").read_text().splitlines()]
    sample = moonshot.proxy_sample(cal, "48+16")
    model = cal["model"]
    wanted = {(w, arm, phase, b) for w in model["workers"] for arm in ("lazy", "candidate")
              for phase, ids in (("base", sample["full"] + sample["tiny"]), ("hi", sample["slope"])) for b in ids}
    keys = {(row["workers"], row["arm"], row["kind"], row["block"]) for row in raw}
    if keys != wanted or len(keys) != len(raw):
        raise ValueError("proxy receipt has missing, unexpected or duplicated cells")
    runs, _ = moonshot.load_runs(save / "proxy-runs.jsonl")
    proxy = moonshot.read_proxy(runs, sample, cal, reps=400)
    official = json.loads((save / "proxy.json").read_text())
    if proxy != official:
        raise ValueError("official per-block rebuild differs from the published interpretation")
    csvout = csv.writer(sys.stdout, lineterminator="\n")
    csvout.writerow(("W", "candidate_over_fresh_lazy", "lo95", "hi95", "deposited_opend_over_lazy",
                     "ratio_of_estimates", "candidate_over_opend_instructions", "candidate_over_opend_dram_bytes",
                     "candidate_over_opend_past_L2_links", "fresh_over_deposited_lazy_instr",
                     "fresh_over_deposited_lazy_nominal_cycles", "out_of_range"))
    for w, c in sorted(official["predicted_over_lazy"]["candidate"].items(), key=lambda p: int(p[0])):
        ca, la = (official["metrics"][arm][w] for arm in ("candidate", "lazy"))
        op, lr = (cal["reference"][w][arm] for arm in ("opend", "lazy"))
        old_pred = cal["reference_predictions"]["predicted_over_lazy"]["opend"][w]["ratio"]
        csvout.writerow((w, c["ratio"], c["lo"], c["hi"], old_pred, c["ratio"] / old_pred,
                         ca["instructions_per_byte"]/op["instructions_per_byte"],
                         ca["dram_bytes_per_byte"]/op["dram_bytes_per_byte"],
                         ca["dep_past_l2_links_per_byte"]/op["dep_past_l2_links_per_byte"],
                         la["instructions_per_byte"]/lr["instructions_per_byte"],
                         la["cycles_per_byte"]/lr["cycles_per_byte"],
                         " / ".join(official["out_of_calibration"][w])))


if __name__ == "__main__":
    main()
