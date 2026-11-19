"""Geometric ratio of paired ten-block CPU sums with t intervals, one pair per run."""
import csv
from collections import defaultdict
from math import exp,log,sqrt
from statistics import mean,stdev
from pathlib import Path
from scipy.stats import t
import sys

def main(path):
    rows=list(csv.DictReader(Path(path).open()))
    for sec in ('quote','frequent'):
        by=defaultdict(lambda:defaultdict(float))
        for r in rows:
            if r['section']==sec:by[r['arm']][int(r['rep'])]+=float(r['total_s'])
        def pair(a,b):
            reps=by[a];logs=[log(reps[k]/by[b][k]) for k in reps];m=mean(logs)
            d=stdev(logs)*t.ppf(.975,len(logs)-1)/sqrt(len(logs))
            print(f'{sec} {a}/{b} {exp(m):.4f} [{exp(m-d):.4f}, {exp(m+d):.4f}]')
        for a in by:
            if a!='compact':pair(a,'compact')
        if 'scatter' in by and 'sort' in by:pair('scatter','sort')
        if 'old' in by:pair('scatter','old')
        if 'rank' in by:pair('rank','scatter')
if __name__=='__main__': main(sys.argv[1])
