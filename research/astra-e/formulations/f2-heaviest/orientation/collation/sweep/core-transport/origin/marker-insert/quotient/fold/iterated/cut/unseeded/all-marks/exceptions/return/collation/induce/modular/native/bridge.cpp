#include "../modular.hpp"
#include <cstdio>
#include <cstdlib>
#include <ctime>

namespace {
using namespace modular;
using Byte=uint8_t;using Q=uint64_t;
struct Sample{
    U arm,n,mark,records,runs,q,layers,joins,terminal,lap,visits,copies,aliases,repeats;
    Q pieces,steps,cap_time,build_time,output_time,retained_capacity,with_tree;
};
struct Log{
    const char* path=std::getenv("MODULAR_STAMPS");std::vector<Sample> rows;
    Log(){if(path)rows.reserve(750);}
    ~Log(){
        if(!path)return;
        FILE* f=std::fopen(path,"w");if(!f)std::abort();
        std::fputs("arm,n,mark,records,runs,q,layers,joins,terminal,lap,visits,copies,aliases,repeats,pieces,steps,cap_ns,induce_ns,spell_ns,retained_bytes,including_tree_bytes\n",f);
        for(auto& r:rows)std::fprintf(f,"%u,%u,%u,%u,%u,%u,%u,%u,%u,%u,%u,%u,%u,%u,%lu,%lu,%lu,%lu,%lu,%lu,%lu\n",
            r.arm,r.n,r.mark,r.records,r.runs,r.q,r.layers,r.joins,r.terminal,r.lap,r.visits,r.copies,r.aliases,r.repeats,
            r.pieces,r.steps,r.cap_time,r.build_time,r.output_time,r.retained_capacity,r.with_tree);
        if(std::fclose(f))std::abort();
    }
} logbook;
Q now(){timespec t{};if(clock_gettime(CLOCK_THREAD_CPUTIME_ID,&t))std::abort();return Q(t.tv_sec)*1000000000+t.tv_nsec;}
struct Worker {
    std::vector<Run> runs;
    std::vector<Byte> output;
    void run(const uint16_t* caps,U count,U n,U mark,U arm){
        assert(arm<3&&n&&mark<n);U q=2u<<arm;
        auto a=now();runs.clear();runs.reserve(count);U x=0;
        for(U i=0;i<count;++i){
            U width=caps[i]&255,byte=caps[i]>>8;assert(width&&width<=127&&width<=n-x);
            if(!runs.empty()&&runs.back().byte==byte)runs.back().end+=width;
            else runs.push_back({x,x+width,0,byte}); // modular::initial reads only begin/end/byte.
            x+=width;
        }
        assert(x==n);auto b=now();
        auto tree=modular::induce(runs,mark,q);auto c=now();
        output.resize(n);auto e=modular::spell(tree,output.data(),n);auto d=now();
        if(logbook.path){
            Q pieces=0,steps=0;
            for(auto& l:tree.layers){pieces+=l.pieces;steps+=l.self+l.partial+l.equal+l.strict;}
            Q retained=sizeof(Run)*runs.capacity()+output.capacity();
            logbook.rows.push_back({arm,n,mark,count,U(runs.size()),q,U(tree.layers.size()),U(tree.grammar.size()),
                tree.terminal,e.lap,e.visits,e.copies,e.aliases,e.repeats,pieces,steps,b-a,c-b,d-c,retained,
                retained+sizeof(Join)*tree.grammar.capacity()+sizeof(Layer)*tree.layers.capacity()});
        }
    }
};
}
extern "C" Worker* grammar_new(){return new Worker;}
extern "C" void grammar_delete(Worker* p){delete p;}
extern "C" const Byte* grammar_run(Worker* w,const uint16_t* caps,U count,U n,U mark,U arm){
    w->run(caps,count,n,mark,arm);return w->output.data();
}
