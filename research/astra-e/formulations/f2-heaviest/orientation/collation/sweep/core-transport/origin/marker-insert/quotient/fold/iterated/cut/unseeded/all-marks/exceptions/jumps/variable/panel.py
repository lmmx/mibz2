import csv
import io
from pathlib import Path
import subprocess
import sys
def main(exe,wires,baseline):
    old={(int(r['index']),int(r['k'])):r for r in csv.DictReader(baseline.open())}
    wr=csv.writer(sys.stdout,lineterminator='\n')
    wr.writerow(('index','k','threshold','n','patterns','logical','components','exceptions','chunks8','fragments','skipped_runs','overlay_steps','edges','finish_fragments','finish_edges'))
    for index in sorted({idx for idx,_ in old}):
        s=subprocess.check_output([exe,wires/f'{index}.mask8'],text=True)
        for row in csv.reader(io.StringIO(s)):
            k,t,_,_,_,components,exceptions,chunks=map(int,row[:8])
            if t==1:
                expected=old[index,k]
                assert (components,exceptions,chunks)==tuple(int(expected[q]) for q in ('states','exceptions','chunk8'))
            wr.writerow((index,*row))
if __name__=='__main__':main(*map(Path,sys.argv[1:]))
