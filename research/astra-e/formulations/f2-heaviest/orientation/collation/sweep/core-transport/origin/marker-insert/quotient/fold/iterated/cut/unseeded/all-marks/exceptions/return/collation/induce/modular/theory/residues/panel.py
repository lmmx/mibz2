"""Compare the independently dense first returns against every original modular layer."""
import csv
import hashlib
import io
import json
from pathlib import Path
import subprocess
import sys
HERE=Path(__file__).resolve().parent
DONOR=next(p/'powers/xor/capped/receipt.json' for p in HERE.parents if (p/'powers/xor/capped/receipt.json').is_file())
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
caps,out=map(Path,sys.argv[1:]);out.mkdir(parents=True,exist_ok=True);exe=out/'census'
subprocess.run(['g++','-std=c++20','-O2','-Wall','-Wextra',HERE/'census.cpp','-o',exe],check=True)
ref={(int(r['block']),int(r['q']),int(r['level'])):r for r in csv.DictReader((HERE.parents[1]/'levels.csv').open())}
fields='block q level n mark phase kept active_residues quotient_arcs singleton_out_residues singleton_out_rows marked_deterministic_prefix closed_marked_period other_closed_residues any_closed_rows n_ge_2q dense_visited'.split()
n=0;groups={}
with (out/'levels.csv').open('x') as f:
    w=csv.writer(f,lineterminator='\n');w.writerow([*fields,'original_eliminate_steps','original_join_steps','original_pieces'])
    for b in json.loads(DONOR.read_text())['inputs']:
        i=b['block'];src=caps/f'block-{i}.caps';assert sha(src)==b['caps_le_sha256']
        for row in csv.DictReader(io.StringIO(subprocess.check_output([exe,src,str(b['mark']),str(i)],text=True)),fieldnames=fields):
            v={k:int(x)for k,x in row.items()}
            old=ref[v['block'],v['q'],v['level']];assert all(v[k]==int(old[k])for k in ('n','kept'))
            assert int(old['marked_lap'])==b['rows']
            joins=sum(int(old[k])for k in ('strict','equal','partial'))
            w.writerow([*row.values(),joins+int(old['self']),joins,old['residue_pieces']]);n+=1
            for kind,condition in [('all',True),('n_ge_2q',v['n_ge_2q']),('q_lt_n_lt_2q',v['q']<v['n']<2*v['q']),('n_le_q',v['n']<=v['q'])]:
                if not condition:continue
                a=groups.setdefault((v['q'],kind),dict(levels=0,marked_closed=0,qualifying_joins=0,total_joins=0,qualifying_pieces=0,
                    prefix_only=0,single_out_rows=0,total_rows=0,other_closed=0))
                a['levels']+=1;a['marked_closed']+=bool(v['closed_marked_period']);a['total_joins']+=joins
                a['qualifying_joins']+=bool(v['closed_marked_period'])*joins
                a['qualifying_pieces']+=bool(v['closed_marked_period'])*int(old['residue_pieces'])
                a['prefix_only']+=bool(v['marked_deterministic_prefix']) and not v['closed_marked_period']
                a['single_out_rows']+=v['singleton_out_rows'];a['total_rows']+=v['n'];a['other_closed']+=v['other_closed_residues']
assert n==len(ref)
with (out/'summary.csv').open('x') as f:
    w=csv.writer(f,lineterminator='\n');w.writerow(['q','scope',*next(iter(groups.values()))])
    for (q,kind),v in sorted(groups.items()):w.writerow([q,kind,*v.values()])
