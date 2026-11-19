#include "../worker.hpp"
#include <cstdio>
#include <cstdlib>
#include <vector>

namespace {
using induced_worker::Charge;
using induced_worker::Worker;
struct Call {uint32_t arm,n,primary,input;Charge cost;};
struct Journal {
    const char* path=std::getenv("SECTION_STAMPS");
    std::vector<Call> calls;
    Journal(){if(path)calls.reserve(750);}
    ~Journal(){
        if(!path)return;
        FILE* file=std::fopen(path,"w");if(!file)std::abort();
        std::fputs("arm,n,primary,input,caps_ns,selector_ns,induce_ns,map_ns,emit_ns,kept,records,joins,lap,selected_spans,vector_bytes,live_with_result\n",file);
        for(const auto& q:calls){
            const auto& c=q.cost;
            std::fprintf(file,"%u,%u,%u,%u,%lu,%lu,%lu,%lu,%lu,%u,%u,%u,%u,%u,%zu,%zu\n",
                q.arm,q.n,q.primary,q.input,c.caps_ns,c.selector_ns,c.induce_ns,c.map_ns,c.emit_ns,
                c.kept,c.records,c.joins,c.lap,c.selected_spans,c.vector_bytes,c.live_with_result);
        }
        if(std::fclose(file))std::abort();
    }
} journal;
}
extern "C" Worker* grammar_new(){return new Worker;}
extern "C" void grammar_delete(Worker* w){delete w;}
extern "C" const uint8_t* grammar_run(Worker* w,const uint16_t* caps,uint32_t count,uint32_t n,uint32_t p,uint32_t mode){
    w->run(caps,count,n,p,mode);if(w->output.size()!=n)std::abort();
    if(journal.path)journal.calls.push_back({mode,n,p,count,w->last});
    return w->output.data();
}
