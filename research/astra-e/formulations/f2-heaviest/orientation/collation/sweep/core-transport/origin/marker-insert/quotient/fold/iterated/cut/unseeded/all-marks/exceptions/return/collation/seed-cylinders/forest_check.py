from pathlib import Path
import csv,hashlib,json,sys,subprocess as sp,tarfile
here=Path(__file__).resolve().parent
raw,ctx,ref,w=map(Path,sys.argv[1:5]);w.mkdir(exist_ok=True,parents=True)
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
head=(here/'seed_driver.cpp').read_text().split('void verify(',1)[0]
(w/'small-geometry.hpp').write_text(head)
flags=['-std=c++20','-O3','-Wall','-Wextra','-Werror']
sp.run(['g++',*flags,'-I',w,'-I',here,here/'forest_driver.cpp','-o',w/'forest'],check=True)
(w/'forest-small.txt').write_bytes(sp.check_output([w/'forest']))
truth={(int(q['index']),int(q['k'])):q['sha256'] for q in csv.DictReader((here.parents[2]/'contexts/blocks.csv').open())}
pub=here.parent/'publish';files=json.loads((pub/'inputs.json').read_text())
with tarfile.open(pub/'columns.tar.xz','r:xz')as arc,(w/'forest-blocks.csv').open('w')as f:
 out=csv.writer(f,lineterminator='\n')
 out.writerow('block k n events seeds internal_nodes unary branch classes covered_rows intersections seed_queries seed_comparisons literal_bytes merge_claims produced_merge_records peak_live_merge_records output_runs phase_steps fallback phase_identical mark_psi_steps'.split())
 for q in files:
  i,k=q['block'],q['k'];path=raw/f'{i}.txt';mod=ctx/str(i)/f'{k}.ctx';want=ref/f'block-{i}.raw'
  wire=w/f'{i}-{k}.col';wire.write_bytes(arc.extractfile(wire.name).read())
  assert sha(wire)==q['column_sha256'] and sha(path)==q['original_rle_sha256'] and sha(want)==q['marked_sha256'] and sha(mod)==truth[i,k]
  row=list(map(int,sp.check_output([w/'forest',path,str(k),mod,wire,want],text=True).split(',')))
  out.writerow([i,k,*row]);f.flush()
(w/'forest-build.json').write_text(json.dumps(dict(flags=flags,compiler=sp.check_output(['g++','--version'],text=True).splitlines()[0],exe=sha(w/'forest'),
 code={n:sha(here/n)for n in ['forest_check.py','forest_driver.cpp','forest.hpp','seeds.hpp','seed_driver.cpp']},
 generated=sha(w/'small-geometry.hpp')),indent=2)+'\n')
