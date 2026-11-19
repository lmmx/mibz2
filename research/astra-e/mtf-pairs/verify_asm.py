"""Compare normalized compiled control bodies between the first and second saved panels."""
import hashlib
import re
import subprocess
import sys

def body(binary,arm):
    cmd=["objdump","-dC","--no-show-raw-insn",binary]
    asm=subprocess.check_output(cmd,text=True)
    start=asm.index("<main::"+arm+"::front_avx2>:")
    lines=asm[start:asm.index("\n\n",start)].splitlines()[1:]
    instructions=[]
    for line in lines:
        op=line.split(":",1)[1].strip().split("#")[0]
        op=re.sub(r"\b[0-9a-f]+ <(.+)>",r"<\1>",op)
        op=re.sub(r"[-+]?0x[0-9a-f]+\(%rip\)","rip",op)
        instructions.append(op)
    return instructions
for arm in ["stock","pair"]:
    a,b=[body(p,arm) for p in sys.argv[1:]]
    assert a==b
    print(arm,len(a),"normalized instructions including padding and labelled targets",hashlib.sha256("\n".join(a).encode()).hexdigest())
