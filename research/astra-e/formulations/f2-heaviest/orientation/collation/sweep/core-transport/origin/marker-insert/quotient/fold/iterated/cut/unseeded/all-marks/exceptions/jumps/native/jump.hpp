#pragma once
#include "../../contexts/contexts.hpp"
#include <bit>
#include <cstring>
#include <immintrin.h>

namespace jump {
// Each physical component carries its exclusive end and its chosen predecessor byte.
struct Geometry {
    std::vector<U> ends;
    std::vector<Byte> modes;
    U exceptions=0;
    size_t bytes()const{return 4*ends.capacity()+modes.capacity();}
    U owner(U x)const{return std::upper_bound(ends.begin(),ends.end(),x)-ends.begin();}
};
using Word=uint64_t;
inline Word low(U bits){return bits==64?~Word(0):(Word(1)<<bits)-1;}
struct Mask{
    std::vector<Word>w;
    std::vector<U>prefix;
    explicit Mask(U n=0):w((n+63)/64+2,0){}
    void finish(){
        prefix.resize(w.size());U sum=0;
        for(size_t i=0;i<w.size();i++){
            prefix[i]=sum;sum+=std::popcount(w[i]);
        }
    }
    bool test(U x)const{return (w[x/64]>>(x%64))&1;}
    U before(U x)const{return prefix[x/64]+std::popcount(w[x/64]&low(x%64));}
    Word read(U x,U length)const{
        U i=x/64,k=x%64;
        Word a=w[i]>>k;
        if(k && length>64-k)a|=w[i+1]<<(64-k);
        return a&low(length);
    }
    void fill(U begin,U end){
        while(begin<end){
            U bit=begin%64,len=std::min(end-begin,64-bit);
            w[begin/64]|=low(len)<<bit;begin+=len;
        }
    }
    size_t bytes()const{return 8*w.capacity()+4*prefix.capacity();}
};
struct Link{U bias,post;Word spelling;};
static_assert(sizeof(Link)==16);
struct Layer{
    Mask mask;
    std::vector<Link> link;
    explicit Layer(U n=0,U c=0):mask(n),link(c){}
    size_t bytes()const{return mask.bytes()+link.capacity()*sizeof(Link);}
};
struct Exception{U dest,edge;};
struct Index{
    U n=0,primary_state=0;
    Layer one,eight;
    std::vector<Exception> exceptions;
    size_t construction_peak=0;
    size_t bytes()const{return one.bytes()+eight.bytes()+sizeof(Exception)*exceptions.capacity();}
};
inline Layer twice(const Layer& layer,U h,U n,const Geometry& g){
    Layer result(n,g.ends.size());
    U s=0,good_before=0;
    for(U c=0;c<g.ends.size();c++){
        U end=g.ends[c];const auto& first=layer.link[c];const auto& second=layer.link[first.post];
        U target=first.bias+layer.mask.before(s);
        U target2=second.bias+layer.mask.before(target);
        result.link[c]={target2-good_before,second.post,(first.spelling<<(8*h))|second.spelling};
        for(U x=s;x<end;){
            U bit=x%64,len=std::min(end-x,64-bit);
            Word slots=layer.mask.w[x/64]&(low(len)<<bit);
            U taken=std::popcount(slots);
            if(taken){
                Word dst=layer.mask.read(target,taken);
                Word live=_pdep_u64(dst,slots);
                result.mask.w[x/64]|=live;good_before+=std::popcount(live);
                target+=taken;
            }
            x+=len;
        }
        s=end;
    }
    result.mask.finish();
    assert(result.mask.before(n)==good_before);
    return result;
}
inline Index build(const Input& raw,const Geometry& g){
    assert(raw.n>0&&raw.primary<raw.n&&!g.ends.empty()&&g.ends.back()==raw.n);
    std::array<U,256> count{},after{},cursor{};
    for(auto r:raw.runs)count[r.symbol]+=r.width;
    U base=0;
    for(U a=0;a<256;a++){
        after[a]=base;if(count[a])cursor[a]=g.owner(base);base+=count[a];
    }
    Index idx;idx.n=raw.n;idx.primary_state=g.owner(raw.primary);idx.one=Layer(raw.n,g.ends.size());idx.exceptions.reserve(g.exceptions);
    U begin=0,good=0;
    Cursor walk{0,raw.runs.front().width};
    for(U c=0;c<g.ends.size();c++){
        U end=g.ends[c],bias=0,post=0;Byte mode=g.modes[c];bool found=false;
        intersect(raw,walk,begin,end,[&](Byte a,U x,U y){
            U at=after[a],w=y-x,target=cursor[a];
            while(g.ends[target]<=at)++target;
            cursor[a]=target;
            assert(at+w<=g.ends[target]);
            if(a==mode){
                if(!found){bias=at-good;post=target;found=true;}
                else{assert(target==post);}
                idx.one.mask.fill(x,y);good+=w;
            }else{
                for(U j=0;j<w;j++)idx.exceptions.push_back({at+j,(target<<8)|a});
            }
            after[a]+=w;
        });
        assert(found);idx.one.link[c]={bias,post,mode};begin=end;
    }
    assert(idx.exceptions.size()==g.exceptions&&good+idx.exceptions.size()==raw.n);
    idx.one.mask.finish();
    auto two=twice(idx.one,1,raw.n,g);
    auto four=twice(two,2,raw.n,g);
    idx.construction_peak=idx.one.bytes()+two.bytes()+four.bytes()+sizeof(Exception)*idx.exceptions.capacity();
    two=Layer{};
    idx.eight=twice(four,4,raw.n,g);
    idx.construction_peak=std::max(idx.construction_peak,idx.bytes()+four.bytes());
    return idx;
}
struct CursorState{U x,c;};
inline Byte unit(const Index& idx,CursorState& p){
    auto& one=idx.one;U r=one.mask.before(p.x);
    if(one.mask.test(p.x)){
        auto& tr=one.link[p.c];p={tr.bias+r,tr.post};return Byte(tr.spelling);
    }
    auto e=idx.exceptions[p.x-r];p={e.dest,e.edge>>8};return Byte(e.edge);
}
inline U emit(const Index& idx,U primary,Byte* out){
    U left=idx.n;CursorState p{primary,idx.primary_state};
    while(left>=8){
        const auto& j=idx.eight;
        if(j.mask.test(p.x)){
            const auto& tr=j.link[p.c];left-=8;
            std::memcpy(out+left,&tr.spelling,8);
            p={tr.bias+j.mask.before(p.x),tr.post};
        }else{out[--left]=unit(idx,p);}
    }
    while(left){out[--left]=unit(idx,p);}
    if(p.x==primary)return 0;
    // The last d bytes are one whole lap from p in forward order, for its true period d.
    U period=0; p={primary,idx.primary_state};
    do {unit(idx,p);++period;assert(period<=idx.n);} while(p.x!=primary);
    std::memmove(out,out+idx.n-period,period);
    for(U filled=period;filled<idx.n;){U n=std::min(filled,idx.n-filled);std::memcpy(out+filled,out,n);filled+=n;}
    return period;
}
}
