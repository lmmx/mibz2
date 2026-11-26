import csv
from pathlib import Path
import subprocess
import sys
def run(exe,wires):
    wr=csv.writer(sys.stdout,lineterminator='\n')
    wr.writerow(('block','width','n','states','original_runs','atoms','edges','exceptions','classes','reduced_runs','jump8_bytes','covered','phase'))
    for i in sorted(int(p.stem) for p in wires.glob('*.mask8')):
        s=subprocess.check_output([exe,wires/f'{i}.mask8'],text=True)
        wr.writerows((i,*r.split(',')) for r in s.splitlines())
if __name__=='__main__':run(*map(Path,sys.argv[1:]))
