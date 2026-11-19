#pragma once
#include <algorithm>
#include <array>
#include <cassert>
#include <cstdint>
#include <numeric>
#include <vector>
namespace strips {
using U=uint32_t;using Q=uint64_t;constexpr U none=UINT32_MAX;
struct Band {U x,y,w,rule;};
struct Join {U left,right;};
struct Span {U first,count;};
struct Tally {
    U n=0,kept=0,old=0,sinks=0,sink_ranges=0,packets=0,cut_ranges=0,hit_paths=0,
      canonical=0,rep_steps=0,joins=0,split=0,empty_paths=0;
    Q reverse_states=0,reverse_queries=0,reverse_compares=0,forward_compares=0,image_sort=0,cut_sort=0;
};
struct Exchange {
    U n;std::vector<Band> b;
    std::vector<U> image,source_page,image_page;
    explicit Exchange(U size):n(size){}
    void index(Tally& t) {
        image.resize(b.size());std::iota(image.begin(),image.end(),0);
        std::sort(image.begin(),image.end(),[&](U i,U j){++t.image_sort;return b[i].y<b[j].y;});
        U src=0,dst=0;
        for(U i=0;i<b.size();++i){assert(b[i].x==src&&b[image[i]].y==dst);src+=b[i].w;dst+=b[image[i]].w;}
        assert(src==n&&dst==n);
        source_page.resize((n+255)/256+1);image_page.resize(source_page.size());U si=0,di=0;
        for(U pg=0;pg<source_page.size();++pg){U x=std::min(n-1,pg*256);
            while(si+1<b.size()&&b[si+1].x<=x)++si;
            while(di+1<b.size()&&b[image[di+1]].y<=x)++di;
            source_page[pg]=si;image_page[pg]=di;
        }
    }
    template<bool Inverse>const Band& lookup(U x,Q& comps)const {
        const auto& p=Inverse?image_page:source_page;
        U lo=p[x>>8],hi=1+p[(x>>8)+1];
        while(lo+1<hi){U mid=(lo+hi)/2;++comps;
            U at=Inverse?b[image[mid]].y:b[mid].x;
            if(at<=x)lo=mid;else hi=mid;}
        return b[Inverse?image[lo]:lo];
    }
};
inline std::vector<Span> sinks(const Exchange& e,U q,Tally& t) {
    std::vector<Span> out;
    if(e.n<=q)return out;
    U bound=e.n-q;
    for(auto b:e.b){
        U lo=b.x+(b.w>q?b.w-q:0),hi=std::min(b.x+b.w,bound);
        if(lo>=hi)continue;
        if(!out.empty()&&out.back().first+out.back().count==lo)out.back().count+=hi-lo;
        else out.push_back({lo,hi-lo});
        t.sinks+=hi-lo;
    }
    U sum=0;for(auto b:e.b)sum+=std::min(b.w,q);
    assert(t.sinks==sum-q);t.sink_ranges=out.size();return out;
}
inline std::vector<Span> boundaries(const Exchange& e,U q,U phase,bool scalar,Tally& t) {
    std::vector<Span> cuts,stack;
    auto roots=sinks(e,q,t);
    auto walk=[&](Span start){
        ++t.packets;stack.push_back(start);
        while(!stack.empty()){
            auto p=stack.back();stack.pop_back();
            for(;;) {
                if(p.first%q==phase){
                    cuts.push_back({(p.first-phase)/q,p.count});t.hit_paths+=p.count;t.reverse_states+=p.count;break;
                }
                const auto& band=e.lookup<true>(p.first,t.reverse_compares);++t.reverse_queries;
                U edge=band.y+band.w;
                bool valid=band.w>q&&p.first<edge-q;
                if(valid)edge-=q;
                U count=std::min(p.count,1+(edge-1-p.first)/q);
                assert(count&&p.first<edge);
                if(count<p.count){stack.push_back({p.first+count*q,p.count-count});++t.split;}
                t.reverse_states+=count;
                if(!valid){t.empty_paths+=count;break;}
                p={band.x+p.first-band.y,count};
            }
        }
    };
    for(auto r:roots) {
        if(scalar){for(U i=0;i<r.count;++i)walk({r.first+i,1});}
        else for(U i=0;i<std::min(q,r.count);++i)walk({r.first+i,1+(r.count-1-i)/q});
    }
    assert(t.hit_paths+t.empty_paths==t.sinks&&t.reverse_states<=e.n);
    std::sort(cuts.begin(),cuts.end(),[&](Span a,Span b){++t.cut_sort;return a.first<b.first;});
    U used=0;
    for(auto cut:cuts){
        if(used&&cuts[used-1].first+cuts[used-1].count==cut.first)cuts[used-1].count+=cut.count;
        else{assert(!used||cuts[used-1].first+cuts[used-1].count<cut.first);cuts[used++]=cut;}
    }
    cuts.resize(used);t.cut_ranges=cuts.size();return cuts;
}
inline Exchange step(Exchange& e,U q,U mark,bool scalar,std::vector<Join>& rules,Tally& t) {
    U phase=mark%q,kept=1+(e.n-1-phase)/q;
    t.n=e.n;t.kept=kept;t.old=e.b.size();e.index(t);
    auto cuts=boundaries(e,q,phase,scalar,t);Exchange next(kept);
    U first=0;
    auto add=[&](U stop) {
        assert(stop>first);U x=phase+q*first,rule=none;
        do{
            const auto a=e.lookup<false>(x,t.forward_compares);++t.rep_steps;
            if(rule==none)rule=a.rule;
            else{rules.push_back({rule,a.rule});rule=255+rules.size();++t.joins;}
            x=a.y+x-a.x;
        }while(x%q!=phase);
        next.b.push_back({first,(x-phase)/q,stop-first,rule});first=stop;
    };
    for(auto c:cuts)for(U h=0;h<c.count;++h)add(c.first+h+1);
    add(kept);t.canonical=next.b.size();
    assert(t.canonical==1+t.hit_paths&&t.canonical<=kept&&t.rep_steps<=e.n);
    return next;
}
} // namespace strips
