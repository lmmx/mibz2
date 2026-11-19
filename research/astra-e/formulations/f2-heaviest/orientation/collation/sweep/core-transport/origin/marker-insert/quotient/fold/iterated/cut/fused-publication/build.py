from pathlib import Path
import hashlib,subprocess,sys
here=Path(__file__).resolve().parent;out=Path(sys.argv[1]).resolve();out.mkdir(parents=True,exist_ok=True)
root=Path(subprocess.check_output(['git','-C',here,'rev-parse','--show-toplevel'],text=True).strip())
base=(here.parent/'powers').relative_to(root);rev='0dd79ba81bb5474d60f113e3de82c027f2351d50'
for name in ['xor/route.cpp','direct_cut.cpp','interval.hpp','native.cpp']:
    p=out/name;p.parent.mkdir(parents=True,exist_ok=True)
    p.write_bytes(subprocess.check_output(['git','-C',root,'show',f'{rev}:{base/name}']))
p=out/'xor/stream.hpp'
p.write_bytes(subprocess.check_output(['git','-C',root,'show',f'37c3bfd:{base}/xor/stream.hpp']))
(out/'stream-fused.hpp').write_bytes((here/'stream.hpp').read_bytes())
(out/'fusion.cpp').write_bytes((here/'fusion.cpp').read_bytes())
subprocess.run(['g++','-std=c++20','-O3','-g0','-Wall','-Wextra','-Wno-return-type',out/'fusion.cpp','-o',out/'fused'],check=True)
