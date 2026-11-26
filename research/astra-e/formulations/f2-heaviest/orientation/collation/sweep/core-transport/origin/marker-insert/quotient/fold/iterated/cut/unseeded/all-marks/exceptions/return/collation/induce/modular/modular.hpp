#pragma once
#include "../induce.hpp"
namespace modular {
using namespace induction;
struct Exchange {std::vector<Arc> arcs;std::vector<U> top,bottom;};
struct Layer {U n,kept,old_bands,pieces,strict=0,equal=0,self=0,partial=0,final_bands=0;};
struct Tower{std::vector<Join> grammar;std::vector<Layer> layers;U terminal=0;};
inline Exchange initial(const std::vector<Run>& runs){
    Exchange e;std::array<std::vector<U>,256> by_byte;
    for(auto r:runs){
        U id=e.arcs.size();e.top.push_back(id);e.arcs.push_back({r.end-r.begin,r.byte});
        by_byte[r.byte].push_back(id);
    }
    for(auto& v:by_byte)e.bottom.insert(e.bottom.end(),v.begin(),v.end());
    return e;
}
inline Exchange step(const Exchange& before,U q,U n,U mark,Tower& tree){
    U phase=mark%q;
    std::vector<std::vector<U>> topq(q),bottomq(q);
    std::vector<Arc> arcs;std::vector<U> start(before.arcs.size());
    U x=0;
    for(U id:before.top){
        auto a=before.arcs[id];start[id]=arcs.size();
        for(U i=0;i<std::min(q,a.width);++i){
            U count=1+(a.width-1-i)/q,nid=arcs.size();
            topq[(x+i)%q].push_back(nid);arcs.push_back({count,a.rule});
        }
        x+=a.width;
    }
    assert(x==n);x=0;
    for(U id:before.bottom){
        U w=before.arcs[id].width;
        for(U i=0;i<std::min(q,w);++i)bottomq[(x+i)%q].push_back(start[id]+i);
        x+=w;
    }
    assert(x==n);
    Order top(arcs.size()+1),bottom(arcs.size()+1);
    for(U j=0;j<q;++j){
        U r=(phase+j)%q;
        for(U a:topq[r])top.append(a);
        for(U a:bottomq[r])bottom.append(a);
    }
    U kept=1+(n-1-phase)/q,left=n-kept;
    Layer level{n,kept,U(before.arcs.size()),U(arcs.size())};
    while(left){
        U a=top.tail,b=bottom.tail,wa=arcs[a].width,wb=arcs[b].width;
        U w=std::min({wa,wb,left});assert(w);left-=w;
        if(a==b){
            ++level.self;
            if(w==wa){top.erase(a);bottom.erase(a);}else arcs[a].width-=w;
            continue;
        }
        U spell=256+tree.grammar.size();
        tree.grammar.push_back({arcs[b].rule,arcs[a].rule});
        if(w<wa&&w<wb){
            ++level.partial;assert(!left);
            U c=arcs.size();arcs.push_back({w,spell});
            arcs[a].width-=w;arcs[b].width-=w;top.after(b,c);bottom.after(a,c);
        }else if(wa==wb){
            ++level.equal;top.erase(a);bottom.erase(b);bottom.replace(a,b);arcs[b].rule=spell;
        }else if(wa>wb){
            ++level.strict;arcs[a].width-=w;arcs[b].rule=spell;
            bottom.erase(b);bottom.after(a,b);
        }else{
            ++level.strict;arcs[b].width-=w;arcs[a].rule=spell;
            top.erase(a);top.after(b,a);
        }
    }
    Exchange next;std::vector<U> name(arcs.size());
    for(U i=top.head;i!=none;i=top.links[i].next){
        name[i]=next.arcs.size();next.top.push_back(name[i]);next.arcs.push_back(arcs[i]);
    }
    for(U i=bottom.head;i!=none;i=bottom.links[i].next)next.bottom.push_back(name[i]);
    level.final_bands=next.arcs.size();tree.layers.push_back(level);return next;
}
inline Tower induce(const std::vector<Run>& runs,U mark,U q){
    assert(q>=2);auto e=initial(runs);U n=runs.back().end;
    Tower tree;
    while(n>1){
        e=step(e,q,n,mark,tree);n=tree.layers.back().kept;mark/=q;
        assert(mark<n);
    }
    assert(mark==0&&e.arcs.size()==1&&e.arcs[0].width==1);
    tree.terminal=e.arcs[0].rule;return tree;
}
struct Emit {U lap=0,visits=0,copies=0,aliases=0,repeats=0;};
inline Emit spell(const Tower& tree,unsigned char* output,U n){
    std::vector<U> first(tree.grammar.size(),none),lengths;
    lengths.reserve(tree.grammar.size());
    auto length=[&](U id){return id<256?1u:lengths[id-256];};
    for(auto [a,b]:tree.grammar)lengths.push_back(length(a)+length(b));
    Emit e;std::vector<U> stack{tree.terminal};
    while(!stack.empty()){
        U id=stack.back();stack.pop_back();++e.visits;
        if(id<256){output[e.lap++]=id;continue;}
        U& address=first[id-256];
        if(address==none){
            address=e.lap;auto [a,b]=tree.grammar[id-256];stack.push_back(a);stack.push_back(b);
        }else{
            U w=length(id);assert(address+w<=e.lap&&e.lap+w<=n);
            std::copy_n(output+address,w,output+e.lap);e.lap+=w;++e.copies;e.aliases+=w;
        }
    }
    assert(e.lap&&e.lap<=n);
    for(U written=e.lap;written<n;){
        U w=std::min(written,n-written);std::copy_n(output,w,output+written);written+=w;++e.repeats;
    }
    return e;
}
}
