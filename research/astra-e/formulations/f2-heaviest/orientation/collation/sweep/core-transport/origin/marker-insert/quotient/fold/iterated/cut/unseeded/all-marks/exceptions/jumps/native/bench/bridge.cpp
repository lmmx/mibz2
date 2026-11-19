#include "../model.hpp"
#include <cstdio>
#include <cstdlib>
#include <ctime>
#include <map>
#include <tuple>

namespace {
using Tick = uint64_t;
Tick now() {
    timespec t{};
    if (clock_gettime(CLOCK_THREAD_CPUTIME_ID, &t)) std::abort();
    return Tick(t.tv_sec) * 1000000000ULL + t.tv_nsec;
}

struct Stamp {
    U arm, n, primary, records, states{}, exceptions{}, repair{};
    Tick caps{}, geometry{}, build{}, emit{}, supply{};
    size_t raw_bytes{}, geometry_bytes{}, index_bytes{}, layer_peak{}, output_bytes{};
};

struct Journal {
    const char* path = std::getenv("JUMP_STAMPS");
    std::vector<Stamp> rows;
    Journal() { if (path) rows.reserve(750); }
    ~Journal() {
        if (!path) return;
        auto* f = std::fopen(path, "w");
        if (!f) std::abort();
        std::fputs("arm,n,primary,records,states,exceptions,repair_steps,caps_ns,geometry_ns,index_ns,emit_ns,supply_ns,raw_bytes,geometry_bytes,index_bytes,index_layer_peak,output_bytes\n", f);
        for (const auto& s : rows) {
            std::fprintf(f, "%u,%u,%u,%u,%u,%u,%u,%lu,%lu,%lu,%lu,%lu,%zu,%zu,%zu,%zu,%zu\n",
                s.arm,s.n,s.primary,s.records,s.states,s.exceptions,s.repair,
                s.caps,s.geometry,s.build,s.emit,s.supply,
                s.raw_bytes,s.geometry_bytes,s.index_bytes,s.layer_peak,s.output_bytes);
        }
        if (std::fclose(f)) std::abort();
    }
} journal;

struct Supplied { Input raw; jump::Geometry geometry; };
using Key = std::tuple<U,U,U>;

struct State {
    // Only the arm executed by this State populates its map. The corpus and
    // its unique (rows, primary, caps) keys are verified by the harness.
    bool variable = std::string(std::getenv("JUMP_MODE")) == "variable";
    std::map<Key,jump::Index> indices;
    std::map<Key,Supplied> supplied;
    std::vector<Byte> output;
};

Supplied supply(const uint16_t* caps, Stamp& s, bool variable) {
    Tick start = now();
    Input raw = from_caps(caps,s.records,s.n,s.primary);
    auto middle = now();
    auto geometry = contexts(raw,variable);
    auto end = now();
    s.caps = middle-start;
    s.geometry = end-middle;
    return {std::move(raw),std::move(geometry)};
}

jump::Index build(const Supplied& in, Stamp& s) {
    s.states = in.geometry.ends.size();
    s.exceptions = in.geometry.exceptions;
    s.raw_bytes = in.raw.runs.capacity()*sizeof(Run);
    s.geometry_bytes = in.geometry.bytes();
    auto start = now();
    auto index = jump::build(in.raw,in.geometry);
    s.build = now()-start;
    return index;
}

void emit(State& st, const jump::Index& index, Stamp& s) {
    s.index_bytes = index.bytes();
    s.layer_peak = index.construction_peak;
    s.states = index.one.link.size();
    s.exceptions = index.exceptions.size();
    auto start = now();
    st.output.resize(s.n);
    s.repair = jump::emit(index,s.primary,st.output.data());
    s.emit = now()-start;
    s.output_bytes = st.output.capacity();
}
}

extern "C" State* grammar_new() { return new State; }
extern "C" void grammar_delete(State* state) { delete state; }
extern "C" const Byte* grammar_run(State* st, const uint16_t* caps, U records,
                                   U n, U primary, U arm) {
    Stamp s{arm,n,primary,records};
    Key key{n,primary,records};
    if (arm == 0) {
        auto it = st->indices.find(key);
        if (it == st->indices.end()) {
            auto start = now();
            {
                auto model = supply(caps,s,st->variable);
                it = st->indices.emplace(key,build(model,s)).first;
            }
            s.supply = now()-start;
        }
        emit(*st,it->second,s);
    } else if (arm == 1) {
        auto it = st->supplied.find(key);
        if (it == st->supplied.end()) {
            auto start = now();
            it = st->supplied.emplace(key,supply(caps,s,st->variable)).first;
            s.supply = now()-start;
        }
        auto index = build(it->second,s);
        emit(*st,index,s);
    } else if (arm == 2) {
        auto model = supply(caps,s,st->variable);
        auto index = build(model,s);
        emit(*st,index,s);
    } else std::abort();
    if (journal.path) journal.rows.push_back(s);
    return st->output.data();
}
