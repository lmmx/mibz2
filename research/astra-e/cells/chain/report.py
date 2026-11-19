"""Derive a table and calibration flags from the retained one-worker event streams."""
import argparse
import csv
import json
from pathlib import Path
import sys

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("tree", type=Path, help="checkout containing the recorded scorer")
    parser.add_argument("results", type=Path)
    a = parser.parse_args()
    sys.path.insert(0, str(a.tree.resolve() / "bench/moonshot"))
    import moonshot

    writer = csv.writer(sys.stdout)
    metrics = ("instructions_per_byte", "cycles_per_byte", "l2_bytes_per_byte", "l3_bytes_per_byte",
               "dram_bytes_per_byte", "dep_dram_links_per_byte", "dep_past_l2_links_per_byte",
               "dep_loads_per_byte", "dram_mlp", "overlap_per_dep_link", "max_dram_chain",
               "max_past_l2_chain")
    writer.writerow(["implementation","input","arm",*metrics,"outside_reference_range"])
    reference_arms = ("lazy","cells","opend","open")
    for name in ("packed","hybrid","narrow"):
        for sample in ("frozen","small-large-slots"):
            arms = {}
            for arm in (*reference_arms,"candidate"):
                result = a.results/name/f"{sample}-{arm}.json"
                arms[arm] = json.loads(result.read_text())["metrics"]
            flags = moonshot.out_of_range(arms["candidate"],[arms[k] for k in reference_arms])
            for arm, data in arms.items():
                writer.writerow([name,sample,arm,*[data[k] for k in metrics],
                                 " | ".join(flags) if arm == "candidate" else ""])

if __name__ == "__main__":
    main()
