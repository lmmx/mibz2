#pragma once
#include "jump.hpp"
#include "selector.inc"

inline Input from_caps(const uint16_t* caps,U records,U n,U mark){
    Input raw;raw.n=n;raw.primary=mark;
    raw.runs.reserve(records);
    U sum=0;
    for(U i=0;i<records;i++){
        U w=caps[i]&255;Byte a=caps[i]>>8;assert(w&&w<=127);sum+=w;
        if(!raw.runs.empty()&&raw.runs.back().symbol==a)raw.runs.back().width+=w;
        else raw.runs.push_back({w,a});
    }
    assert(sum==n&&mark<n);return raw;
}
inline jump::Geometry contexts(const Input& raw,bool variable){
    jump::Geometry result;
    if(!variable){
        Charge cost;auto model=construct(raw,8,cost);
        result.ends.reserve(model.states.size);
        for(auto& bucket:model.states.end)result.ends.insert(result.ends.end(),bucket.begin(),bucket.end());
        result.modes=std::move(model.modal);result.exceptions=model.exception_rows;
        return result;
    }
    ::Geometry geo(raw);
    std::array<U,256> cursor= {};std::copy_n(geo.base.begin(),256,cursor.begin());
    selection::Input src{raw.n,{}};src.runs.reserve(raw.runs.size());
    U x=0;for(auto r:raw.runs){
        src.runs.push_back({x,x+r.width,cursor[r.symbol],r.symbol});cursor[r.symbol]+=r.width;x+=r.width;
    }
    selection::Counts cost;auto parts=selection::select(src,16,16,cost);
    result.ends.reserve(parts.size());result.modes.reserve(parts.size());
    selection::histograms(src.runs,parts,cost,[&](auto st,const auto& sums,const auto& used,U k){
        Byte mode=used[0];for(U i=1;i<k;i++){Byte a=used[i];if(sums[a].n>sums[mode].n||(sums[a].n==sums[mode].n&&a<mode))mode=a;}
        result.exceptions+=st.end-st.begin-sums[mode].n;
        result.ends.push_back(st.end);result.modes.push_back(mode);
    });
    return result;
}
