"""Build the raw variant against the fixed source used in the two first panels."""
import hashlib, json, subprocess, sys
from pathlib import Path
here = Path(__file__).resolve().parent
out = Path(sys.argv[1]).resolve()
out.mkdir(exist_ok=True, parents=True)
root = Path(subprocess.check_output(['git', '-C',here,'rev-parse','--show-toplevel'],text=True).strip())
rel = here.relative_to(root)
def load(revision, source, target):
    dest = out / target
    dest.parent.mkdir(parents=True, exist_ok=True)
    dest.write_bytes(subprocess.check_output(['git','-C',root,'show',f'{revision}:{source}']))
base = rel.parent / 'powers'
for name in ['xor/route.cpp','direct_cut.cpp','interval.hpp','native.cpp']:
    load('0dd79ba81bb5474d60f113e3de82c027f2351d50',base / name,name)
load('37c3bfd',base/'xor/stream.hpp','xor/stream.hpp')
load('3eb3982',rel/'stream.hpp','stream-fused.hpp')
# Replace a single main definition instead of using a compiler symbol macro:
# the donor already includes and demotes two historical executable mains.
s = subprocess.check_output(['git','-C',root,'show',f'3eb3982:{rel}/fusion.cpp'],text=True)
assert s.count('int main(int argc,char**argv)')==1
(out/'fusion-without-main.inc').write_text(s.replace('int main(int argc,char**argv)','int original_fusion_main(int argc,char**argv)'))
(out/'raw.cpp').write_bytes((here/'raw.cpp').read_bytes())
flags = ['-std=c++20','-O3','-g0','-Wall','-Wextra','-Wno-return-type']
subprocess.run(['g++',*flags,out/'raw.cpp','-o',out/'raw'],check=True)
files = [p for p in out.rglob('*') if p.is_file() and (p.suffix in {'.cpp','.hpp','.inc'} or p.name=='raw')]
print(json.dumps({'flags':flags,'compiler':subprocess.check_output(['g++','--version'],text=True).splitlines()[0],
                  'source_sha256':{str(p.relative_to(out)):hashlib.sha256(p.read_bytes()).hexdigest() for p in files}},indent=2))
