import csv
from pathlib import Path
import subprocess
import sys
def run(exe,wires,table):
    baseline={(int(r['block']),int(r['k'])):r for r in csv.DictReader(table.open())}
    wr=csv.writer(sys.stdout,lineterminator='\n')
    wr.writerow(('block','first_k','later_k','depth','k','n','types','contexts','events','classes','incoming_runs','marked_phase','return_steps','semantic_lap','expanded_symbols','terminal'))
    for i,k in sorted(baseline):
        for later in (1,2):
            raw=subprocess.check_output([exe,wires/f'{i}.mask{k}',str(k),str(later)],text=True)
            rows=[list(map(int,s.split(','))) for s in raw.splitlines()]
            base=baseline[i,k]
            assert [int(base[q]) for q in ('rows','alphabet','runs')]==rows[0][5:8]
            assert rows[-1][-1]==1
            for row in rows:wr.writerow((i,k,later,*row))
if __name__=='__main__':run(*map(Path,sys.argv[1:]))
