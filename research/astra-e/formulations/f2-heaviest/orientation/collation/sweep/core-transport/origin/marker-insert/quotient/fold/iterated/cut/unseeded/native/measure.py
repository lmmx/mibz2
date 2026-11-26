"""Run the frozen stage panel and dedicated one-arm RSS processes."""
import csv
from pathlib import Path
import subprocess
import sys

exe,scratch,results=map(lambda x:Path(x).resolve(),sys.argv[1:])
inputs=sorted(scratch.glob('[0-9][0-9][0-9][0-9]/input.txt'))
assert len(inputs)==10
with (results/'pairs.csv').open('x') as out:
    subprocess.run([exe,inputs[0],'panel','40',*inputs],stdout=out,check=True)
with (results/'rss.csv').open('x',newline='') as out:
    writer=csv.writer(out,lineterminator='\n');writer.writerow(['tag','arm','peak_kib'])
    for path in inputs:
        for arm in ('singleton','split-ordinal','unseeded','dense'):
            text=subprocess.check_output([exe,path,arm,'1'],text=True)
            [row]=csv.DictReader(text.splitlines())
            writer.writerow([row['tag'],row['arm'],row['peak_kib']])
