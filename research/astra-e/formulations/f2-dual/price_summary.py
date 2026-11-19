"""Report phase medians per input block, then sum equally across the sampled admissions."""
import csv
from collections import defaultdict
from pathlib import Path
from statistics import median

study = Path(__file__).resolve().parent
records = list(csv.DictReader((study/"phase.csv").open()))
phases = ("prep_mark_ns","discover_ns","merge_ns","residual_output_ns","total_ns")
groups=defaultdict(list)
for r in records:
    groups[r["stream"],r["arm"]].append(r)
assert len(groups)==9*4 and all(len(group)==20 for group in groups.values())
for tag in ("1","all-nine"):
    output={}
    for arm in ("cache","plain","trie","lazy"):
        output[arm]=[sum(median(int(row[k]) for row in group)
                         for (stream,a),group in groups.items()
                         if a==arm and (tag=="all-nine" or stream==tag))/1e6 for k in phases]
    for arm,values in output.items():
        print(tag,arm,*[f"{k}={v:.4f} ms" for k,v in zip(phases,values)],f"vs_lazy={values[-1]/output['lazy'][-1]:.4f}")
