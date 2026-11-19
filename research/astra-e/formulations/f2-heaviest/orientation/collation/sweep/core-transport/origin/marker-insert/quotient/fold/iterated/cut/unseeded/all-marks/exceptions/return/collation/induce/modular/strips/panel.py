from pathlib import Path
import csv,hashlib,json,subprocess as sp,sys
here=Path(__file__).resolve().parent
ref,wires,work=map(Path,sys.argv[1:4]);work.mkdir(parents=True,exist_ok=True)
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
old=list(csv.DictReader((here.parent/'levels.csv').open()))
index={(int(x['block']),int(x['q']),int(x['level'])):x for x in old}
frozen=json.loads((here.parents[3]/'emitter/inputs.json').read_text())
flags=['-std=c++20','-O3','-g0','-Wall','-Wextra','-Werror']
sp.run(['g++',*flags,here/'check.cpp','-o',work/'check'],check=True)
(w:=work/'small.txt').write_bytes(sp.check_output([work/'check']))
(work/'surgery.txt').write_bytes(sp.check_output([sys.executable,here/'surgery.py']))
names='block q level old_rows retained old_bands path_sinks sink_intervals launch_packets reverse_visited reverse_lookups reverse_comparisons packet_splits sort_cut_comparisons cut_intervals finite_paths_with_residue empty_paths canonical_bands representative_steps forward_comparisons new_joins sort_image_comparisons total_joins rauzy_residue_pieces rauzy_strict rauzy_equal rauzy_self rauzy_total_joins'.split()
with (work/'levels.csv').open('w')as f:
 out=csv.writer(f,lineterminator='\n');out.writerow(names)
 for q in frozen:
  b=q['block'];mask=wires/f'{b}.mask8';raw=ref/f'block-{b}.raw';assert sha(mask)==q['mask_sha256'] and sha(raw)==q['raw_sha256']
  data=sp.check_output([work/'check',mask,raw,str(b)],text=True)
  for row in csv.reader(data.splitlines()):
   r=dict(zip(names,map(int,row),strict=True));prior=index.pop((b,r['q'],r['level']))
   for a,k in [('canonical_bands','final_bands'),('old_bands','old_bands'),('rauzy_strict','strict'),('rauzy_equal','equal'),('rauzy_self','self'),('rauzy_residue_pieces','residue_pieces')]:assert r[a]==int(prior[k])
   assert r['canonical_bands']==1+r['finite_paths_with_residue'] and r['reverse_visited']<=max(0,r['old_rows']-r['q'])
   assert r['representative_steps']<=r['old_rows'] and r['path_sinks']==r['finite_paths_with_residue']+r['empty_paths']
   out.writerow(row)
  f.flush()
assert not index
manifest={x.name:sha(x)for x in [here/'strips.hpp',here/'check.cpp',here/'surgery.py',here/'panel.py',here.parent/'modular.hpp',here.parents[1]/'induce.hpp',work/'check']}
(work/'build.json').write_text(json.dumps(dict(sha256=manifest,compiler=sp.check_output(['g++','--version'],text=True).splitlines()[0],flags=flags),indent=2)+'\n')
