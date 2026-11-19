"""Apply only the three outgoing observer events and consume the supplied point on the following step."""
from pathlib import Path
import subprocess
import shutil
import sys

here=Path(__file__).resolve().parent
parent=here.parent
prepared,build=map(lambda x:Path(x).resolve(),sys.argv[1:])
build.mkdir(exist_ok=True,parents=True)
root=Path(subprocess.check_output(['git','-C',here,'rev-parse','--show-toplevel'],text=True).strip())
frozen='d33bcaffe0000292345cf84c823a2b523cdd7409'
def old(name):
    return subprocess.check_output(['git','-C',root,'show',frozen+':'+str((parent/name).relative_to(root))]).decode()
def rep(s,a,b):
    assert s.count(a)==1,a
    return s.replace(a,b)
for file in ['native.cpp','direct_cut.cpp','interval.hpp','observed-interval.hpp']:
    shutil.copyfile(prepared/file,build/file)
s=(prepared/'observed-compact.hpp').read_text()
s=rep(s,'Root out;out.bytes.reserve(A);','Root out;out.bytes.reserve(A);observer.start_publication(A);')
s=rep(s,'out.bytes.push_back(node);','out.bytes.push_back(node);observer.publish_frequency(rem[a]);')
s=rep(s,'append(out.r,id[s.edge],s.width);','observer.emit(out.r,id[s.edge],s.edge,s.width,rem);')
s=rep(s,'append(out.r,id[p.k],take);','observer.emit(out.r,id[p.k],p.k,take,rem);')
(build/'observed-compact.hpp').write_text(s)

s=old('transport.hpp').replace('Mode==Carry::Ordinal','Mode!=Carry::Singleton')
s=rep(s,'{ Singleton, Ordinal }','{ Singleton, Ordinal, Prefix, Delayed }')
s=rep(s,'struct Mark {','''struct PublishedPoint {
    std::vector<U> base;
    U incoming=absent,predecessor=absent,piece=absent,within=0;
    bool valid=false;
};
struct Mark {''')
s=rep(s,'uint64_t moved=0,queries=0,hops=0;','''uint64_t moved=0,queries=0,hops=0;
    PublishedPoint memo;
    uint64_t histogram_runs=0,emitted_prefix=0,one_label_scanned=0;
    explicit Mark(U x):value(x){}''')
s=rep(s,'    Transport(const Root& in,Mark& m):mark(m),base(in.bytes.size()+1) {',
'''    #include "events.hpp"
    Transport(const Root& in,Mark& m):mark(m) {
        if constexpr(published) {
            auto& point=mark.memo;
            if(point.valid) {
                base=std::move(point.base);point.valid=false;
                source=U(std::upper_bound(base.begin(),base.end(),mark.value)-base.begin()-1);
                incoming=point.incoming;mark_run=point.piece;
                if constexpr(Mode==Carry::Prefix) {
                    predecessor=point.predecessor;done=predecessor==mark.value;
                } else {
                    predecessor=base[incoming]+point.within;lazy=true;
                }
                return;
            }
        }
        base.resize(in.bytes.size()+1);
#ifdef AUDIT_MARK
        mark.histogram_runs+=in.r.size();
#endif''')
s=rep(s,'        return mark.value;','        if constexpr(published)remaining=mark.value;\n        return mark.value;')
(build/'transport.hpp').write_text(s)

s=old('run.cpp').replace('Mode==Carry::Ordinal','Mode!=Carry::Singleton')
s=rep(s,'for(;i<carry.mark_run;++i)slice(in.r[i]);','''for(;i<carry.mark_run;++i) {
                const auto r=in.r[i];
                if constexpr(Mode==Carry::Delayed)
                    if(carry.lazy&&r.k==carry.incoming)carry.predecessor+=r.n;
                slice(r);
            }
            if constexpr(Mode==Carry::Delayed) {
                if(carry.lazy) {
#ifdef AUDIT_MARK
                    mark.one_label_scanned+=carry.mark_run;
#endif
                    carry.done=carry.predecessor==mark.value;
                    if(carry.done)return;
                }
            }''')
