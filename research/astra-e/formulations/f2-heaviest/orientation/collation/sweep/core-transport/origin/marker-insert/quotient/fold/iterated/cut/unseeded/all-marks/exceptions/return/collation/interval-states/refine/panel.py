import csv
from pathlib import Path
import subprocess
import sys
def run(exe,wires):
    wr=csv.writer(sys.stdout,lineterminator='\n')
    wr.writerow(('block','penalty','round','n','states','correct_default','jump8_bytes','atoms','safe_endpoints','dp_fragments','dp_labels','dp_saved_starts','splits','new_correct_default'))
    for i in sorted(int(p.stem) for p in wires.glob('*.mask8')):
        s=subprocess.check_output([exe,wires/f'{i}.mask8'],text=True)
        wr.writerows((i,*line.split(',')) for line in s.splitlines())
if __name__=='__main__':run(*map(Path,sys.argv[1:]))
