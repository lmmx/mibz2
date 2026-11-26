"""Price independent construction through each chosen depth, then separately measure post-exec RSS."""
import csv
import subprocess
import sys
from pathlib import Path
exe,storage=map(lambda x:Path(x).resolve(),sys.argv[1:])
table=list(csv.DictReader((storage/'blocks.csv').open()))
indices=list(dict.fromkeys(int(r['index']) for r in table));assert len(indices)==10
inputs=[storage/'raw'/(str(i)+'.txt') for i in indices]
gate=storage/'table.txt'
gate.write_text(''.join(' '.join(r[f] for f in ('states','edges','exception_rows','descriptors'))+'\n' for r in table))
with (storage/'times.csv').open('x') as f:
    subprocess.run([exe,'panel','12',gate,*inputs],stdout=f,check=True)
with (storage/'rss.csv').open('x',newline='') as f:
    w=csv.writer(f,lineterminator='\n');w.writerow(('index','k','peak_kib'))
    for r in table:
        i=int(r['index']);k=r['k']
        peak,c,e=subprocess.check_output([exe,'rss',storage/'raw'/(str(i)+'.txt'),k],text=True).strip().split(',')
        assert (c,e)==(r['states'],r['exception_rows'])
        w.writerow((i,k,peak))
