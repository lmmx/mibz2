"""Fresh native memory and ordering cost. The native program reads original runs only."""
import subprocess
import sys
from pathlib import Path
exe, panel = map(Path,sys.argv[1:])
print("ordinal,h,accepted,high_water_kib,forward_key_comparisons,final_tag_sort_comparisons,mask_sort_comparisons,states,lf_pieces")
for i in range(0,217,24):
    for h in (0,3,4,6,8,10,11,12):
        row=subprocess.check_output([exe,panel,str(i),str(h)]).decode().strip()
        print(row)
