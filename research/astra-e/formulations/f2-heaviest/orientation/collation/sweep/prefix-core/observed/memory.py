from pathlib import Path
import subprocess
import sys
exe,panel=map(Path,sys.argv[1:])
print("ordinal,h,layout,peak_kib,compact_capacity_bytes,gap_capacity_bytes")
for i in range(0,217,24):
    for h,mode in [(-1,2)]+[(x,m) for x in (0,3,8,10) for m in (0,1)]:
        print(subprocess.check_output([exe,panel,str(i),str(h),str(mode)]).decode().strip())
