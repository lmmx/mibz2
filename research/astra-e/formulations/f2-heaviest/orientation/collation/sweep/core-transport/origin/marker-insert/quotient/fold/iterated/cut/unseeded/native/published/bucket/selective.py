"""Specialize the final emission of one old marked slice, retaining the original path for all other slices."""
from pathlib import Path

def rep(s,a,b):
    assert s.count(a)==1,a
    return s.replace(a,b)

def install(build):
    p=build/'events.hpp'
    s=p.read_text().replace(' || Mode==Carry::Bucket',' || Mode==Carry::Bucket || Mode==Carry::Local')
    s=rep(s,'bool lazy=false;', '''static constexpr bool selective=Mode==Carry::Local;
bool lazy=false;
U predecessor_bucket=0,emit_slice_index=0,emit_within=0;
''')
    p.write_text(s)
    p=build/'transport.hpp'
    s=p.read_text().replace('Prefix, Delayed, Bucket','Prefix, Delayed, Bucket, Local')
    s=s.replace('Mode==Carry::Bucket','(Mode==Carry::Bucket||Mode==Carry::Local)')
    s=rep(s,'auto s=parts[mark_slice];','auto s=parts[mark_slice];\n        const U old=mark.value;')
    s=rep(s,'if constexpr(published)remaining=mark.value;', '''if constexpr(selective) {
            if(moved) {
                U rank=mark.value-mark.memo.base[id[first[incoming]]];
                U i=predecessor_bucket;
                while(rank>=parts[i].width){rank-=parts[i].width;++i;}
                assert(parts[i].target==incoming);
                emit_slice_index=i;emit_within=rank;
#ifdef AUDIT_MARK
                mark.locate_slices+=i-predecessor_bucket+1;
#endif
            } else {
                emit_slice_index=mark_slice;emit_within=std::min(within_old,old-base[source]);
            }
        } else if constexpr(published)remaining=mark.value;''')
    s=rep(s,'local_slices=0,possible_fixed=0;', 'local_slices=0,possible_fixed=0,locate_slices=0;')
    p.write_text(s)
    p=build/'run.cpp';s=p.read_text().replace('Mode==Carry::Bucket','(Mode==Carry::Bucket||Mode==Carry::Local)')
    s=rep(s,'carry.bucket_start=offset[carry.source];','carry.bucket_start=offset[carry.source];carry.predecessor_bucket=offset[carry.incoming];')
    s=rep(s,'uint64_t histogram=0,emissions=0,scanned=0,local=0,possible_fixed=0;','uint64_t histogram=0,emissions=0,scanned=0,local=0,possible_fixed=0,located=0;')
    s=rep(s,'stats.possible_fixed=m.possible_fixed;', 'stats.possible_fixed=m.possible_fixed;stats.located=m.locate_slices;')
    s=rep(s,'case 0:return marked<Carry::Ordinal,false>', 'case 0:return marked<Carry::Delayed,false>')
    s=rep(s,'case 1:return marked<Carry::Delayed,false>', 'case 1:return marked<Carry::Bucket,false>')
    s=rep(s,'case 2:return marked<Carry::Bucket,false>', 'case 2:return marked<Carry::Local,false>')
    s=rep(s,'{"histogram","delayed","bucket","dense"}','{"delayed","bucket","local","dense"}')
    s=rep(s,"std::cout<<r.local<<','", "std::cout<<r.located<<','<<r.local<<','")
    p.write_text(s)
    p=build/'observed-compact.hpp';s=p.read_text()
    a=s.index('        for(auto s:parts) {\n            if(into[s.edge]!=no)continue;')
    b=s.index('        return out;',a)
    block=s[a:b]
    block=rep(block,'        for(auto s:parts) {','''        auto publish_slice = [&]<bool Watch>(Slice s) {
            auto put = [&](U a,U n) {
                if constexpr(Watch)observer.emit(out.r,id[a],a,n,rem);
                else append(out.r,id[a],n);
            };''')
    block=block.replace('continue;','return;').replace('observer.emit(out.r,id[s.edge],s.edge,s.width,rem);','put(s.edge,s.width);').replace('observer.emit(out.r,id[p.k],p.k,take,rem);','put(p.k,take);')
    assert block.endswith('        }\n')
    block=block[:-10]+'''        };
        if constexpr(Observer::selective) {
            U hit=observer.emit_slice_index, i=0;
            for(;i<hit;++i)publish_slice.template operator()<false>(parts[i]);
            observer.remaining=observer.emit_within;
            publish_slice.template operator()<true>(parts[i++]);
            assert(observer.remaining==absent&&observer.mark.memo.valid);
            for(;i<parts.size();++i)publish_slice.template operator()<false>(parts[i]);
        } else {
            for(auto s:parts)publish_slice.template operator()<true>(s);
        }
'''
    p.write_text(s[:a]+block+s[b:])
