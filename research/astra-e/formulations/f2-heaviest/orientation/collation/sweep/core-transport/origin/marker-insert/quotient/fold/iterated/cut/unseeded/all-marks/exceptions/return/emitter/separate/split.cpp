// Compile the frozen model, fused physical terminal and chronological comparator.
#define grammar_new original_new
#define grammar_delete original_delete
#define grammar_run original_run
#include "../terminal.cpp"
#undef grammar_run
#undef grammar_delete
#undef grammar_new

U acquire(const Ring& ring,bool primary_is_event,std::vector<Event>& events) {
    events.clear();
    events.reserve(ring.anchor.size());
    U a=ring.primary,t=0;
    if(!primary_is_event) {
        Anchor initial=ring.anchor[a];
        t=initial.distance;a=initial.next;
        if(a==ring.primary) return t;
    }
    do {
        Anchor e=ring.anchor[a];
        events.push_back({t,e.edge});
        t+=e.distance;a=e.next;
    } while(a!=ring.primary);
    return t;
}

void emit_clock_lap(Model& m,const std::vector<Jump>& table,U lap,
                    std::vector<Event>& events,uint8_t* out) {
    // Temporarily lend the original evaluator this worker's clock. Both swaps are O(1).
    U n=m.n;m.n=lap;m.events.swap(events);
    emit<true>(m,table,out);
    m.events.swap(events);m.n=n;
    for(U filled=lap;filled<n;) {
        U take=std::min(filled,n-filled);
        std::memcpy(out+filled,out,take);filled+=take;
    }
}

bool same_events(const std::vector<Event>& a,const std::vector<Event>& b) {
    return a.size()==b.size()&&std::equal(a.begin(),a.end(),b.begin(),
           [](Event x,Event y){return x.clock==y.clock&&x.edge==y.edge;});
}

struct Stamp { U block; Q acquire,emit,release,retained; U events,lap; };
struct SplitState {
    State* original;
    bool fresh;
    std::map<U,bool> primary_flags;
    std::vector<Event> scratch;
    std::vector<Stamp> stamps;
};
extern "C" SplitState* grammar_new() {
    auto mode=std::getenv("RING_MODE");assert(mode);
    auto* result=new SplitState{original_new(),std::string(mode)=="fresh",{},{},{}};
    result->stamps.reserve(400);
    return result;
}
extern "C" void grammar_delete(SplitState* s) {
    if(!s->stamps.empty()) {
        std::ofstream f(std::getenv("RING_STAMPS"));
        f<<"block,acquire_ns,emit_ns,release_ns,allocation_bytes,events,lap\n";
        for(auto q:s->stamps) f<<q.block<<','<<q.acquire<<','<<q.emit<<','<<q.release
            <<','<<q.retained<<','<<q.events<<','<<q.lap<<'\n';
    }
    original_delete(s->original);delete s;
}
extern "C" const uint8_t* grammar_run(SplitState* s,const uint16_t* caps,U records,U n,U p,U arm) {
    State* old=s->original;
    if(arm<2) return original_run(old,caps,records,n,p,arm);
    U b=lookup(n,p,records);
    auto [where,first]=s->primary_flags.try_emplace(b,false);
    if(first) {
        // Untimed preflight constructs the same physically ordered fixture as arm 0.
        original_run(old,caps,records,n,p,0);
        auto name=old->wires/std::to_string(b);
        const auto mask=read_file(name.string()+".mask8");
        bool primary_is_event=mask[16+n+p]!=0;where->second=primary_is_event;
        Model known;known.parse<true>(read_file(name.string()+".pred8"));
        std::vector<Event> found;
        U lap=acquire(old->blocks.at(b).ring,primary_is_event,found);
        assert(lap==n&&same_events(found,known.events));
    }
    auto& c=old->blocks.at(b);
    Q begin=cpu();U lap,count;Q acquired,emitted,capacity;
    {
        std::vector<Event> local;
        auto& events=s->fresh?local:s->scratch;
        lap=acquire(c.ring,where->second,events);
        acquired=cpu();
        old->output.resize(n);
        emit_clock_lap(c.m,c.table,lap,events,old->output.data());
        emitted=cpu();capacity=events.capacity()*sizeof(Event);count=events.size();
    }
    Q released=cpu();
    s->stamps.push_back({b,acquired-begin,emitted-acquired,released-emitted,capacity,count,lap});
    return old->output.data();
}
#ifdef FINITE
// Keep g++ from specializing the general chronological evaluator to outputs shorter than 8.
[[gnu::noipa]] void check_clock_lap(Model& m,const std::vector<Jump>& table,U lap,
                                    std::vector<Event>& events,uint8_t* out) {
    emit_clock_lap(m,table,lap,events,out);
}
#include "small.inc"
#endif
