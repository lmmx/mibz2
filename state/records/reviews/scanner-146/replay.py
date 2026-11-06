#!/usr/bin/env python3
"""Read-only replay of scanner checkpoint archives; no constructor or toolchain execution."""
import collections, gzip, hashlib, json, pathlib, re
ROOT = pathlib.Path(__file__).parent
EV = ROOT / 'evidence'
names = {}
def name(kind, text):
    m = re.fullmatch(r'\((\d+)\)(?: (.*))?', text)
    if not m:
        return text
    key = (kind, m[1])
    if m[2] is not None:
        names[key] = m[2]
    return names[key]

def profile(path):
    own = collections.defaultdict(collections.Counter)
    callees = collections.defaultdict(collections.Counter)
    callcounts = collections.defaultdict(collections.Counter)
    total = 0
    fn = target = None
    pending = False
    address = 0
    positions = events = None
    for number, line in enumerate(gzip.open(path, 'rt'), 1):
        line = line.rstrip('\n')
        if line.startswith('positions:'): positions = line.split()[1:]
        elif line.startswith('events:'): events = line.split()[1:]
        elif line.startswith('summary:'): total = int(line.split()[1])
        elif line.startswith(('fn=', 'cfn=')):
            v = name('fn', line.split('=',1)[1])
            if line.startswith('fn='): fn = v
            else: target = v
        elif re.match(r'(?:c?fi|c?fl|fe|c?ob)=', line): pass
        elif line.startswith('calls='):
            pending = True
            callcounts[fn][target] += int(line.split('=',1)[1].split()[0])
        elif line and (line[0].isdigit() or line[0] in '+-*'):
            parts = line.split()
            a = parts[0]
            if a == '*': pass
            elif a[0] in '+-': address += int(a)
            else: address = int(a,0)
            costs = [int(x) for x in parts[len(positions):]]
            assert len(costs) == len(events), (number, line)
            assert events == ['Ir'], events
            if pending:
                callees[fn][target] += costs[0]
            else:
                own[fn][address] += costs[0]
            pending = False
    assert sum(sum(v.values()) for v in own.values()) == total
    return own, callees, callcounts, total

def listing(path):
    out = {}
    for number, line in enumerate(path.read_text().splitlines(), 1):
        m = re.match(r'\s*([0-9a-f]+):\s+(.*)', line)
        if m: out[int(m[1],16)] = {'line':number,'asm':m[2]}
    return out

def main():
    own, callees, calls, total = profile(EV / 'per-instruction.callgrind.gz')
    report = {'profile_total_ir':total, 'functions':{},'count_classes':{}}
    builds = []
    for tag, asm in [('ctor2','v2-build-false-same-binary.asm'), ('ctor2s','ctor2s-build-false.asm')]:
        fn = f'<c30walk::{tag}::{"Ctor2" if tag=="ctor2" else "Ctor2s"}>::build::<false>'
        code = listing(EV / asm)
        costs = own[fn]
        assert set(costs).issubset(code), set(costs)-set(code)
        assert costs[min(code)] == 434
        builds.append(fn)
        report['functions'][tag] = {'static_instructions':len(code),'observed_instructions':len(costs),'self_ir':sum(costs.values()),'inclusive_ir':sum(costs.values())+sum(callees[fn].values()), 'callees':dict(callees[fn]),'call_counts':dict(calls[fn])}
        report['count_classes'][tag] = dict(sorted(collections.Counter(costs.values()).items()))
        rows = ['address\tlisting_line\texecutions\tasm']
        for addr,x in sorted(code.items()): rows.append(f'{addr:x}\t{x["line"]}\t{costs[addr]}\t{x["asm"]}')
        (ROOT / (tag+'-weighted.tsv')).write_text('\n'.join(rows)+'\n')
    report['self_saving'] = sum(own[builds[0]].values())-sum(own[builds[1]].values())
    report['inclusive_saving'] = report['self_saving']+sum(callees[builds[0]].values())-sum(callees[builds[1]].values())
    classes = set(report['count_classes']['ctor2'])|set(report['count_classes']['ctor2s'])
    delta = [{'frequency':k,'b2_instructions':report['count_classes']['ctor2'].get(k,0),'scanner_instructions':report['count_classes']['ctor2s'].get(k,0),'delta':k*(report['count_classes']['ctor2s'].get(k,0)-report['count_classes']['ctor2'].get(k,0))} for k in sorted(classes)]
    assert sum(x['delta'] for x in delta) == -report['self_saving']
    report['frequency_deltas'] = delta
    records = [json.loads(x) for x in (EV/'corpora.jsonl').read_text().splitlines() if x]
    primary = [x for x in records if x['input']=='wikidata-20261026.bz2' and x['permille']==0 and not x['extra_starts']]
    assert len(primary)==1
    p = primary[0]
    assert p['fresh_pairs']==p['tiles_nonuniform']+p['runs_no_full_tile']
    assert p['runs']==p['held_reuses']+p['runs_no_full_tile']
    assert p['fresh_pairs']+p['lookahead_pairs']==(p['rows']-p['tail_rows'])//16
    assert p['rows_scalar']==16*p['tiles_nonuniform']+p['tail_rows']
    report['primary_counters']=p
    (ROOT/'replay-result.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps({k:v for k,v in report.items() if k not in ('count_classes','frequency_deltas')},indent=2))
if __name__=='__main__': main()
