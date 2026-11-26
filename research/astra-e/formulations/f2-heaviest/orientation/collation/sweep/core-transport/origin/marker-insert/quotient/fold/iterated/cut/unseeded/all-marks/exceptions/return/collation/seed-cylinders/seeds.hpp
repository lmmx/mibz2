#pragma once
#include <algorithm>
#include <array>
#include <cassert>
#include <cstdint>
#include <numeric>
#include <vector>
using U=uint32_t;using Q=uint64_t;using Byte=uint8_t;
constexpr U none=UINT32_MAX;
struct R {U lo,w,image;Byte b;};
struct Mixed {U lo,hi;Byte mode;};
struct Seed {U lo,hi,owner,base;Byte b;};
struct Stats {
    Q seams=0,tested=0,skipped=0,pair_steps=0,success=0,fm_ranks=0,fm_cmps=0,hist_slices=0,
      physical_queries=0,physical_cmps=0,image_queries=0,image_cmps=0;
    std::array<Q,16> reject{};
    Q nodes=0,unary=0,branch=0,leaves=0,covered_rows=0,slices=0,seed_queries=0,seed_cmps=0,spelling=0,
      merge_claims=0,merge_records=0,peak_live=0,phase_steps=0,fallback=0;
};
struct Geometry {
    U n=0;
    std::vector<R> runs,image;
    std::array<std::vector<U>,256> positions;
    std::vector<U> source_page,image_page;
    std::array<U,257> base{};
    void add(Byte b,U w){
        if(!runs.empty()&&runs.back().b==b)runs.back().w+=w;
        else runs.push_back({n,w,0,b});
        n+=w;base[U(b)+1]+=w;
    }
    void finish(){
        assert(!runs.empty());
        std::partial_sum(base.begin(),base.end(),base.begin());auto at=base;
        for(U i=0;i<runs.size();++i){auto& r=runs[i];r.image=at[r.b];at[r.b]+=r.w;positions[r.b].push_back(i);}
        for(auto& bucket:positions)for(U id:bucket){auto r=runs[id];image.push_back({r.image,r.w,r.lo,r.b});}
        auto pages=[&](const std::vector<R>& list,std::vector<U>& out){
            U cur=0;out.resize((n+255)/256+1);
            for(U i=0;i<out.size();++i){U x=std::min(n-1,i*256);
                while(cur+1<list.size()&&list[cur+1].lo<=x)++cur;
                out[i]=cur;
            }
        };
        pages(runs,source_page);pages(image,image_page);
    }
    template<bool Track> U locate(const std::vector<R>& list,const std::vector<U>& page,U x,Q& count,Q& comparisons)const {
        if constexpr(Track)++count;
        U lo=page[x/256],hi=page[x/256+1]+1;
        while(lo+1<hi){U m=(lo+hi)/2;if constexpr(Track)++comparisons;if(list[m].lo<=x)lo=m;else hi=m;}
        return lo;
    }
    template<bool Track> std::pair<Byte,U> psi(U x,Stats& t)const{
        auto r=image[locate<Track>(image,image_page,x,t.image_queries,t.image_cmps)];
        return {r.b,r.image+x-r.lo};
    }
    template<bool Track> U rank_endpoint(Byte b,U x,Stats& t)const{
        if constexpr(Track)++t.fm_ranks;
        auto& pos=positions[b];U lo=0,hi=pos.size();
        while(lo<hi){U mid=(lo+hi)/2;if constexpr(Track)++t.fm_cmps;
            if(runs[pos[mid]].lo<x)lo=mid+1;else hi=mid;}
        if(!lo)return base[b];
        auto r=runs[pos[lo-1]];return r.image+std::min(r.w,x-r.lo);
    }
    template<bool Track,class Visit>void slice(U lo,U hi,Stats& t,Visit visit)const{
        U idx=locate<Track>(runs,source_page,lo,t.physical_queries,t.physical_cmps);
        for(U x=lo;x<hi;){auto r=runs[idx++];U w=std::min(hi,r.lo+r.w)-x;
            visit(r.b,r.image+x-r.lo,w);x+=w;}
    }
};

struct Section {std::vector<Mixed> mixed;std::vector<Seed> seeds;U mass=0;};
template<bool Track>Section classify(const Geometry& g,U k,Stats& t) {
    assert(k&&k<=16);
    Section out;std::array<std::vector<Seed>,256> bucket;
    std::array<U,256> count{},first{};
    std::array<Byte,256> keys;
    if constexpr(Track)t.seams=g.runs.size()-1;
    for(U seam=1;seam<g.runs.size();) {
        U x=g.runs[seam].lo-1,y=x+1;
        if constexpr(Track)++t.tested;
        std::array<Byte,16> word;U j=0;
        for(;j<k;++j){
            auto a=g.psi<Track>(x,t),b=g.psi<Track>(y,t);
            if constexpr(Track)++t.pair_steps;
            if(a.first!=b.first){if constexpr(Track)++t.reject[j];break;}
            word[j]=a.first;x=a.second;y=b.second;
        }
        if(j<k){++seam;continue;}
        U lo=0,hi=g.n;
        for(U h=k;h--;){lo=g.rank_endpoint<Track>(word[h],lo,t);hi=g.rank_endpoint<Track>(word[h],hi,t);}
        assert(lo<g.runs[seam].lo&&g.runs[seam].lo<hi);
        if constexpr(Track)++t.success;
        U distinct=0;
        g.slice<Track>(lo,hi,t,[&](Byte b,U image,U w){
            if constexpr(Track)++t.hist_slices;
            if(!count[b]){keys[distinct++]=b;first[b]=image;}
            count[b]+=w;
        });
        Byte mode=keys[0];for(U i=1;i<distinct;++i){Byte b=keys[i];if(count[b]>count[mode]||(count[b]==count[mode]&&b<mode))mode=b;}
        U owner=out.mixed.size();out.mixed.push_back({lo,hi,mode});
        assert(distinct>=2);
        for(U i=0;i<distinct;++i){Byte b=keys[i];
            if(b!=mode)bucket[b].push_back({first[b],first[b]+count[b],owner,0,b});
            count[b]=0;
        }
        ++seam;
        while(seam<g.runs.size()&&g.runs[seam].lo<hi){++seam;if constexpr(Track)++t.skipped;}
    }
    for(auto& b:bucket)for(auto v:b){
        v.base=out.mass;out.mass+=v.hi-v.lo;
        assert(out.seeds.empty()||out.seeds.back().hi<=v.lo);out.seeds.push_back(v);
    }
    if constexpr(Track)assert(t.seams==t.tested+t.skipped);
    return out;
}
