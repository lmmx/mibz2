"""Record the instructions implementing rank in the real measured release executable."""
import re,subprocess,sys
from collections import Counter
binary=sys.argv[1]
nm=subprocess.check_output(['nm','-nS','-C',binary],text=True)
for line in nm.splitlines():
    m=re.match(r'^([0-9a-f]+) ([0-9a-f]+) [tT] (.+)$',line)
    if not m: continue
    name=m[3]
    if not (name=='<price::counted::Candidate>::build' or
            ('price::dispatch::Dispatch<' in name and name.rsplit('::',1)[-1] in ('build_pop','build_bmi','prepare_pop','prepare_counted'))):
        continue
    begin=int(m[1],16)
    end=begin+int(m[2],16)
    data=subprocess.check_output(['objdump','-d','-M','intel','--no-show-raw-insn',f'--start-address={begin}',f'--stop-address={end}',binary],text=True)
    ins=[]
    for row in data.splitlines():
        op=re.search(r'^\s*[a-f0-9]+:\s+(\S+)',row)
        if op and (op[1] in ('popcnt','bzhi','shrx','shlx') or any(x in row for x in ['0x5555555555555555','0x3333333333333333','0xf0f0f0f0f0f0f0f'])):
            ins.append((op[1],row.strip()))
    print(f'{name} at 0x{begin:x} bytes={end-begin} static matching instructions={dict(Counter(op for op,_ in ins))}')
    for _,row in ins: print(' ',row)
