import csv
from pathlib import Path
import subprocess
import sys
def run(exe,wires):
    w=csv.writer(sys.stdout,lineterminator='\n')
    w.writerow(('index','k','n','contexts','branch_types','branch_visits','smooth_groups','max_smooth','smooth8_bytes','modal8_jumps','all_smooth_modal8','old_dispatches'))
    ids=sorted(int(p.stem) for p in wires.glob('*.pred8'))
    assert len(ids)==10
    for i in ids:
        for k in (4,8,16):
            data=subprocess.check_output([exe,wires/f'{i}.pred{k}'],text=True)
            w.writerow((i,k,*data.strip().split(',')))
if __name__=='__main__':run(*map(Path,sys.argv[1:]))
