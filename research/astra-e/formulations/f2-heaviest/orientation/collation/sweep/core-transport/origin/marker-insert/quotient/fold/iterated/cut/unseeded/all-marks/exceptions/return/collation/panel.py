import csv
from pathlib import Path
import subprocess
import sys
def run(exe,wires):
    w=csv.writer(sys.stdout,lineterminator='\n')
    w.writerow(('block','k','n','rows','alphabet','runs','forward_descents','distinct_return_spelling_bytes','covered','minimal_tested_lookahead','bare_failed','k_failed','marked_prefix_steps'))
    for i in sorted(int(p.stem) for p in wires.glob('*.mask8')):
        for k in (4,8,16):
            fields=subprocess.check_output([exe,wires/f'{i}.mask{k}',str(k)],text=True).strip().split(',')
            assert fields[0]==fields[6]
            w.writerow((i,k,*fields))
if __name__=='__main__':run(*map(Path,sys.argv[1:]))
