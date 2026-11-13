"""Inventory saved evidence only. No generator, decoder, compilation or timing."""
import json,pathlib,collections,statistics,math
def read(p):return json.loads(pathlib.Path(p).read_text())
def interval(v):
    z=[math.log(x) for x in v];m=statistics.mean(z);h=2.2621571628540993*statistics.stdev(z)/math.sqrt(len(z))
    return [math.exp(m),math.exp(m-h),math.exp(m+h)]
base=pathlib.Path('live-raw/20261113T101816Z/attempt')
rows=[json.loads(s) for s in (base/'runs.jsonl').read_text().splitlines()]
ci=read('cells-ci/result.json')
crows=[json.loads(s) for s in pathlib.Path('cells-ci/cells.jsonl').read_text().splitlines()]
out={'scope':'delta inventory; reuses parent verified archives, no repeated archive/decoder replay','historical_ci_source':'57f51bbc0ca62ac3ee1cc148a1010842eea6222a','current_source':'d4f0e8fb3ac68fa649e1810dff58dc06e9c545c2','families':{},'structural_screen':{}}
for corpus in ['wikidata','onebyte','pattern','random','records','runs','text']:
    entry={'historical_input':ci['corpora'][corpus]}
    old=[x for x in crows if x['corpus']==corpus]
    a=[x for x in old if x['arm']=='mibz2-cells'];b=[x for x in old if x['arm']=='mibz2@bin/ab']
    entry['historical_ci']={'pairs':len(a),'decoded_ab_hashes':sorted(set(x['hash'] for x in old)),'decoded_lengths':sorted(set(x['out'] for x in old)),'errors':sorted(set(x.get('err','') for x in old)),'wall_ratio':interval([x['wall']/y['wall'] for x,y in zip(a,b)]),'cpu_ratio':interval([x['cpu']/y['cpu'] for x,y in zip(a,b)]),'rss_kib':{arm:{'min':min(x['rss_kb'] for x in old if x['arm']==arm),'max':max(x['rss_kb'] for x in old if x['arm']==arm),'median':statistics.median(x['rss_kb'] for x in old if x['arm']==arm)} for arm in ['mibz2@bin/ab','mibz2-cells']},'route_census':'NOT MEASURED'}
    if corpus in ['wikidata','pattern','onebyte']:
        checks=[json.loads(x.read_text()) for x in base.glob('00*-'+corpus+'-*.json') if int(x.name[:3])<9]
        assert len(checks)==3
        entry['decoded_sha256']=sorted(set(x['sha256'] for x in checks))
        on=next(x for x in rows if x['corpus']==corpus and x['arm']=='cells' and x['trace'])
        route=collections.Counter();groups=collections.defaultdict(list)
        for block in on['blocks']:
            c=block['counters'];kind='admitted' if block['outcome'].startswith('Ok') else block['outcome'] or 'column'
            route[kind]+=1;groups[kind].append(c)
        entry['current_live_routes']=dict(route)
        off=[x for x in rows if x['corpus']==corpus and not x['trace']]
        a=sorted([x for x in off if x['arm']=='cells'],key=lambda x:x['round']);b=sorted([x for x in off if x['arm']=='baseline'],key=lambda x:x['round'])
        entry['current_live_off']={'pairs':10,'wall_ratio':interval([x['wall_ns']/y['wall_ns'] for x,y in zip(a,b)]),'cpu_ratio':interval([x['cpu_s']/y['cpu_s'] for x,y in zip(a,b)]),'rss_kib_values':sorted(set(x['rss_kib'] for x in off)),'memory_parity_established':False}
        out['structural_screen'][corpus]={}
        for kind,ls in groups.items():
            have=[c for c in ls if 'records' in c];rat=[c['records']/c['rows'] for c in have]
            out['structural_screen'][corpus][kind]={'blocks':len(ls),'rows':sum(c['rows'] for c in ls),'descriptor_blocks':len(have),'r_over_n_range':[min(rat),max(rat)] if rat else None,'bypass_64r_lt_n':sum(64*c['records']<c['rows'] for c in have)}
    else:
        entry['decoded_sha256']='NOT RECORDED in inspected archived CI artifact'
        entry['current_live_routes']='NOT MEASURED'
        entry['current_live_off']='NOT MEASURED'
    out['families'][corpus]=entry
pathlib.Path('seven-coverage.json').write_text(json.dumps(out,indent=2)+'\n')
for k,e in out['families'].items():
    print(k,'old wall/cpu',e['historical_ci']['wall_ratio'],e['historical_ci']['cpu_ratio'],'current',e['current_live_off'])
print(json.dumps(out['structural_screen'],indent=2))
