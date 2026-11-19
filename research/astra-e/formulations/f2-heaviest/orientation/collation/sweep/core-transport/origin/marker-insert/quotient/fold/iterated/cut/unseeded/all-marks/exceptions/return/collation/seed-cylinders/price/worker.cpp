#include "../forest.hpp"
#include <cstdlib>
#include <cstring>
#include <filesystem>
#include <fstream>
#include <map>
#include <string>
#include <time.h>
#include "ids.inc"
Q cpu(){timespec t;if(clock_gettime(CLOCK_THREAD_CPUTIME_ID,&t))std::abort();return Q(t.tv_sec)*1000000000+t.tv_nsec;}
void geometry(Geometry& g,const uint16_t* caps,U num) {
    g.n=0;g.runs.clear();g.image.clear();for(auto& p:g.positions)p.clear();
    g.base.fill(0);for(U j=0;j<num;++j)g.add(Byte(caps[j]>>8),caps[j]&255);
    g.finish();
}
struct Cache {Geometry g;Section sec;std::vector<Byte> raw;};
struct Stamp {U b,arm,mixed,seeds,rows,classes,runs;Q geometry,classify,forest,terminal;};
struct State {
    std::filesystem::path ref;std::map<U,Cache> inputs;Geometry working;
    std::vector<Byte> output;std::vector<U> ptr,packed;std::vector<uint16_t> order;
    std::vector<Stamp> stamps;
};
extern "C" State* grammar_new(){auto p=std::getenv("SEED_REF");assert(p);return new State{p,{},{},{},{},{},{},{}};}
extern "C" void grammar_delete(State* s){
    if(auto dir=std::getenv("SEED_STAMPS");dir&&!s->stamps.empty()){
        auto id=s->stamps.front().arm;std::ofstream f(std::filesystem::path(dir)/("phases-"+std::to_string(id)+".csv"));
        f<<"block,arm,mixed,seeds,events,classes,runs,index_ns,classify_ns,forest_ns,terminal_ns\n";
        for(auto v:s->stamps)f<<v.b<<','<<v.arm<<','<<v.mixed<<','<<v.seeds<<','<<v.rows<<','<<v.classes<<','<<v.runs
                              <<','<<v.geometry<<','<<v.classify<<','<<v.forest<<','<<v.terminal<<'\n';
    }delete s;
}
void terminal(State& s,const Child& c,U e,U n){
    // All ten frozen k=8 cases fit two 16-bit fields. The generic correctness
    // gate uses separate full-width class/row arrays for arbitrary objects.
    assert(e<=65536&&c.classes.size()<=65536);s.output.resize(n);
    if(c.origin==none){assert(!c.pure.empty());for(U i=0;i<n;++i)s.output[i]=c.pure[i%c.pure.size()];return;}
    s.ptr.resize(c.classes.size());for(U i=0;i<c.classes.size();++i)s.ptr[i]=c.classes[i].base;
    s.packed.resize(e);U dest=0;for(auto r:c.incoming){
        U& start=s.ptr[r.id];std::iota(s.packed.begin()+start,s.packed.begin()+start+r.w,(r.id<<16)|dest);start+=r.w;dest+=r.w;}
    assert(dest==e);s.order.clear();s.order.reserve(e);U x=c.origin;
    do{U v=s.packed[x];s.order.push_back(v>>16);x=v&65535;}while(x!=c.origin);
    U p=0;const auto first=c.classes[s.order.front()];
    auto copy=[&](const Literal& v,U from,U size){if(size)std::memcpy(s.output.data()+p,c.dictionary.data()+v.offset+from,size);p+=size;};
    copy(first,c.skip,first.weight-c.skip);
    for(U j=1;j<s.order.size();++j){auto v=c.classes[s.order[j]];copy(v,0,v.weight);}
    copy(first,0,c.skip);assert(p&&p<=n);
    while(p<n){U take=std::min(p,n-p);std::memcpy(s.output.data()+p,s.output.data(),take);p+=take;}
}
extern "C" const Byte* grammar_run(State* s,const uint16_t* caps,U num,U n,U p,U arm){
    U b=lookup(n,p,num);auto [it,fresh]=s->inputs.try_emplace(b);auto& in=it->second;Stats null;
    if(fresh&&arm==0){std::ifstream f(s->ref/("block-"+std::to_string(b)+".raw"),std::ios::binary);in.raw.resize(n);f.read((char*)in.raw.data(),n);assert(f&&f.peek()==EOF);}
    if(fresh&&arm==1){geometry(in.g,caps,num);in.sec=classify<false>(in.g,8,null);}
    const Geometry* g=&in.g;const Section* sec=&in.sec;Section computed;Child col;
    Q start=cpu();
    if(arm!=1){geometry(s->working,caps,num);g=&s->working;}
    Q indexed=cpu();
    if(arm!=1){computed=classify<false>(*g,8,null);sec=&computed;}
    Q classified=cpu();
    if(arm!=0)col=form<false>(*g,*sec,p,null);
    Q formed=cpu();
    if(arm!=0)terminal(*s,col,sec->mass,n);
    Q end=cpu();s->stamps.push_back({b,arm,U(sec->mixed.size()),U(sec->seeds.size()),sec->mass,U(col.classes.size()),
                   U(col.incoming.size()),indexed-start,classified-indexed,formed-classified,end-formed});
    return arm?s->output.data():in.raw.data();
}
