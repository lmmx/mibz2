#pragma once
#include <algorithm>
#include <cassert>
#include <cstdint>
#include <utility>
#include <vector>

namespace ranges {
using U=uint32_t;
constexpr U none=UINT32_MAX;
struct Node {
    U left=0,right=0,size=0,width=0,age=0,lazy=0;
    int32_t first=0,kind=-1;
    uint64_t priority=0;
};
struct Rope {
    std::vector<Node> nodes;
    uint64_t sequence=0;
    void reset(size_t reserve) {
        nodes.clear();nodes.reserve(reserve);nodes.emplace_back();sequence=10;
    }
    uint64_t random() {
        // SplitMix64 priorities are independent of the input values; the fixed
        // seed makes this reproducible. No deterministic height bound is claimed.
        uint64_t z=(sequence+=0x9e3779b97f4a7c15ULL);
        z=(z^(z>>30))*0xbf58476d1ce4e5b9ULL;z=(z^(z>>27))*0x94d049bb133111ebULL;return z^(z>>31);
    }
    U node(int32_t kind,int32_t first,U width,U age) {
        assert(width);nodes.push_back({0,0,width,width,age,0,first,kind,random()});return U(nodes.size()-1);
    }
    U size(U t)const {return nodes[t].size;}
    U add(U t,U delta) {
        if(t){nodes[t].age+=delta;nodes[t].lazy+=delta;}return t;
    }
    void push(U t) {auto& v=nodes[t];if(v.lazy){add(v.left,v.lazy);add(v.right,v.lazy);v.lazy=0;}}
    U update(U t){auto& v=nodes[t];v.size=size(v.left)+v.width+size(v.right);return t;}
    U join(U a,U b) {
        if(!a)return b;if(!b)return a;
        if(nodes[a].priority>nodes[b].priority){push(a);nodes[a].right=join(nodes[a].right,b);return update(a);}
        push(b);nodes[b].left=join(a,nodes[b].left);return update(b);
    }
    std::pair<U,U> boundary(U t,U k) {
        if(!k)return {0,t};if(k==size(t))return {t,0};
        push(t);U l=size(nodes[t].left);
        if(k<=l){auto [a,b]=boundary(nodes[t].left,k);nodes[t].left=b;return {a,update(t)};}
        assert(k>=l+nodes[t].width);
        auto [a,b]=boundary(nodes[t].right,k-l-nodes[t].width);nodes[t].right=a;return {update(t),b};
    }
    std::pair<U,U> split(U t,U k) {
        assert(k<=size(t));if(!k)return {0,t};if(k==size(t))return {t,0};
        U p=t,at=0;
        for(;;) {
            push(p);U begin=at+size(nodes[p].left),end=begin+nodes[p].width;
            if(k==begin||k==end)return boundary(t,k);
            if(k<begin){p=nodes[p].left;continue;}
            if(k>end){at=end;p=nodes[p].right;continue;}
            auto [front,rem]=boundary(t,begin);auto [one,tail]=boundary(rem,end-begin);
            auto old=nodes[one];assert(one==p && !old.left && !old.right);
            U left=k-begin,right=end-k;
            nodes[one]={0,0,left,left,old.age,0,old.first,old.kind,random()};
            U other=node(old.kind,old.first+int32_t(left),right,old.age);
            return {join(front,one),join(other,tail)};
        }
    }
    template<class F> void visit(U t,F& fn) {
        if(!t)return;push(t);visit(nodes[t].left,fn);fn(nodes[t]);visit(nodes[t].right,fn);
    }
};
} // namespace ranges
