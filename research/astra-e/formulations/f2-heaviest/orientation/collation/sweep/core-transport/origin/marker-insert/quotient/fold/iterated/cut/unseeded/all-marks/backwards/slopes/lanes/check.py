from array import array
import csv
import json
from pathlib import Path
import struct
import subprocess
import sys
sys.path.insert(0,str(Path(__file__).resolve().parents[2]))
from encode import real


def price(last, mark, exe):
    n=len(last)
    p=sorted(range(n),key=last.__getitem__)
    loc=array('I',[n])*n
    x=mark
    for i in range(n):
        assert loc[x]==n
        loc[x]=i
        x=p[x]
    assert x==mark and loc.itemsize==4 and sys.byteorder=='little'
    result=subprocess.check_output([exe],input=struct.pack('<I',n)+loc.tobytes())
    rows=[list(map(int,line.split())) for line in result.splitlines()]
    assert len(rows)==66
    return rows


def main(exe,tapes):
    for k in range(2,81):
        # With first-column labels a=0,b=1, first→last is stable sort indices.
        last=bytes([1,0,1])+bytes(2*k-3)
        by_s={r[0]:r for r in price(last,0,exe)}
        assert by_s[1][1]==k
        # The exceptional x=0 may split the ascending even-rank lane once.
        assert by_s[2][1]<=4
    old_file=Path(__file__).resolve().parents[1]/'blocks.jsonl'
    old={r['index']:r for r in map(json.loads,old_file.read_text().splitlines())}
    writer=csv.writer(sys.stdout,lineterminator='\n')
    writer.writerow(('index','n','stride','pieces','singletons','implied','max_span'))
    for tape in sorted(tapes.glob('*.irec.xz')):
        m,runs,mark,_,_=real.source(tape)
        last=b''.join(bytes([k])*c for k,c in runs)
        rows=price(last,mark,exe)
        assert rows[0][1]==old[m['index']]['global_axis']['spans']
        writer.writerows((m['index'],len(last),*row) for row in rows)


if __name__=='__main__':main(Path(sys.argv[1]).resolve(),Path(sys.argv[2]))
