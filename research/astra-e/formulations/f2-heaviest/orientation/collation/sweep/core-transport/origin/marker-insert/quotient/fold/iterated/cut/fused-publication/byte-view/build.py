"""Compile exact cap-view equality checks or the full bounded-input C ABI treatment."""
from pathlib import Path
import subprocess,sys,hashlib,json
here=Path(__file__).resolve().parent
raw=Path(sys.argv[1]).resolve();out=Path(sys.argv[2]).resolve();mode=sys.argv[3]
out.mkdir(exist_ok=True,parents=True)
root=Path(subprocess.check_output(['git','-C',here,'rev-parse','--show-toplevel'],text=True).strip())
rel=here.parent.relative_to(root)
caps=subprocess.check_output(['git','-C',root,'show',f'd8ce5ca:{rel}/caps.cpp'],text=True)
head=caps.split('struct PrefixStamp{')[0].replace('#include "raw-lib.inc"','#include "fusion-without-main.inc"')
(out/'capped-base.inc').write_text(head)
(out/'journal.inc').write_text('struct PrefixStamp{'+caps.split('struct PrefixStamp{',1)[1].split('extern "C" const uint8_t* grammar_run(',1)[0])
base=rel.parent/'powers'
direct=subprocess.check_output(['git','-C',root,'show',f'0dd79ba:{base}/direct_cut.cpp'],text=True)
tail=direct.split('for(U& v:in.bytes) v=index[v];\n',1)[1].split('std::vector<uint8_t> decode(',1)[0]
assert tail.endswith('}\n') and tail.startswith('    for(;;) {')
(out/'tail.inc').write_text('template<class Builder> std::vector<uint8_t> finish_seeded(Root in,std::vector<Node> nodes,U horizon,Stats& stats){\n'+tail)
early=subprocess.check_output(['git','-C',root,'show',f'63c44f6:{base}/xor/early/prefix.cpp'],text=True)
spell=early[early.index('void spell('):early.index('struct Geometry')]
rest=early[early.index('template<typename Link>'):early.index('extern "C" U prefix_chain')]
(out/'residual.inc').write_text('struct Prefix{Root state;std::vector<Node> grammar;U horizon,rows;};\n'+spell+rest)
source=here/('check.cpp' if mode=='check' else 'price.cpp');exe=out/('check' if mode=='check' else 'libgrammar.so')
cmd=['g++','-std=c++20','-O3','-g0','-Wno-return-type','-I',raw,'-I',out]
if mode=='price':cmd+=['-fPIC','-shared']
subprocess.run([*cmd,source,'-o',exe],check=True)
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
print(json.dumps({'mode':mode,'compiler':subprocess.check_output(['g++','--version'],text=True).splitlines()[0],
                  'sha256':{p.name:sha(p) for p in [exe,source,here/'matrix.hpp',out/'capped-base.inc',out/'tail.inc',out/'residual.inc']},
                  'donors':['0dd79ba','3eb3982','d8ce5ca','63c44f6']},indent=2))
