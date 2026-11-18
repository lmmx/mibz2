"""Source-level counts and 64-byte line models, no timing or machine calibration."""
import argparse
import csv
from collections import OrderedDict
from pathlib import Path
import sys

from packets import contract as packets
from queue_ranges import contract as queues
from screen import inputs

HERE=Path(__file__).resolve().parent


def rows(name):
    with name.open() as f:
        return list(csv.DictReader(f))


def paired(suffix):
    return [dict((k,int(v)) for k,v in row.items()) for group in ('frozen','sample')
            for row in rows(HERE/(group+suffix+'.csv'))]


class Cache:
    def __init__(self, limit):
        self.limit=limit
        self.lines=OrderedDict()
        self.fills=self.dirty=0

    def touch(self,line,write):
        if line not in self.lines:
            self.fills += 1
            if len(self.lines)==self.limit:
                self.dirty += self.lines.popitem(last=False)[1]
            self.lines[line]=write
        else:
            self.lines[line] |= write
            self.lines.move_to_end(line)

    def transfers(self):
        return self.fills+self.dirty+sum(self.lines.values())


class Memory:
    def __init__(self):
        self.used=set()
        self.small=Cache(512)
        self.large=Cache(4096)

    def access(self,address,n,write=False):
        for line in range(address//64,(address+n-1)//64+1):
            self.used.add(line)
            self.small.touch(line,write)
            self.large.touch(line,write)


class Staged:
    """External ranges, an open-address transfer index, and coalesced residual records."""
    def __init__(self, ranges):
        self.mem=Memory()
        self.cap=1<<(2*ranges-1).bit_length()
        self.keys=[-1]*self.cap
        self.probes=self.source=self.arcs=0
        for i in range(self.cap):
            self.mem.access(16*i,4,True)

    def observe(self,op,key):
        if op in ('get','put'):
            i=((key*2654435761)&0xffffffff)>>(32-(self.cap.bit_length()-1))
            while True:
                self.probes+=1
                self.mem.access(16*i,4)
                if self.keys[i] in (-1,key):
                    break
                i=(i+1)%self.cap
            if op=='put':
                assert self.keys[i]==-1
                self.keys[i]=key
            else:
                assert self.keys[i]==key
            self.mem.access(16*i,16,op=='put')
        elif op in ('save_source','read_source'):
            self.source=max(self.source,key+1)
            self.mem.access((1<<32)+12*key,12,op=='save_source')
        else:
            assert op=='save_residual'
            self.arcs=key+1
            self.mem.access((2<<32)+12*key,12,True)

    def finish(self):
        for i in range(self.arcs):
            self.mem.access((2<<32)+12*i,12)


def fields(r):
    c,e,d,t,b=(r[k] for k in ('cuts','equal','closures','top_wins','bottom_wins'))
    assert c==t+b+e and d==1
    rt,rb,peak,letters=(r[k] for k in ('raw_top','raw_bottom','peak_pairs','bands'))
    assert rt+rb==letters
    ht,hb=e+d+b,e+d+t
    placement=3*c+e+3*t+4*b
    eager=6*(ht+hb)+placement
    lazy=5*letters+4*(ht-rt)+3*(hb-rb)+2*(letters-peak)+2*letters+placement+2*e
    words=2*c+2*(c-r['rules'])
    return eager,lazy,words,ht+hb


def lines(n):
    return (n+63)//64


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('folder',type=Path)
    args=parser.parse_args()
    res=paired('-residual')
    control={r['block']:r for r in res if r['B']==0}
    cut2={r['block']:r for r in res if r['B']==2}
    queue={r['block']:r for r in paired('-queues')}
    screen=paired('')
    prior=rows(HERE.parent/'2026-11-18-cells-endpoint-sweep'/'traffic.csv')
    writer=None
    for block,n,p,m,bands in inputs(args.folder,None):
        old,q=control[block],queue[block]
        eager,lazy,words,refs=fields(old)
        for model,g in [('eager',eager),('lazy',lazy)]:
            saved=next(r for r in prior if r['model']==model and int(r['block'])==block)
            assert g==sum(int(saved[k]) for k in ('core_r','core_w','hint_r'))
            assert words==sum(int(saved[k]) for k in ('word_r','word_w'))
        qe,ql,qw,qref=fields(q)
        se,sl,sw,sref=fields(cut2[block])
        memory=Staged(q['ranges'])
        (support,_,_,arcs),_=queues(bands,n,p,memory.observe)
        assert len(arcs)==memory.arcs
        memory.finish()
        pm=Memory()
        _,_,_,pc=packets(bands,n,p,2,lambda j:pm.access(4*j,4))
        matrices=[r for r in screen if r['block']==block]
        tile_min=min((lines(8*r['tiles'])+lines(8*r['shift_min_errors']),r['B']) for r in matrices)
        eg_credit=eager-qe+words-qw+8*(old['bands']-q['bands'])
        lz_credit=lazy-ql+words-qw+6*(old['bands']-q['bands'])
        row=dict(block=block,capped_input_lines=lines(2*m),output_lines=lines(n),
                 base_refills=refs,queue_refills=qref,B2_refills=sref,
                 base_eager_geo=eager,base_lazy_geo=lazy,base_word_fields=words,
                 queue_eager_geo=qe,queue_lazy_geo=ql,queue_word_fields=qw,
                 B2_eager_geo=se,B2_lazy_geo=sl,B2_word_fields=sw,
                 min_tile_scalar_lines=tile_min[0],attaining_B=tile_min[1],
                 B2_start_lines=len(pm.used),B2_start_fills32=pm.small.fills,
                 B2_start_fills256=pm.large.fills,B2_start_probes=pc['probes'],
                 queue_source_records=memory.source,queue_residual_records=memory.arcs,
                 queue_index_slots=memory.cap,queue_hash_probes=memory.probes,
                 queue_stage_lines=len(memory.mem.used),queue_transfers32=memory.mem.small.transfers(),
                 queue_transfers256=memory.mem.large.transfers(),
                 queue_source_residual_ops=2*(memory.source+memory.arcs),
                 eager_field_credit=eg_credit,lazy_field_credit=lz_credit,
                 queue_order_fields=3*((support+63)//64)+4*q['bands'],
                 B2_order_fields=3*((cut2[block]['support']+63)//64)+4*cut2[block]['bands'])
        if writer is None:
            writer=csv.DictWriter(sys.stdout,fieldnames=row,lineterminator='\n')
            writer.writeheader()
        writer.writerow(row)

if __name__=='__main__':
    main()
