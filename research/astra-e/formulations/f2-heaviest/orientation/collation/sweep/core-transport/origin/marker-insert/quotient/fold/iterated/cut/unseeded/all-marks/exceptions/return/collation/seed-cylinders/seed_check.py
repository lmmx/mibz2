from pathlib import Path
import csv,hashlib,json,sys,subprocess as sp
here=Path(__file__).resolve().parent
raw,ctx,w=map(Path,sys.argv[1:4]);w.mkdir(exist_ok=True,parents=True)
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
flags=['-std=c++20','-O3','-Wall','-Wextra','-Werror']
sp.run(['g++',*flags,here/'seed_driver.cpp','-o',w/'seeds'],check=True)
(w/'seed-small.txt').write_bytes(sp.check_output([w/'seeds']))
original={int(q['index']):q['input_sha256'] for q in csv.DictReader((here.parents[2]/'contexts/inputs.csv').open())}
truth={(int(q['index']),int(q['k'])):q['sha256'] for q in csv.DictReader((here.parents[2]/'contexts/blocks.csv').open())}
with (w/'seed-blocks.csv').open('w')as f:
 out=csv.writer(f,lineterminator='\n')
 out.writerow('block k n original_runs seams tested skipped compared_pairs mixed_contexts seed_cylinders exception_rows fm_rank_calls fm_comparisons histogram_slices psi_calls psi_comparisons'.split()+['reject_at_'+str(j)for j in range(1,17)])
 for i,h in original.items():
  path=raw/f'{i}.txt';assert sha(path)==h
  for k in (4,8,16):
   ref=ctx/str(i)/f'{k}.ctx';assert sha(ref)==truth[i,k]
   row=list(map(int,sp.check_output([w/'seeds',path,str(k),ref],text=True).split(',')))
   assert row[2]==row[3]+row[4] and row[3]==row[6]+sum(row[14:])
   assert row[5]==k*row[6]+sum((j+1)*v for j,v in enumerate(row[14:]))
   out.writerow([i,k,*row,*([0]*(16-k))]);f.flush()
(w/'seed-build.json').write_text(json.dumps(dict(flags=flags,compiler=sp.check_output(['g++','--version'],text=True).splitlines()[0],
 code={name:sha(here/name)for name in ('seed_check.py','seed_driver.cpp','seeds.hpp')},exe=sha(w/'seeds')),indent=2)+'\n')
