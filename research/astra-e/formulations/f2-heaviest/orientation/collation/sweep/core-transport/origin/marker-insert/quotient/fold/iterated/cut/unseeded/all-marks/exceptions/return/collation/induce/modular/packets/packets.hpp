#pragma once
#include "../modular.hpp"
namespace packets {
using namespace modular;
struct Strip {U end,dest,rule;};
struct Stats{
    U n,kept,input,pieces=0,fragments=0,joins=0,outputs=0,queries=0,probes=0,peak_pending=0,max_depth=0;
    uint64_t visits=0;
};
struct Layout{
    std::vector<Strip> strips;
    std::vector<U> pages;
    static constexpr U page=64;
    U find(U y,Stats& s)const{
        ++s.queries;
        U i=pages[y/page],hi=pages[y/page+1];
        while(i<hi){
            ++s.probes;U mid=i+(hi-i)/2;
            if(strips[mid].end<=y)i=mid+1;else hi=mid;
        }
        assert(y<strips[i].end);return i;
    }
};
inline Layout build(const Exchange& e,U q,U n,U mark){
    // Give each old strip offset modulo q a temporary ID. Both orders
    // refer to that same offset, even when their starting residues differ.
    struct Part {U width,rule,dest;};
    std::vector<Part> parts;std::vector<U> first(e.arcs.size());
    std::vector<std::vector<U>> top(q),bottom(q);
    U x=0;
    for(U id:e.top){
        auto a=e.arcs[id];first[id]=parts.size();
        for(U i=0;i<std::min(q,a.width);++i){
            U k=parts.size();parts.push_back({1+(a.width-1-i)/q,a.rule,0});
            top[(x+i)%q].push_back(k);
        }
        x+=a.width;
    }
    assert(x==n);x=0;
    for(U id:e.bottom){
        U w=e.arcs[id].width;
        for(U i=0;i<std::min(q,w);++i)bottom[(x+i)%q].push_back(first[id]+i);
        x+=w;
    }
    assert(x==n);x=0;U phase=mark%q;
    for(U j=0;j<q;++j)for(U k:bottom[(phase+j)%q]){parts[k].dest=x;x+=parts[k].width;}
    Layout l;x=0;l.strips.reserve(parts.size());
    for(U j=0;j<q;++j)for(U k:top[(phase+j)%q]){
        auto a=parts[k];x+=a.width;l.strips.push_back({x,a.dest,a.rule});
    }
    U a=0;
    for(U y=0;y<n;y+=Layout::page){
        while(l.strips[a].end<=y)++a;
        l.pages.push_back(a);
    }
    l.pages.push_back(l.strips.size()-1);
    return l;
}
struct Task{U y,width,rule,depth;};
inline Exchange reduce(const Layout& l,Tower& tree,Stats& s){
    Exchange next;std::vector<U> dest_order(s.kept,none);
    std::vector<Task> pending{{0,s.kept,none,0}};
    U source=0;
    while(!pending.empty()){
        Task v=pending.back();pending.pop_back();
        for(;;){
            U i=l.find(v.y,s);
            U start=i?l.strips[i-1].end:0;
            auto a=l.strips[i];
            U w=std::min(v.width,a.end-v.y);
            if(w<v.width){
                pending.push_back({v.y+w,v.width-w,v.rule,v.depth});
                s.peak_pending=std::max(s.peak_pending,U(pending.size()));
            }
            U y=a.dest+v.y-start;
            U rule=a.rule;
            if(v.rule!=none){
                rule=256+tree.grammar.size();tree.grammar.push_back({v.rule,a.rule});s.joins++;
            }
            ++s.fragments;s.visits+=w;s.max_depth=std::max(s.max_depth,v.depth+1);
            if(y<s.kept){
                assert(y+w<=s.kept&&dest_order[y]==none);U id=next.arcs.size();
                next.arcs.push_back({w,rule});next.top.push_back(id);dest_order[y]=id;
                source+=w;break;
            }
            v={y,w,rule,v.depth+1};
        }
    }
    assert(source==s.kept&&s.visits<=s.n&&s.fragments<=s.visits);
    for(U id:dest_order)if(id!=none)next.bottom.push_back(id);
    s.outputs=next.arcs.size();return next;
}
inline Tower compute(const std::vector<Run>& runs,U mark,U q,std::vector<Stats>& rows){
    U n=runs.back().end;auto e=initial(runs);Tower tree;
    while(n>1){
        U kept=1+(n-1-mark%q)/q;
        Stats s{n,kept,U(e.arcs.size())};
        auto l=build(e,q,n,mark);s.pieces=l.strips.size();e=reduce(l,tree,s);
        rows.push_back(s);n=kept;mark/=q;
    }
    assert(e.arcs.size()==1&&e.arcs[0].width==1);tree.terminal=e.arcs[0].rule;
    return tree;
}
}
