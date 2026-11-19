"""Reproduce physical gap chain geometry only from authenticated capped original blocks."""
import csv
import hashlib
import json
from pathlib import Path
import subprocess
import sys
HERE=Path(__file__).resolve().parent
DONOR=next(p/'powers/xor/capped/receipt.json' for p in HERE.parents if (p/'powers/xor/capped/receipt.json').is_file())
caps,out=map(Path,sys.argv[1:]);out.mkdir(parents=True,exist_ok=True)
exe=out/'chains'
subprocess.run(['g++','-std=c++20','-O2','-Wall','-Wextra',HERE/'chains.cpp','-o',exe],check=True)
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
manifest={'cpp':sha(HERE/'chains.cpp'),'exe':sha(exe),'donor_receipt':sha(DONOR),'inputs':[]}
detail=out/'affine.csv'
header='block,n,runs,paths,edges,affine_bursts,single_edge_bursts,longest_burst,edge_lookups_elided\n'
with detail.open('x') as fd:fd.write(header)
saved=HERE.parents[2]/'blocks.csv'
manifest['adaptive_receipt']=sha(saved)
reference={int(v['block']):int(v['safe_endpoints']) for v in csv.DictReader(saved.open()) if v['penalty']=='1' and v['round']=='0'}
assert len(reference)==10
with (out/'blocks.csv').open('x') as f,(out/'path-hist.csv').open('x') as g,(out/'width-hist.csv').open('x') as h:
    f.write('block,n,runs,alphabet,marked_lap,arcs,paths,grounded,flexible,max_vertices,support_width_sum,support_width_max,intervals_with_terminal,additional_witnesses,minimum_selected_in_box,paths_forced_full,grounded_only_log2,feasible_box_log2,all_thresholds_log2,feasible_integer_lower_exponent,relaxed_integer_upper_exponent,f_supported_flexible,supporting_self_flexible,independent_optional_singletons,boundaries_safe_from_F_partition\n')
    g.write('block,grounded,length,allowed_feasible_thresholds,count\n')
    h.write('block,floor_log2_positive_support_width,count\n')
    for q in json.loads(DONOR.read_text())['inputs']:
        i=q['block'];path=caps/f'block-{i}.caps';digest=sha(path)
        assert digest==q['caps_le_sha256']
        manifest['inputs'].append(dict(block=i,mark=q['mark'],caps_sha256=digest))
        r=subprocess.run([exe,path,str(q['mark']),str(i),detail],text=True,capture_output=True,check=True)
        row=r.stdout.strip().split(',')
        assert int(row[-1])==reference[i]
        assert int(row[1])==q['rows'] and int(row[2])==q['maximal_runs'] and row[1]==row[4]
        f.write(r.stdout)
        for s in r.stderr.splitlines():
            label,values=s.split(',',1)
            assert label in ('path','width')
            (g if label=='path' else h).write(values+'\n')
(out/'inputs.json').write_text(json.dumps(manifest,indent=2)+'\n')
two=out/'alternating.csv'
with two.open('x') as fd:fd.write(header)
for g in range(1,65):
    path=out/'two-run.caps'
    path.write_bytes(bytes((g,1,g+1,0)))
    subprocess.run([exe,path,'0',str(g),two],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,check=True)
path.unlink()
for row in csv.DictReader(two.open()):
    g,n,r,paths,edges,bursts,single,longest,elided=map(int,row.values())
    assert (n,r,paths,edges,bursts,single,longest,elided)==(2*g+1,2,1,2*g-1,2*g-1,2*g-1,1,0)
positive=out/'translating.csv'
with positive.open('x') as fd:fd.write(header)
for n in range(3,128):
    path=out/'one-shift.caps';path.write_bytes(bytes((1,1,n-1,0)))
    subprocess.run([exe,path,'0',str(n),positive],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,check=True)
path.unlink()
for row in csv.DictReader(positive.open()):
    j,n,r,paths,edges,bursts,single,longest,elided=map(int,row.values())
    assert (j,r,paths,edges,bursts,single,longest,elided)==(n,2,1,n-2,1,int(n==3),n-2,n-3)
