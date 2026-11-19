"""Insert only the selected suffix markers into an ordinary weighted BWT."""
from bisect import bisect_right
from collections import Counter
from itertools import groupby,product
from pathlib import Path
import csv
import ctypes as ct
import json
import sys

HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parents[1]))
from return_map import Atlas,Column,Oracle,append_run,heavy_set,incoming,truth
sys.path.insert(0,str(HERE.parents[1]/'native'))
from ordinal_writer import Byte,Object,Owned,Piece,U32
sys.path.insert(0,str(HERE.parents[1]/'origin'))
from origin import paths,read_input


class Insert:
    def __init__(self,raw,words,old_order,frequency,c,h):
        self.words=[c]
        self.bases=[0]
        self.old_ids={k:i+1 for i,k in enumerate(old_order)}
        self.selected={k for k in old_order if words[k][min(h,len(words[k])-1):]==c}
        self.s=s=sum(frequency[k] for k in self.selected)
        self.m=sum(frequency.values())
        self.old_bases={}
        pos=0
        for k in old_order:
            self.old_bases[k]=pos
            self.bases.append(s+pos)
            self.words.append(words[k][:-len(c)] if k in self.selected else words[k])
            pos+=frequency[k]
        markers=[]
        original=[]
        for k,w in raw:
            if k in self.selected:
                append_run(markers,self.old_ids[k],w)
                append_run(original,0,w)
            else:
                append_run(original,self.old_ids[k],w)
        self.marker_runs=len(markers)
        self.original_runs=len(original)
        self.root=markers
        for k,w in original:
            append_run(self.root,k,w)
        self.raw=raw
        self.old_order=old_order
        self.old_starts=[self.old_bases[k] for k in old_order]
        assert len(self.root)<=len(raw)+sum(1 for k,_ in raw if k in self.selected)
        assert sum(w for _,w in self.root)==self.m+s

    def mark(self,original,offset):
        k=self.old_order[bisect_right(self.old_starts,original)-1]
        length=len(self.words[self.old_ids[k]])
        if k not in self.selected or offset<length:
            return self.s+original,offset,0
        # Only the indicated symbol's source cursor is needed on the slow case.
        slot=self.old_bases[k]
        rank=0
        for i,(q,w) in enumerate(self.raw):
            if q==k:
                if original<slot+w:
                    return rank+original-slot,offset-length,i+1
                slot+=w
            if q in self.selected:
                rank+=w
        raise AssertionError('missing marked occurrence')

    def abi(self,lib,mark,skip,n):
        book=bytearray()
        spans=[]
        for w in self.words:
            spans.append(Piece(len(book),len(w)))
            book.extend(w)
        words=(Piece*len(spans))(*spans)
        source=(U32*len(spans))(*self.bases)
        data=(Byte*len(book)).from_buffer(book)
        runs=(Piece*len(self.root))(*(Piece(k,w) for k,w in self.root))
        obj=Object(runs,len(runs),words,source,len(words),data,len(book),mark,skip,self.m+self.s,n)
        flag=U32()
        item=lib.consume(ct.byref(obj),ct.byref(flag))
        try:
            return ct.string_at(item.data,item.len),int(flag.value)
        finally:
            lib.release(item)


