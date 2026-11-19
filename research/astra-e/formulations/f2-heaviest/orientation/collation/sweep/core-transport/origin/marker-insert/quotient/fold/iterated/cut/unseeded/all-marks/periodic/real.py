"""Independent representative-orbit certificate on the ten marked original maximal-run tapes."""
from array import array
import csv,hashlib
from itertools import accumulate
from math import gcd
from pathlib import Path
import sys

writer=None
for path in sorted(Path(sys.argv[1]).glob('0*/input.txt')):
    lines=path.read_text().splitlines();mark=int(lines[0])
    runs=[tuple(map(int,l.split())) for l in lines[1:]]
    g=gcd(*(width for c,width in runs));n=sum(width for c,width in runs);d=n//g
    counts=[0]*256
    for c,width in runs:counts[c]+=width//g
    source=[0,*accumulate(counts)];next_slot=source[:];F=bytearray()
    for c,width in enumerate(counts):F.extend(bytes([c])*width)
    psi=array("I",[0])*d;at=0
    for c,width in runs:
        width//=g;j=next_slot[c];psi[j:j+width]=array("I",range(at,at+width));next_slot[c]+=width;at+=width
    q=mark//g;period=bytearray()
    while True:
        period.append(F[q]);q=psi[q]
        if q==mark//g:break
        assert len(period)<d
    got=(period*((n+len(period)-1)//len(period)))[:n]
    expected=(path.parent/'cut.bin').read_bytes();assert got==expected
    row=dict(tape=path.parent.name,n=n,mark=mark,maximal_runs=len(runs),gcd=g,quotient_rows=d,first_return=len(period),single_circular_tape=int(len(period)==d),original_sha256=hashlib.sha256(got).hexdigest())
    if writer is None:writer=csv.DictWriter(sys.stdout,row,lineterminator="\n");writer.writeheader()
    writer.writerow(row);sys.stdout.flush()
