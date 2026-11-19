#pragma once
#include "seeds.hpp"
struct Part {U id,w;};
struct Literal {U base,width,offset,weight;};
struct Child {
    std::vector<Literal> classes;
    std::vector<Byte> dictionary;
    std::vector<Part> incoming;
    U origin=none,skip=0,default_lap=0;
    std::vector<Byte> pure;
};
bool append_run(std::vector<Part>& list,U id,U w){
    if(!list.empty()&&list.back().id==id){list.back().w+=w;return false;}
    list.push_back({id,w});return true;
}
struct SectionIndex {
    const std::vector<Seed>& seeds;std::vector<U> page;
    SectionIndex(const Section& sec,U n):seeds(sec.seeds),page((n+255)/256+1){
        U i=0;for(U j=0;j<page.size();++j){
            U x=std::min(j*256,n);while(i<seeds.size()&&seeds[i].hi<=x)++i;page[j]=i;
        }
    }
    template<bool Track>U containing(U x,U hi,Stats& t)const{
        if constexpr(Track)++t.seed_queries;
        U lo=page[x/256],end=std::min(U(seeds.size()),page[x/256+1]+1);
        while(lo<end){U mid=(lo+end)/2;if constexpr(Track)++t.seed_cmps;
            if(seeds[mid].hi<=x)lo=mid+1;else end=mid;}
        if(lo<seeds.size()&&seeds[lo].lo<=x){assert(hi<=seeds[lo].hi);return lo;}
        assert(lo==seeds.size()||hi<=seeds[lo].lo);return none;
    }
};
struct Node {
    U leaf=none,width=0,actions=0,act_count=0,child=0,child_count=0,read=0,used=0;
    std::vector<Part> result;
};
struct Task {U target,lo,hi,depth;Byte prefix;};
struct RootForest {
    std::vector<Node> nodes;std::vector<Part> actions,local;
    std::vector<Task> tasks;std::vector<Byte> path;
    std::array<U,256> count{},image{},index{};
    std::array<Byte,256> keys;
};
template<bool Track>Child form(const Geometry& g,const Section& section,U primary,Stats& t) {
    Child child;SectionIndex index(section,g.n);RootForest f;
    U degree=0,mark_dist=none,which_seed=none;Q live=0;
    for(U seed=0;seed<section.seeds.size();++seed) {
        const auto root=section.seeds[seed];
        f.nodes.clear();f.nodes.emplace_back();f.actions.clear();f.tasks.clear();f.path.clear();
        f.tasks.push_back({0,root.lo,root.hi,0,0});
        while(!f.tasks.empty()){
            auto task=f.tasks.back();f.tasks.pop_back();
            f.path.resize(task.depth);if(task.target)f.path.push_back(task.prefix);
            U lo=task.lo,hi=task.hi;
            for(;;){
                if(!f.path.empty()){
                    U hit=index.containing<Track>(lo,hi,t);
                    if(hit!=none){
                        U id=child.classes.size();auto s=section.seeds[hit];
                        child.classes.push_back({s.base+lo-s.lo,hi-lo,U(child.dictionary.size()),U(f.path.size())});
                        child.dictionary.insert(child.dictionary.end(),f.path.rbegin(),f.path.rend());
                        auto& n=f.nodes[task.target];n.leaf=id;n.width=hi-lo;
                        if constexpr(Track){++t.leaves;t.spelling+=f.path.size();}
                        break;
                    }
                }
                if(lo<=primary&&primary<hi){
                    assert(mark_dist==none);mark_dist=f.path.size();which_seed=seed;
                }
                if constexpr(Track){++t.nodes;t.covered_rows+=hi-lo;}
                U run=g.locate<Track>(g.runs,g.source_page,lo,t.physical_queries,t.physical_cmps);
                const auto head=g.runs[run];
                if(hi<=head.lo+head.w) {
                    if constexpr(Track){++t.unary;++t.slices;}
                    U shift=head.image+lo-head.lo;hi=shift+(hi-lo);lo=shift;
                    f.path.push_back(head.b);continue;
                }
                f.local.clear();degree=0;
                for(U x=lo;x<hi;){
                    auto r=g.runs[run++];U w=std::min(hi,r.lo+r.w)-x;
                    if(!f.count[r.b]){f.keys[degree++]=r.b;f.image[r.b]=r.image+x-r.lo;}
                    f.count[r.b]+=w;f.local.push_back({r.b,w});x+=w;
                }
                assert(degree>=2);
                if constexpr(Track){++t.branch;t.slices+=f.local.size();}
                U childbase=f.nodes.size(),actionbase=f.actions.size();
                f.nodes.resize(childbase+degree);
                auto& node=f.nodes[task.target];node.actions=actionbase;node.act_count=f.local.size();node.child=childbase;node.child_count=degree;
                for(U j=0;j<degree;++j){
                    Byte b=f.keys[j];U index=childbase+j;f.index[b]=index;
                    f.tasks.push_back({index,f.image[b],f.image[b]+f.count[b],U(f.path.size()),b});
                    f.count[b]=0;
                }
                for(auto r:f.local)f.actions.push_back({f.index[r.id],r.w});
                break;
            }
        }
        for(U idx=f.nodes.size();idx--;){
            auto& node=f.nodes[idx];if(node.leaf!=none)continue;
            for(U j=0;j<node.act_count;++j){
                auto act=f.actions[node.actions+j];auto& c=f.nodes[act.id];U remaining=act.w;
                while(remaining){
                    Part next=c.leaf!=none?Part{c.leaf,c.width}:c.result[c.read];
                    U take=std::min(remaining,next.w-c.used);assert(take);
                    bool add=append_run(node.result,next.id,take);
                    if constexpr(Track){++t.merge_claims;if(add){++t.merge_records;++live;t.peak_live=std::max(t.peak_live,live);}}
                    remaining-=take;c.used+=take;if(c.used==next.w){++c.read;c.used=0;}
                }
            }
            for(U j=0;j<node.child_count;++j){
                auto& c=f.nodes[node.child+j];assert(c.read==(c.leaf!=none?1:c.result.size())&&c.used==0);
                if constexpr(Track)live-=c.result.size();
                std::vector<Part>().swap(c.result);
            }
        }
        auto& n=f.nodes.front();
        if(n.leaf!=none)append_run(child.incoming,n.leaf,n.width);
        else{for(auto r:n.result)append_run(child.incoming,r.id,r.w);if constexpr(Track)live-=n.result.size();}
        assert(!live);
    }
    if(mark_dist==none) {
        U x=primary;
        do{auto [b,y]=g.psi<Track>(x,t);child.pure.push_back(b);x=y;}while(x!=primary);
        child.default_lap=child.pure.size();if constexpr(Track)t.fallback=child.default_lap;
    }else{
        U x=primary;for(U i=0;i<mark_dist;++i)x=g.psi<Track>(x,t).second;
        if constexpr(Track)t.phase_steps=mark_dist;
        auto root=section.seeds[which_seed];assert(root.lo<=x&&x<root.hi);
        U rank=root.base+x-root.lo;
        if(!mark_dist){child.origin=rank;child.skip=0;}
        else{
            U at=0,run=0;while(at+child.incoming[run].w<=rank)at+=child.incoming[run++].w;
            U id=child.incoming[run].id,occ=rank-at;
            for(U j=0;j<run;++j)if(child.incoming[j].id==id)occ+=child.incoming[j].w;
            auto c=child.classes[id];child.origin=c.base+occ;assert(c.weight>=mark_dist);
            child.skip=c.weight-mark_dist;
        }
    }
    if constexpr(Track){assert(t.covered_rows<=g.n);assert(t.nodes==t.unary+t.branch);
        assert(t.slices<=t.nodes+g.runs.size()-1);assert(t.leaves==child.classes.size());}
    return child;
}
