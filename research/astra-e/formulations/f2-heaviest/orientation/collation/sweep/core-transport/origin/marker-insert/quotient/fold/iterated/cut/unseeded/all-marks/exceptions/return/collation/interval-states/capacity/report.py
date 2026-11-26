import csv
from pathlib import Path
from collections import defaultdict
import sys

def report(folder):
    with (folder/"bounds.csv").open() as f:
        bounds = list(csv.DictReader(f))
    with (folder/"dual.csv").open() as f:
        dual = list(csv.DictReader(f))
    # The λ=0 objective is n at every horizon.
    sizes = {r["block"]: int(r["relaxed_objective"]) for r in dual if r["h"]=="1" and r["penalty"]=="0"}
    total = sum(sizes.values())
    d = defaultdict(dict)
    for row in dual:
        h, cost = int(row["h"]), int(row["penalty"])
        d[h,cost][row["block"]] = int(row["relaxed_objective"])
    curves = [(h,c,sum(per.values())) for (h,c),per in d.items() if len(per)==len(sizes)]
    writer = csv.writer(sys.stdout,lineterminator="\n")
    writer.writerow(("policy","total_states","exceptions","per_block_floor","shared_budget_floor","shared_h","shared_charge"))
    for policy in dict.fromkeys(r["policy"] for r in bounds):
        subset = [r for r in bounds if r["policy"] == policy]
        first = [r for r in subset if r["h"]=="1"]
        states = sum(int(r["states"]) for r in first)
        errors = total-sum(int(r["actual_all_default"]) for r in first)
        bound = sum(max(int(r["exception_floor"]) for r in subset if r["block"]==b) for b in sizes)
        strength,h,cost = max(((total-v-c*(states-len(sizes))+h-1)//h,h,c) for h,c,v in curves)
        writer.writerow((policy,states,errors,bound,strength,h,cost))
    target = total-sum(int(r["actual_all_default"]) for r in bounds if r["h"]=="1" and r["policy"]=="k8")
    minimum,cost,h = max((len(sizes)+(total-h*target-v+c-1)//c,c,h) for h,c,v in curves if c>0)
    print(f"states_required_for_{target}_exceptions,{minimum},charge,{cost},h,{h}")

if __name__=="__main__":
    report(Path(sys.argv[1]))
