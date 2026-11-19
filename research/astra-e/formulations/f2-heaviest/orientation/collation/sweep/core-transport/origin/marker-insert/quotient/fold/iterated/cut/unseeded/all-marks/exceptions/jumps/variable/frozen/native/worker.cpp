#include <span>
#include <cstdio>
#include <cstdlib>
#include <ctime>
#include "model.hpp"
#include "dictionaries.inc"
namespace fixed {
struct Occ {U begin,end,base;};
struct Directory{
    std::array<std::vector<Occ>,256> letter;
    std::array<U,256> base;
    explicit Directory(const Input& raw){
        ::Geometry g(raw);
        std::copy_n(g.base.begin(),256,base.begin());
        auto next=base;U x=0;
        for(auto [w,a]:raw.runs){
            letter[a].push_back({x,x+w,next[a]});next[a]+=w;x+=w;
        }
    }
    U rank(Byte a,U x)const{
        auto& v=letter[a];
        auto i=std::lower_bound(v.begin(),v.end(),x,[](Occ r,U pos){return r.begin<pos;});
        if(i==v.begin())return base[a];
        --i;return i->base+std::min(x,i->end)-i->begin;
    }
};
jump::Geometry project(const Input& raw,U arm){
    Directory directory(raw);
    selection::Counts charge;
    std::vector<selection::State> prev{{0,raw.n,0}},owner=prev;
    unsigned depth=0;
    for(auto level:all_models[arm]){
        depth++;
        std::vector<selection::State> next,active;
        next.reserve(level.size());active.reserve(level.size());
        for(U item:level){
            auto par=prev[item>>8];Byte a=Byte(item);
            U l=0,r=0;
            if(par.begin<par.end){l=directory.rank(a,par.begin);r=directory.rank(a,par.end);}
            selection::State st{l,r,(uint64_t(depth)<<32)|l};
            next.push_back(st);if(r>l)active.push_back(st);
        }
        owner=selection::overlay(owner,active,charge);prev.swap(next);
    }
    jump::Geometry out;out.modes.reserve(owner.size());out.ends.reserve(owner.size());
    std::array<U,256> freq{};std::array<Byte,256> touched{};
    Cursor read{0,raw.runs.front().width};
    for(auto st:owner){
        U used=0;
        intersect(raw,read,st.begin,st.end,[&](Byte a,U x,U y){
            if(!freq[a])touched[used++]=a;
            freq[a]+=y-x;
        });
        Byte mode=touched[0];
        for(U j=1;j<used;j++){
            Byte a=touched[j];if(freq[a]>freq[mode]||(freq[a]==freq[mode]&&a<mode))mode=a;
        }
        out.ends.push_back(st.end);out.modes.push_back(mode);out.exceptions+=st.end-st.begin-freq[mode];
        for(U j=0;j<used;j++)freq[touched[j]]=0;
    }
    return out;
}
uint64_t cpu(){timespec t;auto rc=clock_gettime(CLOCK_THREAD_CPUTIME_ID,&t);assert(!rc);return uint64_t(t.tv_sec)*1000000000+t.tv_nsec;}
}
struct Stamp {U n,mark,arm;uint64_t caps,project,index,write;size_t states;U exceptions,repair;};
struct State {std::vector<Byte> out;std::vector<Stamp> stamps;bool log=false;};
extern "C" State* grammar_new(){
    auto* s=new State;
    s->log=std::getenv("FROZEN_JUMP_LOG")!=nullptr;
    if(s->log)s->stamps.reserve(256);
    return s;
}
extern "C" void grammar_delete(State* s){
    if(s->log){
        auto f=std::fopen(std::getenv("FROZEN_JUMP_LOG"),"a");assert(f);
        for(auto q:s->stamps)std::fprintf(f,"%u,%u,%u,%lu,%lu,%lu,%lu,%zu,%u,%u\n",q.n,q.mark,q.arm,
            q.caps,q.project,q.index,q.write,q.states,q.exceptions,q.repair);
        std::fclose(f);
    }
    delete s;
}
extern "C" const uint8_t* grammar_run(State* s,const uint16_t* caps,U records,U n,U mark,U arm){
    assert(arm<3);auto a=fixed::cpu();
    auto raw=from_caps(caps,records,n,mark);auto b=fixed::cpu();
    auto model=fixed::project(raw,arm);auto c=fixed::cpu();
    auto idx=jump::build(raw,model);auto d=fixed::cpu();
    s->out.resize(n);auto lap=jump::emit(idx,mark,s->out.data());auto e=fixed::cpu();
    if(s->log)s->stamps.push_back({n,mark,arm,b-a,c-b,d-c,e-d,model.ends.size(),model.exceptions,lap});
    return s->out.data();
}
