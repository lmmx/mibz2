"""Collect each audit/control child process's actual Linux maximum resident set."""
from pathlib import Path
import json
import os
import subprocess
import sys
exe,corpus,out=map(Path,sys.argv[1:4])
with out.open('x',buffering=1) as f:
    for r in range(6):
        for a in (['base','audit'] if r%2==0 else ['audit','base']):
            with subprocess.Popen([exe,a,corpus],stdout=subprocess.PIPE) as proc:
                pid,status,usage=os.wait4(proc.pid,0)
                proc.returncode=os.waitstatus_to_exitcode(status)
                assert pid==proc.pid and proc.returncode==0
                z=json.load(proc.stdout)
            assert z['bytes']==209700196 and z['count']==(5 if a=='audit' else 0)
            print(json.dumps(dict(round=r,label=a,peak_rss_kib=usage.ru_maxrss,**z)),file=f)
