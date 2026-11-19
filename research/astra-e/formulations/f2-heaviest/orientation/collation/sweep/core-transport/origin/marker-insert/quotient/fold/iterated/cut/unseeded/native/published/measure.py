"""Balanced four-arm study of the identical unseeded trajectories."""
import csv
from pathlib import Path
import subprocess
import sys
exe,scratch,results=map(lambda x:Path(x).resolve(),sys.argv[1:])
inputs=sorted(scratch.glob('[0-9][0-9][0-9][0-9]/input.txt'))
assert len(inputs)==10
with (results/'pairs.csv').open('x') as f:
    subprocess.run([exe,inputs[0],'panel','40',*inputs],stdout=f,check=True)
with (results/'rss.csv').open('x',newline='') as f:
    w=csv.writer(f,lineterminator='\n');w.writerow(['tag','arm','peak_kib'])
    for src in inputs:
        for arm in ('histogram','prefix','delayed','dense'):
            [row]=csv.DictReader(subprocess.check_output([exe,src,arm,'1'],text=True).splitlines())
            w.writerow([row['tag'],row['arm'],row['peak_kib']])