def verify(col,oracle,u,raw,words,minima,freq,c,h,marks,lib=None):
    obj=Insert(raw,words,sorted(words,key=minima.get),freq,c,h)
    old,_=truth(oracle,u)
    # Dense "source -> old destination" truth and exact physical H cut rows.
    src={}
    h_at={}
    for word,pairs in old.items():
        pos=min(h,len(word)-1)
        for x,y in pairs:
            src[x-u[0]]=(y-u[0],word)
            if word[pos:]==c:
                z=x
                for _ in range(pos):
                    z=oracle.forward[z]
                h_at[y-u[0]]=z
    assert obj.s==len(h_at)
    h_dest=sorted(h_at)
    pos={y:i for i,y in enumerate(h_dest)}
    assert [h_at[y] for y in h_dest]==sorted(h_at.values())
    actual=[None]*(obj.m+obj.s)
    cursors=obj.bases.copy()
    y=0
    for k,w in obj.root:
        for _ in range(w):
            x=cursors[k]
            cursors[k]+=1
            assert actual[x] is None
            actual[x]=(y,k)
            y+=1
    for j,target in enumerate(h_dest):
        assert actual[j]==(obj.s+target,0)
    for x,(y,w) in src.items():
        d=min(h,len(w)-1)
        target,k=actual[obj.s+x]
        assert target==(pos[y] if w[d:]==c else obj.s+y)
        assert obj.words[k]==(w[:d] if w[d:]==c else w)
    backwards=[0]*col.rows
    for x,y in enumerate(oracle.forward):
        backwards[y]=x
    full=used=fallback=probes=inside=0
    for primary in marks:
        x=primary
        expected=bytearray()
        for _ in range(col.rows):
            expected.append(oracle.first[x]);x=oracle.forward[x]
        x=primary
        offset=0
        while not u[0]<=x<u[1] and offset<col.rows:
            x=backwards[x];offset+=1
        if offset==col.rows:
            fallback+=1
            continue
        mark,skip,work=obj.mark(x-u[0],offset)
        probes+=work
        inside+=work>0
        v=mark
        thought=bytearray()
        left=skip
        while len(thought)<col.rows:
            v,k=actual[v]
            thought.extend(obj.words[k][left:left+col.rows-len(thought)])
            left=0
        assert thought==expected
        if lib is not None and obj.s:
            data,taken=obj.abi(lib,mark,skip,col.rows)
            assert data==expected
            used+=taken
        full+=1
    selected_runs=sum(1 for k,_ in raw if k in obj.selected)
    return dict(m=obj.m,s=obj.s,old_K=len(words),changed_K=len(obj.selected),old_R=len(raw),
                selected_run_bound=selected_runs,filtered_R=obj.marker_runs,substituted_R=obj.original_runs,
                inserted_R=len(obj.root),old_D=sum(map(len,words.values())),
                new_weight_bytes=sum(map(len,obj.words)),empty_weights=sum(not w for w in obj.words),
                forced_by_mode=int(bool(obj.s and obj.s>=max(freq.values()))),entered_marks=full,
                missing_old_marks=fallback,native_second=used,inside_core_marks=inside,mark_slow_probes=probes)


def library(path):
    lib=ct.CDLL(str(Path(path).resolve()))
    lib.consume.argtypes=[ct.POINTER(Object),ct.POINTER(U32)]
    lib.consume.restype=Owned
    lib.release.argtypes=[Owned]
    lib.release.restype=None
    return lib


def small(lib):
    total=Counter();witness={}
    for n in range(1,7):
        for last in product(range(2),repeat=n):
            col=Column([(c,sum(1 for _ in it)) for c,it in groupby(last)])
            oracle=Oracle(bytes(last))
            for a in range(n):
                for b in range(a+1,n+1):
                    u=(a,b)
                    raw,words,minima,freq,_=incoming(col,u,Atlas(col,u,0))
                    for h in (0,1,3,n):
                        for c in sorted({w[min(h,len(w)-1):] for w in words.values()}|{b'x'}):
                            row=verify(col,oracle,u,raw,words,minima,freq,c,h,range(n),lib)
                            total.update(objects=1,marks=n,**{k:row[k] for k in
                               ('entered_marks','native_second','inside_core_marks','missing_old_marks')})
                            for name,cond in [('mixed_source',row['changed_K']>=2),('empty_weights',row['empty_weights']>0),
                                  ('winner_loses',0<row['s'] and not row['forced_by_mode']),
                                  ('absent',row['s']==0)]:
                                if cond:
                                    total[name]+=1
                                    witness.setdefault(name,dict(last=bytes(last).hex(),u=[a,b],h=h,c=c.hex(),**row))
    print(json.dumps(dict(total=total,witness=witness),indent=2))


def real(lib,tapes):
    writer=None
    fixed=b'":"value","property'
    for tape in paths(tapes):
        runs,mark=read_input(tape)
        col=Column(runs)
        oracle=Oracle(b''.join(bytes([c])*n for c,n in runs))
        q=col.interval(b'"')
        for name,u in [('quote',q),('frequent',heavy_set(col,q,1)[0][0])]:
            raw,words,minima,freq,_=incoming(col,u,Atlas(col,u,0))
            for h in (4,8,12):
                c=b',' if name=='quote' else fixed[h:]
                row=dict(ordinal=int(tape.name.split('.')[0]),section=name,h=h,
                     **verify(col,oracle,u,raw,words,minima,freq,c,h,[mark],lib))
                if writer is None:
                    writer=csv.DictWriter(sys.stdout,fieldnames=row,lineterminator='\n')
                    writer.writeheader()
                writer.writerow(row)
                sys.stdout.flush()


if __name__=='__main__':
    lib=library(sys.argv[1])
    real(lib,sys.argv[2]) if len(sys.argv)>2 else small(lib)
