"""Measure each fresh single-worker process after decoding all original blocks with CRCs."""
import csv
import os
from pathlib import Path
import sys
executable=Path(sys.argv[1]).resolve()
corpus=Path(sys.argv[2]).resolve()
labels=[0,2,3]
writer=csv.writer(sys.stdout,lineterminator='\n')
writer.writerow(['round','position','arm','max_rss_kib'])
with open(os.devnull,'w') as sink:
    for i in range(12):
        for pos in range(3):
            a=labels[(i+(pos if i//3%2==0 else -pos))%3]
            args=[str(executable),'memory',str(corpus),str(a)]
            pid=os.posix_spawn(str(executable),args,os.environ,file_actions=[(os.POSIX_SPAWN_DUP2,sink.fileno(),1)])
            _,status,usage=os.wait4(pid,0)
            assert os.waitstatus_to_exitcode(status)==0
            writer.writerow([i+1,pos,a,usage.ru_maxrss])
            sys.stdout.flush()
