"""Stream the induction-root bucket and index its complement. All end offsets are exclusive."""
from bisect import bisect_right
from collections import Counter, deque
from itertools import groupby, product
from pathlib import Path
import csv
import json
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))
from quotient import Atlas, Column, Quotient, incoming, paths, read_input


class Runs:
    def __init__(self):
        self.starts = []
        self.records = []
        self.end = 0

    def add(self, word, width, lf):
        if not width:
            return
        if self.records:
            last, pos = self.records[-1]
            if last == word and pos + self.end - self.starts[-1] == lf:
                self.end += width
                return
        self.starts.append(self.end)
        self.records.append((word, lf))
        self.end += width

    def segments(self, lo, hi):
        assert 0 <= lo < hi <= self.end
        i = bisect_right(self.starts, lo) - 1
        while lo < hi:
            start = self.starts[i]
            word, lf = self.records[i]
            end = min(self.starts[i+1],hi) if i+1 < len(self.starts) else hi
            yield word, end-lo, lf+lo-start
            lo = end
            i += 1


def full(q):
    indexed = Runs()
    seen = Counter()
    for w, length in q.root:
        indexed.add(w, length, q.bases[w]+seen[w])
        seen[w] += length
    return indexed


def prepared(q):
    z = q.ids[q.z]
    b = q.cut
    e = b+q.freq[q.z]
    # Phase 1. Only noncoincident insertions need descriptors. Old source
    # order gives each event's rank in its own ID without a range query.
    events = []
    hazard_seen = Counter()
    passed_z = 0
    for k, width in q.raw:
        w = q.ids[k]
        if w == z:
            passed_z += width
        elif k in q.selected:
            events.append((b+passed_z, w, width, q.bases[w]+hazard_seen[w]))
            hazard_seen[w] += width
    # The root sink here materializes a reference list to audit the stream.
    # Each invocation of sink.add can instead consume or coalesce one pending
    # record and feed the existing per-child/selector reducer.
    root_sink = Runs()
    outer = Runs()
    old_at = event = 0
    seen = Counter()
    for k, width in q.raw:
        w = z if k in q.selected else q.ids[k]
        lf = q.bases[w]+seen[w]
        end = old_at+width
        lo = old_at
        if lo < b:
            cap = min(b, end)
            outer.add(w, cap-lo, lf)
            lo = cap
        while event < len(events) and events[event][0] <= min(end,e):
            cut, v, span, to = events[event]
            assert lo <= cut
            root_sink.add(w, cut-lo, lf+lo-old_at)
            root_sink.add(v, span, to)
            event += 1
            lo = cut
        if lo < e:
            cap = min(end,e)
            root_sink.add(w, cap-lo, lf+lo-old_at)
            lo = cap
        if lo < end:
            outer.add(w, end-lo, lf+lo-old_at)
        seen[w] += width
        old_at = end
    assert event == len(events)
    assert root_sink.end == q.s
    assert outer.end == q.m-q.freq[q.z]
    return root_sink,outer


def check(q):
    indexed = full(q)
    root, outer = prepared(q)
    b = q.cut
    z = q.ids[q.z]
    mass = q.s
    assert list(root.segments(0,mass)) == list(indexed.segments(b,b+mass))
    # This proves every 1-row LF and every range query restricted to ONE side.
    # The slit can contain adjacent equal labels whose published LF jumps over
    # a mass in the removed interval. Runs.add retains the break in that case.
    for x,y,offset in ((0,b,0),(b+mass,indexed.end,mass)):
        if x < y:
            assert list(indexed.segments(x,y)) == list(outer.segments(x-offset,y-offset))
    # Verify the exact endpoint/branch schedule of the actual contracted tree.
    freq = [q.freq[w] if w != q.z else mass for w in q.order]
    chosen = max(range(len(freq)),key=lambda j:(freq[j],-q.bases[j]))
    report = Counter(rows=indexed.end, published_runs=len(indexed.records),root_rows=mass,
        streamed_runs=len(root.records), outer_runs=len(outer.records), events=q.entry_runs,
        old_input_runs=len(q.raw), outer_pages=(outer.end+63)//64,
        full_pages=(indexed.end+63)//64,marker_chosen=int(chosen==z))
    if chosen != z or mass == indexed.end:
        return report
    todo = deque([(b,b+mass,True)])
    while todo:
        x,y,is_root = todo.popleft()
        report['nodes'] += 1
        while True:
            if is_root:
                got = list(root.segments(0,mass))
            else:
                assert y<=b or x>=b+mass
                offset = mass if x>=b+mass else 0
                got = list(outer.segments(x-offset,y-offset))
            expect = list(indexed.segments(x,y))
            assert got == expect
            report['root_queries' if is_root else 'outer_queries'] += 1
            report['root_pieces' if is_root else 'outer_pieces'] += len(got)
            is_root = False
            if len(got)==1 and got[0][0]!=z:
                _,width,x = got[0]
                y = x+width
                continue
            children={}
            for word,width,lf in got:
                if word != z:
                    a,hi = children.get(word,(lf,lf))
                    assert hi==lf
                    children[word]=(a,hi+width)
            todo.extend((lo,hi,False) for lo,hi in children.values())
            break
    assert report['root_queries']==1
    return report


def small():
    total=Counter()
    for n in range(1,7):
        for last in product(range(3),repeat=n):
            order=sorted(set(last),reverse=True)
            raw=[(k,sum(1 for _ in v)) for k,v in groupby(last)]
            f=Counter(last)
            for flags in product((False,True),repeat=len(order)):
                selected={k for k,flag in zip(order,flags) if flag}
                for z in selected:
                    words={k:(b'' if k==z else bytes([k]))+b'!' for k in order}
                    total.update(check(Quotient(raw,words,order,f,selected,z)))
                    total['objects']+=1
    print(json.dumps(total,indent=2,sort_keys=True))


def real(tapes):
    writer=None
    for tape in paths(tapes):
        runs,_=read_input(tape)
        col=Column(runs)
        u=col.interval(b'"')
        raw,lead,minima,count,_=incoming(col,u,Atlas(col,u,0))
        order=sorted(lead,key=minima.get)
        words={k:w[1:]+b'"' for k,w in lead.items()}
        z=next(k for k,w in words.items() if w==b',"')
        for h in (0,4,8,12):
            if h:
                selected={k for k,w in lead.items() if w[min(h,len(w)-1):]==b','}
                pivot=z
            else:
                pivot=max(order,key=lambda k:(count[k],-minima[k]))
                selected={pivot}
            q=Quotient(raw,words,order,count,selected,pivot)
            row=dict(ordinal=int(tape.name.split('.')[0]),h=h,**check(q))
            if writer is None:
                writer=csv.DictWriter(sys.stdout,fieldnames=row,lineterminator='\n')
                writer.writeheader()
            writer.writerow(row)
            sys.stdout.flush()


if __name__=='__main__':
    real(sys.argv[1]) if len(sys.argv)>1 else small()
