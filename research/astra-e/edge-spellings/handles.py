"""Exact band dictionary census, including the equal-input-cost neighbor opportunity."""
import csv
from collections import Counter
from pathlib import Path
import struct
import sys
import lzma

def words(path):
    with (lzma.open(path, 'rb') if path.suffix=='.xz' else path.open('rb')) as f:
        while header:=f.read(24):
            i,stream,n,m,edge,count=struct.unpack('<6I',header)
            items=[]
            for _ in range(count):
                h,q,k=struct.unpack('<3I',f.read(12))
                w=f.read(k)
                assert len(w)==k and q>0 and k>=2
                items.append((h,q,w))
            yield (i,stream,n,m,edge),items

def census(path,output):
    writer=None
    for (i,stream,n,m,edge),items in words(path):
        row=dict(block=i,stream=stream,n=n,m=m,edge=edge)
        count=Counter()
        keys=set()
        remain=set()
        bytewords=set()
        spellings={}
        previous=None
        last_by_handle={}
        for h,q,w in items:
            key=(h,w[-1]);tag=h>>30
            assert spellings.setdefault(key,w)==w
            duplicate=key in keys
            count.update(bands=1,visits=q,spelling=len(w),expanded=q*len(w))
            for cat,condition in [('handle',duplicate),('bytes',w in bytewords),
                                  ('adjacent',key==previous),('cached_suffix',last_by_handle.get(h)==w[-1])]:
                if condition:
                    count.update({cat+'_bands':1,cat+'_spelling':len(w),
                                  cat+'_visits':q,cat+'_expanded':q*len(w)})
            if duplicate:
                count.update({f'tag{tag}_bands':1,f'tag{tag}_spelling':len(w)})
            previous=key
            last_by_handle[h]=w[-1]
            keys.add(key)
            bytewords.add(w)
            remain.add(h)
        assert count['handle_bands']==count['cached_suffix_bands']
        for cat in ['handle','bytes','adjacent','cached_suffix']:
            row.update({cat+'_'+k:count[cat+'_'+k] for k in ['bands','spelling','visits','expanded']})
        for tag in range(4):
            row.update({f'tag{tag}_'+k:count[f'tag{tag}_'+k] for k in ['bands','spelling']})
        row.update({k:count[k] for k in ['bands','visits','spelling','expanded']})
        row['offsets']=4*(count['bands']+1)
        row['distinct_handles']=len(remain)
        row['max_word']=max(len(w) for _,_,w in items)
        if writer is None:
            writer=csv.DictWriter(output,row.keys(),lineterminator='\n')
            writer.writeheader()
        writer.writerow(row)
if __name__=='__main__':
    census(Path(sys.argv[1]),sys.stdout)
