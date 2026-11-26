#include "clock-model.hpp"
#include <iostream>

struct Anchor {
    U next, distance, edge;
};
static_assert(sizeof(Anchor)==12);
struct Ring {
    U primary=0;
    std::vector<Anchor> anchor;
};

// Prepare the conditional table with a deliberately dense independent stable LF.
// Explicitly remove this fixture from the timed and retained terminal state.
std::vector<U> stable_lf(std::span<const uint8_t> last) {
    std::array<U,256> hist{},offset{};
    for(auto c:last) ++hist[c];
    for(U b=1;b<256;++b) offset[b]=offset[b-1]+hist[b-1];
    std::vector<U> lf;lf.reserve(last.size());
    for(auto c:last) lf.push_back(offset[c]++);
    return lf;
}

Ring make_ring(const std::vector<U>& lf,std::span<const uint8_t> exception,
               U primary,const std::vector<U>& payload,bool lap_order) {
    U n=lf.size();
    std::vector<U> slot(n,UINT32_MAX),rows;
    auto add=[&](U x) {slot[x]=rows.size();rows.push_back(x);};
    if(lap_order) {
        // Number the marked cycle's anchors first. Number foreign rows afterwards.
        U x=primary;
        do {if(x==primary||exception[x]) add(x);x=lf[x];} while(x!=primary);
    }
    for(U x=0;x<n;++x)
        if((exception[x]||x==primary)&&slot[x]==UINT32_MAX) add(x);
    Ring result;result.primary=slot[primary];
    result.anchor.reserve(rows.size());
    for(U x:rows) {
        U distance=1,y=lf[x];
        while(slot[y]==UINT32_MAX) {y=lf[y];++distance;}
        result.anchor.push_back({slot[y],distance,payload[x]});
    }
    return result;
}

void emit_ring(const Model& model,const std::vector<Jump>& table,
               const Ring& ring,uint8_t* out) {
    U a=ring.primary,remaining=model.n;
    do {
        const Anchor entry=ring.anchor[a];
        out[--remaining]=uint8_t(entry.edge);
        U state=entry.edge>>8,defaults=entry.distance-1;
        while(defaults>=8) {
            const Jump& w=table[state];remaining-=8;
            std::memcpy(out+remaining,w.bytes.data(),8);state=w.next;defaults-=8;
        }
        while(defaults--) {
            U edge=model.transition[state];out[--remaining]=uint8_t(edge);state=edge>>8;
        }
        a=entry.next;
    } while(a!=ring.primary);
    if(remaining) {
        U lap=model.n-remaining;
        std::memmove(out,out+remaining,lap);
        for(U filled=lap;filled<model.n;) {
            U take=std::min(filled,model.n-filled);
            std::memcpy(out+filled,out,take);filled+=take;
        }
    }
}

struct Cache {
    Model m;
    std::vector<Jump> table;
    Ring ring;
};
struct State {
    std::filesystem::path wires;
    std::map<U,Cache> blocks;
    std::vector<uint8_t> output;
};
#ifndef SMALL_CHECK
extern "C" State* grammar_new() {
    const char* wires=std::getenv("CLOCK_WIRES");assert(wires);
    return new State{wires,{},{}};
}
extern "C" void grammar_delete(State* s){delete s;}
extern "C" const uint8_t* grammar_run(State* s,const uint16_t* caps,U records,U n,U primary,U arm) {
    assert(caps&&arm<3);U id=lookup(n,primary,records);
    auto [it,created]=s->blocks.try_emplace(id);
    Cache& c=it->second;
    if(created) {
        auto name=s->wires/std::to_string(id);
        c.m.parse<true>(read_file(name.string()+".pred8"));assert(c.m.n==n);
        prepare(c.m,c.table);
        if(arm!=1) {
            auto wire=read_file(name.string()+".mask8");
            assert(wire.size()==16+2ull*n&&!std::memcmp(wire.data(),"CLKMASK1",8));
            assert(load32(wire.data()+8)==n&&load32(wire.data()+12)==primary);
            std::span<uint8_t> last(wire.data()+16,n),exc(wire.data()+16+n,n);
            U pos=0;
            for(U j=0;j<records;++j) {
                U w=caps[j]&255;assert(w<=n-pos);
                for(U k=0;k<w;++k) assert(last[pos+k]==(caps[j]>>8));
                pos+=w;
            }
            assert(pos==n);
            auto lf=stable_lf(last);
            std::vector<U> payload(n,UINT32_MAX);
            U x=primary,state=c.m.start,j=0;
            for(U t=0;t<n;++t) {
                U edge=c.m.transition[state];
                if(exc[x]) {
                    assert(j<c.m.events.size()&&c.m.events[j].clock==t);
                    edge=c.m.events[j++].edge;
                }
                assert(uint8_t(edge)==last[x]);
                if(exc[x]||x==primary) payload[x]=edge;
                state=edge>>8;x=lf[x];assert(x!=primary||t+1==n);
            }
            assert(j==c.m.events.size()&&state==c.m.start&&x==primary);
            c.ring=make_ring(lf,exc,primary,payload,arm==2);
            std::vector<Event>().swap(c.m.events);
        }
    }
    s->output.resize(n);
    if(arm==1) emit<true>(c.m,c.table,s->output.data());
    else emit_ring(c.m,c.table,c.ring,s->output.data());
    return s->output.data();
}
#else
int main() {
    Q total=0,shorts=0,partial=0,artificial=0,foreign=0;
    for(U alphabet:{2u,3u}) for(U n=1;n<=(alphabet==2?7u:5u);++n) {
        U columns=1;for(U i=0;i<n;++i)columns*=alphabet;
        for(U l=0;l<columns;++l) {
            U code=l;std::vector<uint8_t> last(n);
            for(auto& b:last){b=code%alphabet;code/=alphabet;}
            auto lf=stable_lf(last);
            std::vector<U> psi(n);for(U x=0;x<n;++x)psi[lf[x]]=x;
            for(U mask=0;mask<(1u<<n);++mask) {
                std::vector<uint8_t> exc(n);std::vector<U> payload(n);
                Model m;m.n=n;m.transition.resize(n);
                for(U x=0;x<n;++x) {
                    exc[x]=(mask>>x)&1;payload[x]=(lf[x]<<8)|last[x];
                    // A deliberately wrong default byte and successor at actual exceptions.
                    m.transition[x]=exc[x]?((((lf[x]+1)%n)<<8)|(last[x]^0xff)):payload[x];
                }
                std::vector<Jump> table;prepare(m,table);
                for(U p=0;p<n;++p) {
                    std::vector<uint8_t> want,got(n);
                    U y=p;
                    for(U i=0;i<n;++i){y=psi[y];want.push_back(last[y]);}
                    U x=p,lap=0,events=0;
                    do {events+=exc[x];++lap;x=lf[x];} while(x!=p);
                    ++total;shorts+=lap<n;partial+=n%lap!=0;artificial+=!exc[p];
                    foreign+=events!=U(__builtin_popcount(mask));
                    for(bool layout:{false,true}) {
                        auto ring=make_ring(lf,exc,p,payload,layout);
                        emit_ring(m,table,ring,got.data());assert(got==want);
                    }
                }
            }
        }
    }
    std::cout<<"cases,short_laps,nondivisor,artificial,foreign_exception\n"
             <<total<<','<<shorts<<','<<partial<<','<<artificial<<','<<foreign<<'\n';
}
#endif
