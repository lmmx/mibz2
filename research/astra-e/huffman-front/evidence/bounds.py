"""Conditional critical-path budgets, reconstructed from pinned original archives."""
import csv
import hashlib
import io
import json
import math
from pathlib import Path
import re
import statistics
import subprocess
import tarfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
ARCHIVES = {
    'roles': ('824d095447faa59d1c33f8e685828784aed863ee', 'opend-sweep/run-20261118T225038Z.tar.gz'),
    'scalar': ('1ac706d628cd875092bfcbcf50e72f526018ee82','astra-e-scalar/run-20261119T045900Z.tar.gz'),
    'counter': ('1b8d2a33a9976e0f8bc4d7cda2e6188dd6ab8ab5','astra-e-cap-counter/run-20261119T061036Z.tar.gz'),
    'finder': ('727674e185744b91077dd4a862f2cd9e1485f0d0','par-scan/20261110T030608Z.tar.gz'),
    'postwalk': ('0026923046a02598bc7ce9607902e02ce01ee1d3','fused/run-20261117T115747Z.tar.gz'),
}
MED = statistics.median
WIKI_BYTES = 838_800_784


def blob(rev, path):
    return subprocess.check_output(['git', '-C', ROOT, 'show', rev + ':' + path])


def archive(name, member):
    revision, path = ARCHIVES[name]
    packed = blob(revision, path)
    assert hashlib.sha256(packed).hexdigest() == blob(revision, path+'.sha256').decode().split()[0]
    with tarfile.open(fileobj=io.BytesIO(packed)) as z:
        found = [m for m in z if m.name.rsplit('/',1)[-1]==member]
        assert len(found)==1 and found[0].isfile()
        return z.extractfile(found[0]).read().decode()


def rows(name):
    return [json.loads(s) for s in archive(name, 'runs.jsonl').splitlines()]


def role_readings(records, w):
    sample = [r for r in records if r['corpus']=='wikidata-x4' and r['threads']==w]
    lazy_wall = {r['round']:r['wall'] for r in sample if '-occ-' in r['arm']}
    assert len(lazy_wall)==12 and len(sample)==24
    by_arm = {}
    for r in sample:
        l=lazy_wall[r['round']]
        who='lazy' if '-occ-' in r['arm'] else 'opend'
        ts=dict(s.split('=') for s in r['thread_cpu'].split())
        worker=[float(value) for k,value in ts.items() if re.fullmatch(r'mibz2-\d+',k)]
        assert len(worker)==w and r['termination']=='success'
        scan=float(ts['mibz2-scan'])
        named=float(ts.get('main',0))+float(ts.get('mibz2-read',0))
        by_arm.setdefault(who,[]).append(dict(scanner=scan/l,workers=sum(worker)/l,
            other_residual=(r['cpu']-sum(worker)-scan)/l,named_other=named/l))
    return {a: {k:MED(row[k] for row in values) for k in values[0]} for a,values in by_arm.items()}


def budgets(records,w,arm,roles):
    a = {r['round']:r for r in records if (r['workers'],r['arm'])==(w,arm)}
    ref = {r['round']:r for r in records if (r['workers'],r['arm'])==(w,'lazy')}
    assert a.keys()==ref.keys()
    assert all(r['out_bytes']==WIKI_BYTES for r in [*a.values(),*ref.values()])
    assert not any('thread_cpu' in r for r in records)
    lazy=MED(r['wall_s'] for r in ref.values())
    baseline=statistics.geometric_mean(a[i]['wall_s']/ref[i]['wall_s'] for i in a)
    cpu=MED(a[i]['cpu_s']/ref[i]['wall_s'] for i in a)
    fits={}
    for source,old in roles.items():
        # O contains quantization residual as well as other roles: these are transfer assumptions.
        scan=old['scanner']
        o=old['other_residual']
        need=(cpu-o)-.80*w
        fits[source]=dict(scanner_room_to_target=.80-scan,
            scanner_room_to_target_seconds=(.80-scan)*lazy,
            allowed_scanner_growth=.80/scan-1,
            scanner_room_to_match_scalar=baseline-scan,
            worker_floor_if_no_cpu_saved=(cpu-o)/w,
            min_net_worker_saving_over_lazy=max(need,0),
            min_net_cpu_saving_ms=1000*max(need,0)*lazy,
            worker_floor_using_only_named_other=(cpu-old['named_other'])/w)
    return dict(rounds=len(a),scalar_over_lazy=baseline,lazy_median_wall_seconds=lazy,
                scalar_process_cpu_per_lazy_wall=cpu,role_transfer=fits)


def costs():
    def table(rev,path):
        return list(csv.DictReader(io.StringIO(blob(rev,'research/astra-e/'+path).decode())))
    geo=table('8ca3ec32336f24a4fbf44eb1f3894666164e12c5','front-fusion/geometry.csv')
    old=table('5e29072b016095921700f40abb8da10740d34cc4','mtf-pairs/geometry/blocks.csv')
    blocks={int(r['block']):r for r in old}
    assert len(blocks)==431 and len(geo)==383
    assert all(int(row['syms'])==int(blocks[int(row['stream'])]['syms']) for row in geo)
    fallback=[r for r in blocks.values() if r['route']=='0']
    assert len(fallback)==48
    return dict(x4=dict(symbols=4*sum(int(r['syms']) for r in blocks.values()),
        u16_payload=8*sum(int(r['syms']) for r in blocks.values()),
        capped_records=4*sum(int(r['caps']) for r in geo),
        capped_u16_payload=8*sum(int(r['caps']) for r in geo),
        column_fallback_symbols=4*sum(int(r['syms']) for r in fallback),
        column_fallback_rows=4*sum(int(r['n']) for r in fallback),
        word_aligned_run_bitmaps=32*sum((int(r['syms'])+63)//64 for r in blocks.values())),
        max_symbols_in_block=max(int(r['syms']) for r in blocks.values()),
        max_caps_in_block=max(int(r['caps']) for r in geo),
        u16_reservation_per_full_slot=2*18002*50)


def main():
    raw=rows('roles')
    # The independent verified scalar is called candidate in its original four-arm run.
    measured={k:rows(k) for k in ('scalar','counter')}
    result={}
    for w in range(7,11):
        role=role_readings(raw,w)
        result[w]=dict(historical_role_demands_per_lazy_wall=role,
            scalar48=budgets(measured['scalar'],w,'candidate',role),
            scalar40=budgets(measured['counter'],w,'scalar',role))
    print(json.dumps(dict(workers=result,payload=costs(),
            prior_find_result=archive('finder','outcome.txt').strip(),
            prior_fused_scope=archive('postwalk','console.log').splitlines()[0]),indent=2))


if __name__=='__main__':
    main()
