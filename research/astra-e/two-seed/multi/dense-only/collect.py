"""Run true compressed-stream CPU-charge observations, alternating the label order."""
import json
from pathlib import Path
import subprocess
import sys
exe,corpus,dest=(Path(v).resolve() for v in sys.argv[1:4])
with dest.open('x',buffering=1) as f:
    for i in range(10):
        for label in (['base','audit'] if i%2==0 else ['audit','base']):
            z=json.loads(subprocess.check_output([exe,label,corpus]))
            assert z['bytes']==209700196 and z['count']==(5 if label=='audit' else 0)
            print(json.dumps(dict(round=i,label=label,**z)),file=f)
