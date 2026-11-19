"""Sample co-runners and their last observed CPU while the pinned standalone panel runs."""
import csv
import os
from pathlib import Path
import subprocess
import sys
import time

here=Path(__file__).resolve().parent
def sample():
    answer={}
    for p in Path("/proc").iterdir():
        if not p.name.isdigit():continue
        try:
            stat=(p/"stat").read_text()
            l,r=stat.index("("),stat.rindex(")")
            fields=stat[r+2:].split()
            name=stat[l+1:r]
            affinity=os.sched_getaffinity(int(p.name))
            if not affinity.intersection(os.sched_getaffinity(0)):continue
            answer[p.name]=(int(fields[11])+int(fields[12]),name,fields[36],fields[16],
                            ",".join(map(str,sorted(affinity))))
        except (FileNotFoundError,ProcessLookupError,PermissionError,ValueError,IndexError):continue
    return answer

out=Path(sys.argv[3])
with (out/"co-runners.csv").open("x") as f:
    w=csv.writer(f,lineterminator="\n");w.writerow(["time_s","pid","command","last_cpu","nice","cpu_ticks_delta","affinity"])
    prev=sample();start=time.monotonic()
    p=subprocess.Popen([sys.executable,str(here/"paired.py"),"run",sys.argv[1],sys.argv[2],str(out/"runs.csv")])
    while p.poll() is None:
        try:p.wait(timeout=5)
        except subprocess.TimeoutExpired:pass
        current=sample()
        for pid,(ticks,name,cpu,nice,aff) in current.items():
            old=prev.get(pid)
            if old is None:continue
            delta=ticks-old[0]
            if delta>0:w.writerow([round(time.monotonic()-start,3),pid,name,cpu,nice,delta,aff])
        f.flush();prev=current
    sys.exit(p.returncode)
