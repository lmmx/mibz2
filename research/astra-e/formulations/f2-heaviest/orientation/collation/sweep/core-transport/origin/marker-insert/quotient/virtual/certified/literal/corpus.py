"""Freeze held-out identity by intrinsic serialized caps and marked rank, excluding external ordinal."""
from hashlib import sha256
from pathlib import Path
import csv
import struct
import sys

def records(path):
    data=Path(path).read_bytes();at=0
    while at<len(data):
        i,n,p,k=struct.unpack_from('<4I',data,at);end=at+16+2*k
        assert end<=len(data)
        yield i,sha256(data[at+4:end]).hexdigest(),data[at:end]
        at=end

ten=list(records(sys.argv[1]));all_=list(records(sys.argv[2]))
assert len(ten)==10 and len(all_)==228
train={h:i for i,h,_ in ten}
assert len(train)==10 and len({h for _,h,_ in all_})==228
common={i:train[h] for i,h,_ in all_ if h in train}
assert len(common)==9
with Path(sys.argv[3]).open('wb') as held:
    writer=csv.writer(sys.stdout,lineterminator='\n')
    writer.writerow(('admission','digest','train_ordinal','held_out'))
    for i,h,data in all_:
        writer.writerow((i,h,train.get(h,''),int(i not in common)))
        if i not in common:held.write(data)
