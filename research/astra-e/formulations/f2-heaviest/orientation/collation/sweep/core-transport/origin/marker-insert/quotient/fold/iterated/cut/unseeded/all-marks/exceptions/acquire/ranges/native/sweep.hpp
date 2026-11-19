#pragma once
#include "rope.hpp"
namespace ranges {
struct Ann {U x,label,w,age;};
struct Piece {U source,target,w,age;};
struct Region {U x,w,u,v;};
inline void hole_sweep(U cycle,std::vector<Ann>& u,std::vector<Ann>& v,Rope& rope,std::vector<Piece>& out) {
    if(u.empty()){assert(v.empty());return;}
    auto compare=[](Ann a,Ann b){return a.x<b.x;};std::sort(u.begin(),u.end(),compare);
    // Incoming destination anchor leaves already populate V in increasing order.
    std::vector<U> edges;edges.reserve(2*(u.size()+v.size()));
    for(auto arr:{&u,&v})for(auto z:*arr){edges.push_back(z.x);edges.push_back(z.x+z.w);}
    std::sort(edges.begin(),edges.end());edges.erase(std::unique(edges.begin(),edges.end()),edges.end());
    std::vector<Region> group;U ui=0,vi=0;int balance=0,sign=0;
    auto walk=[&]() {
        U q=0;
        for(U i=0;i<group.size();++i) {
            auto g=group[sign>0?i:group.size()-1-i];bool hasu=g.u!=none,hasv=g.v!=none;
            auto loc=[&](const Ann& a){return a.label+g.x-a.x;};
            if(hasu&&hasv){out.push_back({loc(v[g.v]),loc(u[g.u]),g.w,u[g.u].age+v[g.v].age});continue;}
            if(hasv){U x=loc(v[g.v]);int32_t id=sign>0?int32_t(x):-int32_t(x+g.w-1);
                q=rope.join(q,rope.node(-1,id,g.w,v[g.v].age));continue;}
            if(hasu){
                auto [p,rest]=rope.split(q,g.w);q=rest;U x=loc(u[g.u]);
                int32_t dst=sign>0?int32_t(x):-int32_t(x+g.w-1);
                auto emit=[&](const Node& leaf){U w=leaf.width;
                    U s=sign>0?U(leaf.first):U(-leaf.first-int32_t(w)+1);
                    U t=sign>0?U(dst):U(-dst-int32_t(w)+1);
                    out.push_back({s,t,w,leaf.age+cycle+u[g.u].age});dst+=int32_t(w);};
                rope.visit(p,emit);continue;
            }
            U h=rope.size(q);assert(h);rope.add(q,(g.w/h)*cycle);
            auto [p,rest]=rope.split(q,g.w%h);q=rope.join(rest,rope.add(p,cycle));
        }
        assert(!q);group.clear();
    };
    for(U i=1;i<edges.size();++i) {
        U l=edges[i-1],end=edges[i];
        while(ui<u.size()&&u[ui].x+u[ui].w<=l)++ui;
        while(vi<v.size()&&v[vi].x+v[vi].w<=l)++vi;
        bool a=ui<u.size()&&u[ui].x<=l,b=vi<v.size()&&v[vi].x<=l;
        int diff=int(b)-int(a);
        while(l<end) {
            U w=end-l;
            if((balance>0&&diff<0&&w>U(balance))||(balance<0&&diff>0&&w>U(-balance)))w=U(std::abs(balance));
            int before=balance;balance+=diff*int(w);
            if(!before&&!balance){
                if(a&&b)out.push_back({v[vi].label+l-v[vi].x,u[ui].label+l-u[ui].x,w,v[vi].age+u[ui].age});
            }else{
                if(group.empty())sign=balance>0?1:-1;
                group.push_back({l,w,a?ui:none,b?vi:none});if(!balance)walk();
            }
            l+=w;
        }
    }
    assert(!balance && group.empty());
}
} // namespace ranges
