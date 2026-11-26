"""Compile three frozen dictionaries from the one named training tape."""
from pathlib import Path
import hashlib,json,subprocess,sys
HERE=Path(__file__).resolve().parent
out=Path(sys.argv[1]).resolve()
out.mkdir(parents=True,exist_ok=True)
training=Path(sys.argv[2]).resolve()
repo=Path(subprocess.check_output(['git','rev-parse','--show-toplevel'],text=True).strip())
def source(rev,path):
    return subprocess.check_output(['git','show',f'{rev}:{path.relative_to(repo)}'],text=True)
proj=source('0eb27930',HERE.parent/'project.cpp')
base=source('9e232479',HERE.parents[1]/'states.cpp')
# Project the exact checked constructor; remove both validation-only mains.
i=base.index('std::vector<Key> oracle')
j=proj.index('struct Node');e=proj.index('struct Ranks')
(out/'build_patterns.cpp').write_text(base[:i]+'\n'+proj[j:e]+r'''
int main(int argc,char** argv){
    assert(argc==2);auto block=input(argv[1]);U arm=0;
    for(U t:{32,128,512}){
        for(auto& level:train(block,t)){
            std::cout<<arm;
            for(auto x:level)std::cout<<" "<<((x.parent<<8)|x.prefix);
            std::cout<<"\n";
        }
        arm++;
    }
}
''')
subprocess.run(['g++','-std=c++17','-O3',out/'build_patterns.cpp','-o',out/'train'],check=True)
data=subprocess.check_output([out/'train',training],text=True)
groups=[[] for i in range(3)]
for line in data.splitlines():
    a,*words=map(int,line.split())
    groups[a].append(words)
assert [sum(map(len,g)) for g in groups]==[30740,12879,6527]
with (out/'dictionaries.inc').open('w') as f:
    refs=[]
    for a,group in enumerate(groups):
        for level,words in enumerate(group):
            name=f'dict_{a}_{level}'
            f.write(f'static constexpr U {name}[]={{'+','.join(map(str,words))+'};\n')
        dname=f'dict_{a}'
        f.write(f'static const std::span<const U> {dname}[]={{'+','.join(f'dict_{a}_{i}' for i in range(len(group)))+'};\n')
        refs.append(dname)
    f.write('static const std::span<const std::span<const U>> all_models[]={'+','.join(refs)+'};\n')
for local in ('native/jump.hpp','native/model.hpp','../contexts/contexts.hpp'):
    suffix=HERE.parents[2] / local
    target=out/'jumps'/local
    target.parent.mkdir(exist_ok=True,parents=True)
    target.write_text(source('8e3e6732',suffix.resolve()))
subprocess.run([sys.executable,HERE.parents[2]/'native/generate.py',out/'jumps/native'],check=True)
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
print(json.dumps(dict(training_sha=sha(training),dictionaries_sha=sha(out/'dictionaries.inc'),
                      words_per_arm=[sum(map(len,g)) for g in groups],native='8e3e6732',training_source='0eb27930'),indent=2))