s=rep(s,'struct Record {','''struct Snapshot {std::vector<Run> column;std::vector<U> words;std::vector<Node> nodes;U ordinal,skip;};
struct Record {
#ifdef AUDIT_MARK
    std::vector<Snapshot> snapshots;
#endif
    uint64_t histogram=0,emissions=0,scanned=0;''')
s=rep(s,'    bool normalized=false;', '    bool normalized=false;')
s=rep(s,'        const auto& b=step.carry.base;','''        const auto& b=step.carry.base;
#ifdef AUDIT_MARK
        if(trace) {
            assert(b==bases(in));
            U pos=0,cnt=0;bool found=false;
            U a=step.carry.incoming;
            for(U i=0;i<in.r.size();++i) {
                auto r=in.r[i];
                if(pos+r.n>m.value) {
                    assert(i==step.carry.mark_run&&r.k==a);
                    assert(b[a]+cnt+m.value-pos==step.carry.predecessor);
                    found=true;break;
                }
                if(r.k==a)cnt+=r.n;
                pos+=r.n;
            }
            assert(found);
            stats.snapshots.push_back({in.r,in.bytes,grammar,m.value,m.skip});
        }
#endif''')
s=rep(s,'            stats.moves=m.moved;', '            stats.histogram=m.histogram_runs;stats.emissions=m.emitted_prefix;stats.scanned=m.one_label_scanned;\n            stats.moves=m.moved;')
s=rep(s,'case 0:return marked<Carry::Singleton,true>', 'case 0:return marked<Carry::Ordinal,false>')
s=rep(s,'case 1:return marked<Carry::Ordinal,true>', 'case 1:return marked<Carry::Prefix,false>')
s=rep(s,'case 2:return marked<Carry::Ordinal,false>', 'case 2:return marked<Carry::Delayed,false>')
s=rep(s,'{"singleton","split-ordinal","unseeded","dense"}','{"histogram","prefix","delayed","dense"}')
s=rep(s,'void small() {','''void compare(const Record& a,const Record& b) {
#ifdef AUDIT_MARK
    assert(a.snapshots.size()==b.snapshots.size());
    for(U i=0;i<a.snapshots.size();++i) {
        const auto& x=a.snapshots[i];const auto& y=b.snapshots[i];
        assert(x.ordinal==y.ordinal&&x.skip==y.skip&&x.column==y.column&&x.words==y.words);
        assert(x.nodes.size()==y.nodes.size());
        for(U j=0;j<x.nodes.size();++j) {
            auto p=x.nodes[j],q=y.nodes[j];
            assert(p.n==q.n&&p.l==q.l&&p.r==q.r&&p.kind==q.kind);
        }
    }
#else
    (void)a;(void)b;
#endif
}
void small() {''')
s=rep(s,'            const auto& a=records[0];', '            compare(records[0],records[1]);compare(records[0],records[2]);\n            const auto& a=records[0];')
s=rep(s,'        auto want=direct(in);\n        for(U a=0;a<3;++a)', '        auto want=direct(in);Record reference;\n        for(U a=0;a<3;++a)')
s=rep(s,'            std::cout<<names[a]<<', '            if(a)compare(reference,r);else reference=r;\n            std::cout<<r.histogram<<\',\'<<r.emissions<<\',\'<<r.scanned<<\',\'<<names[a]<<')
(build/'run.cpp').write_text(s)
for name,options in [('run',[]),('audit',['-DAUDIT_MARK'])]:
    subprocess.run(['g++','-std=c++20','-O3','-g0','-Wall','-Wextra',*options,
                    '-I',str(here),str(build/'run.cpp'),'-o',str(build/name)],check=True)
