"""Add the compact destination-bucket identity to the frozen published-point native harness."""
from pathlib import Path
import shutil,subprocess,sys
here=Path(__file__).resolve().parent
donor,build=map(lambda x:Path(x).resolve(),sys.argv[1:])
build.mkdir(parents=True,exist_ok=True)
def rep(s,a,b):
    assert s.count(a)==1,a
    return s.replace(a,b)
for f in ('native.cpp','direct_cut.cpp','interval.hpp','observed-interval.hpp','observed-compact.hpp'):
    shutil.copyfile(donor/f,build/f)

events=(here.parent/'events.hpp').read_text().replace('Mode==Carry::Prefix || Mode==Carry::Delayed','Mode==Carry::Prefix || Mode==Carry::Delayed || Mode==Carry::Bucket')
(build/'events.hpp').write_text(events)
s=(donor/'transport.hpp').read_text().replace('Ordinal, Prefix, Delayed','Ordinal, Prefix, Delayed, Bucket')
s=rep(s,'uint64_t histogram_runs=0,emitted_prefix=0,one_label_scanned=0;','''uint64_t histogram_runs=0,emitted_prefix=0,one_label_scanned=0,local_slices=0,possible_fixed=0;''')
s=rep(s,'    Transport(const Root& in,Mark& m)', '''    U bucket_start=0,within_old=0,alias_offset=0;
    bool will_move=false;
    Transport(const Root& in,Mark& m)''')
s=rep(s,'predecessor=base[incoming]+point.within;lazy=true;', '''within_old=point.within;
                    if(Mode==Carry::Delayed||source==incoming) {
                        predecessor=base[incoming]+point.within;lazy=true;
#ifdef AUDIT_MARK
                        mark.possible_fixed+=source==incoming;
#endif
                    }''')
s=rep(s,'mark_run=i; const auto r=in.r[i];','mark_run=i;within_old=mark.value-at; const auto r=in.r[i];')
s=rep(s,'            std::for_each(parts.begin(),hit,apply);','''            if constexpr(Mode==Carry::Bucket) {
                if(will_move) {
                    auto bucket=parts.begin()+bucket_start;
                    std::for_each(parts.begin(),bucket,apply);
                    alias_offset=std::min(within_old,mark.value-base[source]);
                    for(auto i=bucket;i!=hit;++i) {
                        if(i->source==incoming)alias_offset+=i->width;
                        apply(*i);
                    }
#ifdef AUDIT_MARK
                    mark.local_slices+=mark_slice-bucket_start;
#endif
                } else std::for_each(parts.begin(),hit,apply);
            } else std::for_each(parts.begin(),hit,apply);''')
s=rep(s,'U owner=into[parts[mark_slice].edge]!=absent?incoming:source;', '''U owner=into[parts[mark_slice].edge]!=absent?(Mode==Carry::Bucket?0:incoming):source;''')
s=rep(s,'mark.value=z-gone-(moved?0:local);','''if constexpr(Mode==Carry::Bucket) {
                assert(will_move==moved);
                mark.value=moved?mark.memo.base[id[s.edge]]+alias_offset:z-gone-local;
            } else mark.value=z-gone-(moved?0:local);''')
(build/'transport.hpp').write_text(s)
s=(donor/'run.cpp').read_text().replace('if constexpr(Mode==Carry::Delayed)','if constexpr(Mode==Carry::Delayed||Mode==Carry::Bucket)')
s=rep(s,'uint64_t histogram=0,emissions=0,scanned=0;', 'uint64_t histogram=0,emissions=0,scanned=0,local=0,possible_fixed=0;')
s=rep(s,'if(flip)for(auto& col:color)col^=1;', '''if(flip)for(auto& col:color)col^=1;
        if constexpr(Mode==Carry::Bucket) {
            carry.bucket_start=offset[carry.source];
            carry.will_move=!color[carry.incoming]&&color[carry.source];
        }''')
s=rep(s,'assert(b[a]+cnt+m.value-pos==step.carry.predecessor);','''if(step.carry.predecessor!=absent)assert(b[a]+cnt+m.value-pos==step.carry.predecessor);
                    else assert(Mode==Carry::Bucket&&a!=step.carry.source);''')
s=rep(s,'stats.scanned=m.one_label_scanned;', 'stats.scanned=m.one_label_scanned;stats.local=m.local_slices;stats.possible_fixed=m.possible_fixed;')
s=rep(s,'case 1:return marked<Carry::Prefix,false>', 'case 1:return marked<Carry::Delayed,false>')
s=rep(s,'case 2:return marked<Carry::Delayed,false>', 'case 2:return marked<Carry::Bucket,false>')
s=rep(s,'{"histogram","prefix","delayed","dense"}','{"histogram","delayed","bucket","dense"}')
s=rep(s,"std::cout<<r.histogram<<','", "std::cout<<r.local<<','<<r.possible_fixed<<','<<r.histogram<<','")
(build/'run.cpp').write_text(s)
from selective import install
install(build)
for name,flags in [('run',[]),('audit',['-DAUDIT_MARK'])]:
    subprocess.run(['g++','-std=c++20','-O3','-g0','-Wall','-Wextra',*flags,str(build/'run.cpp'),'-o',str(build/name)],check=True)
