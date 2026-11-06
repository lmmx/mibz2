#!/usr/bin/env python3
"""Compare archived listings modulo relocation and diagnostic-data addresses."""
import collections, json, pathlib, re
from replay import EV, ROOT, listing
B2 = EV / 'b2'

def normalize(code):
    base=min(code)
    output=[]
    for addr, item in code.items():
        text=item['asm'].replace('ctor2s','ctor2').replace('Ctor2s','Ctor2')
        text=text.split('#')[0].rstrip()
        text=re.sub(r'0x[0-9a-f]+\(%rip\)', '<reloc>(%rip)',text)
        text=text.replace('-<reloc>','<reloc>')
        text=re.sub(r'\b[0-9a-f]+ (?=<)', '<target> ',text)
        output.append(f'{addr-base:x}: {text}')
    return output

result={}
for old,new in [('ctor2-build-false.asm','v2-build-false-same-binary.asm'),('ctor2-finish-false.asm','ctor2s-finish-false.asm'),('ctor2-refill-false.asm','ctor2s-refill-false.asm')]:
    a,b=listing(B2/old),listing(EV/new)
    result[old]={'b2_count':len(a),'scanner_archive_count':len(b),'normalized_identical':normalize(a)==normalize(b)}
    assert normalize(a)==normalize(b),(old,new)
a,b=listing(EV/'attempt-3-ctor2s-build-false.asm'),listing(EV/'ctor2s-build-false.asm')
result['attempt3_final']={'normalized_identical':normalize(a)==normalize(b),'attempt_start':hex(min(a)),'final_start':hex(min(b))}
assert normalize(a)==normalize(b)
for n in (1,2,3):
    code=listing(EV/f'attempt-{n}-ctor2s-build-false.asm')
    result[f'attempt{n}']={'instruction_count':len(code),'start':hex(min(code))}
(ROOT/'asm-comparison.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(result,indent=2))
