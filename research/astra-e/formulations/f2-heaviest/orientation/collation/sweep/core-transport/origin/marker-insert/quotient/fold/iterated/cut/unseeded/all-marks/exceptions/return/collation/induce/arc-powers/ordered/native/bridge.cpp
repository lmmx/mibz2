#include "terminal.hpp"
#include "../ordered.hpp"
#include <cstdio>
#include <cstdlib>
#include <ctime>

namespace {
using namespace arc_cut;
using Q=uint64_t;
struct Sample {
    U arm,n,mark,records,runs,layers,nodes,terminal,lap;
    arc_output::Counts emit;
    Q caps_ns,induce_ns,spell_ns,retained_bytes,with_tree;
};
struct Log {
    const char* path=std::getenv("ARCCUT_STAMPS");std::vector<Sample> rows;
    Log(){if(path)rows.reserve(750);}
    ~Log(){
        if(!path)return;
        FILE* f=std::fopen(path,"w");if(!f)std::abort();
        std::fputs("arm,n,mark,records,runs,layers,nodes,terminal,lap,events,literal_bytes,memo_calls,memo_bytes,power_calls,power_bytes,lap_bytes,caps_ns,induce_ns,spell_ns,retained_bytes,with_tree_bytes\n",f);
        for(auto r:rows)std::fprintf(f,"%u,%u,%u,%u,%u,%u,%u,%u,%u,%u,%u,%u,%u,%u,%u,%u,%lu,%lu,%lu,%lu,%lu\n",
            r.arm,r.n,r.mark,r.records,r.runs,r.layers,r.nodes,r.terminal,r.lap,r.emit.events,r.emit.literals,r.emit.memo_calls,r.emit.memo_bytes,r.emit.power_calls,r.emit.power_bytes,r.emit.lap_bytes,
            r.caps_ns,r.induce_ns,r.spell_ns,r.retained_bytes,r.with_tree);
        if(std::fclose(f))std::abort();
    }
} logbook;
Q tick(){timespec t{};if(clock_gettime(CLOCK_THREAD_CPUTIME_ID,&t))std::abort();return Q(t.tv_sec)*1000000000+t.tv_nsec;}
struct Worker {
    std::vector<Run> runs;
    std::vector<uint8_t> out;
    arc_output::Memo memo;
    void run(const uint16_t* caps,U count,U n,U mark,U arm){
        assert(arm<3&&n&&mark<n);auto t0=tick();
        runs.clear();runs.reserve(count);U x=0;
        for(U i=0;i<count;++i){
            U width=caps[i]&255,byte=caps[i]>>8;assert(width&&width<=127&&width<=n-x);
            if(!runs.empty()&&runs.back().byte==byte)runs.back().end+=width;
            else runs.push_back({x,x+width,0,byte});
            x+=width;
        }
        assert(x==n);auto t1=tick();Grammar g;std::vector<Layer> layers;
        auto base=modular::initial(runs);U terminal=arm?arc_order::solve(std::move(base),mark,g,layers)
                                                      :arc_cut::solve(std::move(base),mark,g,layers);
        auto t2=tick();arc_output::Counts c{};
        if(arm==2){out.clear();out.reserve(n);g.emit(terminal,out,n);}
        else c=memo.emit(g,terminal,out,n);
        auto t3=tick();assert(out.size()==n);
        if(logbook.path){
            Q retained=sizeof(Run)*runs.capacity()+out.capacity()+memo.bytes();
            logbook.rows.push_back({arm,n,mark,count,U(runs.size()),U(layers.size()),U(g.nodes.size()),terminal,g.len(terminal),c,
                t1-t0,t2-t1,t3-t2,retained,retained+layers.capacity()*sizeof(Layer)+g.nodes.capacity()*sizeof(Node)});
        }
    }
};
}
extern "C" Worker* grammar_new(){return new Worker;}
extern "C" void grammar_delete(Worker* w){delete w;}
extern "C" const uint8_t* grammar_run(Worker* w,const uint16_t* caps,U records,U n,U mark,U arm){
    w->run(caps,records,n,mark,arm);return w->out.data();
}
