import hashlib,json,math,statistics as st
from pathlib import Path

def ratio(xs):
    ys=[math.log(x) for x in xs];m=st.mean(ys);h=2.145*st.stdev(ys)/math.sqrt(len(ys))
    return [math.exp(m),math.exp(m-h),math.exp(m+h)]

out={}
for name,folder,archive,digest in [('new','direct-evidence','direct.zip','00b0e008bdd8a90108ccaa87e21467dd35d70df35e1b71b86e687078a64a3e94'),('old','old-stage-evidence','old-stage.zip','9be443df35b8aa51c08c4b9491557b0d077cb6b78a11167625e2f7195e88d539')]:
    assert hashlib.sha256(Path(archive).read_bytes()).hexdigest()==digest
    x=json.loads(Path(folder,'run-fed-1.json').read_text());f=x['files'][0]
    assert x['rounds']==15 and len(f['entry'])==len(f['phases'])==15
    O=209700196;cb=x['ghz']*1e9/O
    y={'archive_sha256':digest,'head':x['head'],'host':x['host'],'ghz':x['ghz'],'B_A':ratio([sum(r[1])/sum(r[0]) for r in f['entry']]),'front_B_A':ratio([r[1][0]/r[0][0] for r in f['phases']]),'fill_B_build_A':ratio([r[1][2]/r[0][2] for r in f['phases']]),'sort_B_build_A':ratio([r[1][1]/r[0][2] for r in f['phases']]),'phase_ns_per_output_B':[[st.median(r[a][p] for r in f['phases'])*1e9/O for p in range(4)] for a in range(2)],'B_synthetic':{q['name']:ratio([sum(r[1])/sum(r[0]) for r in q['entry']]) for q in x['files'][1:]}}
    if name=='new':
        y.update(D_A=ratio([sum(r[2])/sum(r[0]) for r in f['entry']]),D_B=ratio([sum(r[2])/sum(r[1]) for r in f['entry']]),place_sortfill=ratio([r[2][2]/sum(r[1][1:3]) for r in f['phases']]))
        gap=st.median((r[2][2]-sum(r[1][1:3]))*cb for r in f['phases']);sort=st.median(r[1][1]*cb for r in f['phases']);fill=st.median(r[1][2]*cb for r in f['phases']);R=f['records'];N=f['rows_runs'];S=f['d_streams'];C=f['cells']
        y['budget']={'gap_per_output_B':gap,'gap_per_record':gap*O/R,'gap_per_cell':gap*O/C,'gap_per_seed':gap*O/S,'sort_per_output_B':sort,'fill_per_output_B':fill,'sort_allowance_per_record':sort*O/R,'sort_fill_per_accepted_row':(sort+fill)*O/N,'records_per_output_B':R/O,'accepted_rows_per_output_B':N/O,'cells_per_record':C/R,'D_extra_source_store_ops':f['d_vector']+f['d_scalar']-f['slots']/8,'D_scalar':f['d_scalar'],'seed_probe_upper_20_per_seed':20*S,'seed_only_required_cost_per_probe_at_upper_count':gap*O/(20*S),'four_byte_index_bytes_saved':4*R,'physical_cacheline_misses_allowance_per_record_if_all_sort_saved':{str(L):sort*O/R/L for L in [20,50,100,200]}}
    out[name]=y
Path('replay-result.json').write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps(out,indent=2))
