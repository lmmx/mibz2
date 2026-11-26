"""Build/check all 80 native models from the original run tapes and freeze logical work/output hashes."""
import csv,hashlib,json,lzma,struct,subprocess,sys
from pathlib import Path
here=Path(__file__).resolve().parent
powers=(here.parents[3]/'powers').resolve()
sys.path.insert(0,str(powers))
import importlib.util
spec=importlib.util.spec_from_file_location("powers_oracle",powers/"real.py")
module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
read_source=module.source
exe,tapes,storage=map(lambda p:Path(p).resolve(),sys.argv[1:])
storage.mkdir(exist_ok=True,parents=True)
data=storage/'data';data.mkdir(exist_ok=True)
orig=storage/'raw';orig.mkdir(exist_ok=True)
counts=[];steps=[];hashes=[]
pdir=here.parent/'panel'
panel={(int(r['index']),int(r['k'])):r for r in csv.DictReader((pdir/'blocks.csv').open())}
for tape in sorted(tapes.glob('*.irec.xz')):
    meta,runs,primary,_,_=read_source(tape);idx=meta['index']
    input_path=orig/(str(idx)+'.txt')
    input_path.write_text(str(primary)+'\n'+''.join(f'{a} {n}\n' for a,n in runs))
    folder=data/str(idx);folder.mkdir(exist_ok=True)
    output=subprocess.check_output([exe,'check',input_path,folder],text=True)
    for r in csv.DictReader(output.splitlines()):
        k=int(r['k']);p=panel[idx,k]
        assert all(int(r[c])==int(p[w]) for c,w in [('n','n'),('states','states'),('edges','edges'),('exception_rows','exceptions')])
        wire=(folder/(str(k)+'.ctx')).read_bytes()
        assert wire[:8]==b'CTXMOD01'
        n,mark,depth,c,d,e,s=struct.unpack_from('<7I',wire,8)
        assert (n,mark,depth,c,d,e)==(meta['n'],primary,k,int(r['states']),int(r['descriptors']),int(r['exception_rows']))
        assert len(wire)==36+9*c+13*d and s<c
        counts.append(dict(index=idx,**r,primary_state=s,sha256=hashlib.sha256(wire).hexdigest()))
    for r in csv.DictReader((folder/'layers.csv').open()):steps.append(dict(index=idx,**r))
    hashes.append(dict(index=idx,tape=tape.name,tape_sha256=hashlib.sha256(tape.read_bytes()).hexdigest(),
                       input_sha256=hashlib.sha256(input_path.read_bytes()).hexdigest()))
for name,records in [('blocks',counts),('layers',steps),('inputs',hashes)]:
    with (storage/(name+'.csv')).open('w',newline='') as f:
        w=csv.DictWriter(f,fieldnames=records[0],lineterminator='\n');w.writeheader();w.writerows(records)
