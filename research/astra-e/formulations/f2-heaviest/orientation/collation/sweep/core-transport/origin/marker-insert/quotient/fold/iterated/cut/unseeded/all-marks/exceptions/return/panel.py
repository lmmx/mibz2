"""Run the first-return constructor on physical-only inputs and archive all dimensions."""
import argparse
import csv
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile

HERE=Path(__file__).resolve().parent
FIELDS='coalesced n runs anchors roots raw_atoms atoms hops visited_width singleton_hops splits run_splits preimage_splits leaves return_runs map_lookups map_comparisons max_stack marked_visits exceptions marked_lap clock_comparisons overlay_heap_bytes walk_heap_bytes'.split()
def main():
    p=argparse.ArgumentParser()
    p.add_argument('work',type=Path)
    p.add_argument('masks',type=Path)
    p.add_argument('--tapes',type=Path)
    p.add_argument('--positions',type=Path)
    a=p.parse_args()
    a.work.mkdir(parents=True,exist_ok=True)
    exe=a.work/'induce'
    subprocess.run(['g++','-std=c++17','-O2','-Wall','-Wextra',HERE/'induce.cpp','-o',exe],check=True)
    with (a.work/'exhaustive.txt').open('x') as out:subprocess.run([exe],stdout=out,check=True)
    match={(int(r['index']),int(r['k'])):r for r in csv.DictReader((HERE.parent/'panel/blocks.csv').open()) if int(r['k']) in (4,8,16)}
    assert len(match)==30
    if any(not (a.masks/f'{i}.{prefix}{k}').is_file() for i,k in match for prefix in ('pred','mask')):
        if a.tapes is None or a.positions is None:
            p.error('rebuilding masks requires --tapes (the ten original .irec.xz) and --positions (may be an empty directory)')
        generator=a.work/'wire-generator'
        subprocess.run(['g++','-std=c++17','-O3',HERE.parent/'panel/census.cpp','-o',generator],check=True)
        with (a.work/'wire-blocks.csv').open('x') as out:
            subprocess.run([sys.executable,'-B',HERE.parent/'panel/panel.py',generator,a.tapes,a.positions,
                a.work/'wire-depths.csv',a.masks],stdout=out,check=True)
    digests=[]
    with (a.work/'blocks.csv').open('x') as out:
        writer=csv.writer(out,lineterminator='\n');writer.writerow(['index','k',*FIELDS])
        for i,k in sorted(match):
            inp=a.masks/f'{i}.mask{k}';oracle=a.masks/f'{i}.pred{k}'
            raw=subprocess.check_output([exe,inp,oracle],text=True)
            for mode,s in enumerate(raw.splitlines()):
                v=list(map(int,s.split(',')));d=dict(zip(FIELDS,v,strict=True))
                assert d['coalesced']==mode and d['n']==int(match[i,k]['n']) and d['exceptions']==int(match[i,k]['exceptions'])
                assert d['n']==d['visited_width']==d['marked_lap']
                writer.writerow([i,k,*v])
            digests.append(dict(index=i,k=k,mask=hashlib.sha256(inp.read_bytes()).hexdigest(),pred=hashlib.sha256(oracle.read_bytes()).hexdigest()))
    (a.work/'inputs.json').write_text(json.dumps(digests,indent=2)+'\n')

if __name__=='__main__':main()
