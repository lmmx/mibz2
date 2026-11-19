import csv
from pathlib import Path
import subprocess
import sys
def run(exe,wires):
    w=csv.writer(sys.stdout,lineterminator='\n')
    w.writerow(('index','k','n','states','smooth_types','smooth_stretches','minimum_cyclic_light_crossings',
                'seam_adjustment','memcpy_segments','max_segments','naive_dictionary_bytes','max_path'))
    for i in sorted(int(p.stem) for p in wires.glob('*.pred8')):
        for k in (4,8,16):
            r=subprocess.check_output([exe,wires/f'{i}.pred{k}'],text=True)
            w.writerow((i,k,*r.strip().split(',')))
if __name__=='__main__':run(*map(Path,sys.argv[1:]))
