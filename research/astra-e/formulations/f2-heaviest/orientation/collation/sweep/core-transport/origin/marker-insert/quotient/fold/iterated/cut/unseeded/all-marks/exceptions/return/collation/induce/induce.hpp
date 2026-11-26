#pragma once
#include <algorithm>
#include <array>
#include <cassert>
#include <cstdint>
#include <limits>
#include <numeric>
#include <vector>
namespace induction {
using U=uint32_t;
constexpr U none=std::numeric_limits<U>::max();
struct Run{U begin,end,base,byte;};
struct Span{U end;bool retained;};
struct Arc{U width,rule;};
struct Join{U left,right;};
struct Order{
    struct Links{U prev=none,next=none;};
    std::vector<Links> links;
    U head=none,tail=none;
    explicit Order(size_t n):links(n){}
    void append(U a){
        links[a]={tail,none};
        if(tail!=none)links[tail].next=a;else head=a;
        tail=a;
    }
    void erase(U a){
        auto [p,n]=links[a];
        if(p==none)head=n;else links[p].next=n;
        if(n==none)tail=p;else links[n].prev=p;
    }
    void after(U a,U x){
        U n=links[a].next;
        links[x]={a,n};links[a].next=x;
        if(n==none)tail=x;else links[n].prev=x;
    }
    void replace(U a,U x){
        auto [p,n]=links[a];links[x]={p,n};
        if(p==none)head=x;else links[p].next=x;
        if(n==none)tail=x;else links[n].prev=x;
    }
};
struct Record{U source,target,width,rule;};
struct Result{
    U initial=0,seams=0,kept=0,strict=0,equal=0,self=0,partial=0;
    std::vector<Join> grammar;
    std::vector<Record> records;
};
inline Result induce(const std::vector<Run>& runs,const std::vector<Span>& spans){
    Result res;U n=spans.back().end,begin=0;
    res.seams=spans.size()-1;
    for(auto s:spans){if(s.retained)res.kept+=s.end-begin;begin=s.end;}
    assert(res.kept);
    std::vector<Arc> arcs;
    std::array<std::vector<U>,2> top_groups;
    std::array<std::array<std::vector<U>,256>,2> bottom_groups;
    std::array<U,256> target{};
    std::array<bool,256> started{};
    U src=0;
    for(auto r:runs){
        U b=r.byte;
        if(!started[b]){
            target[b]=std::lower_bound(spans.begin(),spans.end(),r.base+1,
                [](Span s,U q){return s.end<q;})-spans.begin();
            started[b]=true;
        }
        for(U x=r.begin;x<r.end;){
            U y=r.base+x-r.begin;
            while(spans[target[b]].end<=y)++target[b];
            U w=std::min({r.end-x,spans[src].end-x,spans[target[b]].end-y});
            assert(w);U a=arcs.size();arcs.push_back({w,b});
            top_groups[!spans[src].retained].push_back(a);
            bottom_groups[!spans[target[b]].retained][b].push_back(a);
            x+=w;if(x==spans[src].end)++src;
        }
    }
    res.initial=arcs.size();assert(res.initial<=runs.size()+2*res.seams);
    Order top(arcs.size()+1),bottom(arcs.size()+1);
    for(auto& v:top_groups)for(U a:v)top.append(a);
    for(auto& buckets:bottom_groups)for(auto& v:buckets)for(U a:v)bottom.append(a);
    U remaining=n-res.kept;
    while(remaining){
        U t=top.tail,b=bottom.tail,tw=arcs[t].width,bw=arcs[b].width;
        U w=std::min({tw,bw,remaining});assert(w);
        remaining-=w;
        if(t==b){
            ++res.self;
            if(w==tw){top.erase(t);bottom.erase(t);}
            else arcs[t].width-=w;
            continue;
        }
        U rule=256+res.grammar.size();
        res.grammar.push_back({arcs[b].rule,arcs[t].rule});
        if(w<tw&&w<bw){
            ++res.partial;assert(!remaining);
            U a=arcs.size();arcs.push_back({w,rule});
            arcs[t].width-=w;arcs[b].width-=w;
            top.after(b,a);bottom.after(t,a);
        }else if(w==tw&&w==bw){
            ++res.equal;top.erase(t);bottom.erase(b);bottom.replace(t,b);arcs[b].rule=rule;
        }else if(w==bw){
            ++res.strict;arcs[t].width-=w;arcs[b].rule=rule;
            bottom.erase(b);bottom.after(t,b);
        }else{
            ++res.strict;arcs[b].width-=w;arcs[t].rule=rule;
            top.erase(t);top.after(b,t);
        }
    }
    std::vector<U> base(arcs.size());U x=0;
    for(U a=bottom.head;a!=none;a=bottom.links[a].next){base[a]=x;x+=arcs[a].width;}
    assert(x==res.kept);x=0;
    for(U a=top.head;a!=none;a=top.links[a].next){
        res.records.push_back({x,base[a],arcs[a].width,arcs[a].rule});x+=arcs[a].width;
    }
    assert(x==res.kept);return res;
}
}
