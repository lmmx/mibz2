"""One fresh native process per block and layout, excluding the dense verifier."""
from pathlib import Path
import subprocess
import sys
exe, panel, out = map(Path,sys.argv[1:])
for hull,name in ((False,"memory.csv"),(True,"hull-memory.csv")):
    with (out/name).open("w") as f:
        print("ordinal,h,fused,high_water_kib,output_bytes",file=f)
        for i in range(0,217,24):
            for h in ((0,3) if hull else range(4)):
                args=[exe,panel,str(i),str(h),str(int(h>0))]
                if hull:args.append("hull")
                f.write(subprocess.check_output(args).decode())
