"""Aggregate the frozen eight paired sums of ten systematic blocks per section."""
import csv
import math
from pathlib import Path
from statistics import mean, stdev
import sys
from collections import defaultdict
from scipy.stats import t

def summary(path):
    rows=list(csv.DictReader((path/"paired.csv").open()))
    hwm=list(csv.DictReader((path/"fresh.csv").open()))
    for name in "quote","frequent":
        sums=defaultdict(lambda:defaultdict(float))
        for x in rows:
            if x["section"]!=name:continue
            for field in "total_cpu_s","indexed_emit_cpu_s","construct_cpu_s":
                sums[x["arm"],field][int(x["rep"])]+=float(x[field])
        if ("paged","total_cpu_s") in sums:
            tests=[("paged","total_cpu_s","tree"),("paged","total_cpu_s","direct"),("paged","total_cpu_s","compact"),
                   ("paged","indexed_emit_cpu_s","direct"),("paged","indexed_emit_cpu_s","compact")]
        else:
            tests=[(a,"total_cpu_s",b) for a in ("free_geometry","free_order") for b in ("direct","compact")]
        for a,f,b in tests:
            x=sums[a,f];y=sums[b,"total_cpu_s"]
            logs=[math.log(x[k]/y[k]) for k in sorted(x)]
            m=mean(logs);e=t.ppf(.975,len(logs)-1)*stdev(logs)/math.sqrt(len(logs))
            print(f"{name} {a}.{f}/{b}: {math.exp(m):.4f} [{math.exp(m-e):.4f}, {math.exp(m+e):.4f}]")
        for arm in sorted({x["arm"] for x in hwm}):
            r=[int(x["process_hwm_kib"]) for x in hwm if x["section"]==name and x["arm"]==arm]
            print(f"{name} {arm} fresh-process VmHWM KiB {min(r)}..{max(r)}")
if __name__=="__main__":summary(Path(sys.argv[1]))
