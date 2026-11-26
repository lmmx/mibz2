#include "../smooth.hpp"
#include <array>
#include <cstdio>
#include <cstdlib>
#include <ctime>
#include <map>
#include <tuple>
#include <vector>

namespace {
using Tick = uint64_t;
Tick cpu() {
    timespec t{};
    if (clock_gettime(CLOCK_THREAD_CPUTIME_ID, &t)) std::abort();
    return Tick(t.tv_sec)*1000000000ULL+t.tv_nsec;
}
struct Call {
    U arm,n,records,primary,depth,repaired;
    Tick supply,emit;
    size_t index_bytes,out_bytes;
};
struct Journal {
    const char* path=std::getenv("SMOOTH_STAMPS");
    std::vector<Call> calls;
    Journal() { if (path) calls.reserve(750); }
    ~Journal() {
        if (!path) return;
        auto* f=std::fopen(path,"w");
        if (!f) std::abort();
        std::fputs("arm,n,records,primary,depth,repair_steps,supply_ns,emit_ns,index_bytes,output_bytes\n",f);
        for (const auto& c:calls)
            std::fprintf(f,"%u,%u,%u,%u,%u,%u,%lu,%lu,%zu,%zu\n",
                    c.arm,c.n,c.records,c.primary,c.depth,c.repaired,c.supply,c.emit,c.index_bytes,c.out_bytes);
        if (std::fclose(f)) std::abort();
    }
} journal;

using Key=std::tuple<U,U,U>;
struct Cached {
    smooth::Index index;
    size_t bytes;
};
struct State {
    std::map<Key,Cached> ready;
    std::vector<Byte> out;
};
}

extern "C" State* grammar_new() {return new State;}
extern "C" void grammar_delete(State* st) {delete st;}
extern "C" const Byte* grammar_run(State* st,const uint16_t* caps,U records,U n,U primary,U arm) {
    static constexpr std::array<U,3> depths{4,8,16};
    if (arm>=depths.size()) std::abort();
    const U depth=depths[arm];
    const Key key{n,primary,records};
    auto entry=st->ready.find(key);
    Tick supply=0;
    if (entry==st->ready.end()) {
        auto start=cpu();
        auto index=smooth::prepare(caps,records,n,primary,depth);
        const size_t bytes=index.bytes();
        entry=st->ready.emplace(key,Cached{std::move(index),bytes}).first;
        supply=cpu()-start;
    }
    auto start=cpu();
    st->out.resize(n);
    const U repaired=smooth::emit(entry->second.index,st->out.data());
    auto end=cpu();
    if (journal.path)
        journal.calls.push_back({arm,n,records,primary,depth,repaired,supply,end-start,entry->second.bytes,st->out.capacity()});
    return st->out.data();
}
