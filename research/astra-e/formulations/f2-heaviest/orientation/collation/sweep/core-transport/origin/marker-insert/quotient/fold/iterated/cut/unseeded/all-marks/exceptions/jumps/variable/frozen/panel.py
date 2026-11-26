import csv
from pathlib import Path
import subprocess
import sys
def run(exe,wires):
    idx=sorted(int(v.stem) for v in wires.glob('*.mask8'))
    assert len(idx)==10 and idx[0]==1
    wr=csv.writer(sys.stdout,lineterminator='\n')
    wr.writerow(('index','threshold','n','dictionary','active','components','exceptions','chunks8','rle_runs','rank_queries','rank_comparisons','overlay_steps','finish_fragments','finish_edges'))
    for i in idx:
        s=subprocess.check_output([exe,wires/'1.mask8',wires/f'{i}.mask8'],text=True)
        wr.writerows((i,*r.split(',')) for r in s.splitlines())
if __name__=='__main__':run(*map(Path,sys.argv[1:]))
