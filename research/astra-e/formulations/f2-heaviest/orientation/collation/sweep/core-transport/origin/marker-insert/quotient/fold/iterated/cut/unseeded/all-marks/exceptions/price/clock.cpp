#include <algorithm>
#include <array>
#include <cassert>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <ctime>
#include <filesystem>
#include <fstream>
#include <map>
#include <span>
#include <vector>
#include "ids.inc"
using U=uint32_t;
using Q=uint64_t;
Q cpu(){timespec t;int ok=clock_gettime(CLOCK_THREAD_CPUTIME_ID,&t);assert(!ok);return Q(t.tv_sec)*1000000000+t.tv_nsec;}
std::vector<uint8_t> read_file(const std::filesystem::path& p){
    std::ifstream f(p,std::ios::binary|std::ios::ate);assert(f);std::vector<uint8_t> v(f.tellg());f.seekg(0);
    f.read((char*)v.data(),v.size());assert(f);return v;
}
U load32(const uint8_t* p){return U(p[0])|(U(p[1])<<8)|(U(p[2])<<16)|(U(p[3])<<24);}
struct Event{U clock,edge;};
struct Jump{std::array<uint8_t,8> bytes;U next;};
static_assert(sizeof(Event)==8&&sizeof(Jump)==12);
struct Model{
    U n=0,start=0;std::vector<U> transition;std::vector<Event> events;
    Model()=default;
    template<bool Check> void parse(std::span<const uint8_t> wire){
        assert(wire.size()>=24&&!std::memcmp(wire.data(),"PREDCLK1",8));
        auto p=wire.data()+8;n=load32(p);start=load32(p+4);U states=load32(p+8),count=load32(p+12);p+=16;
        assert(n&&states&&start<states&&states<(1u<<24)&&count<=n&&wire.size()==24+5ull*states+9ull*count);
        transition.resize(states);events.resize(count);
        for(auto& a:transition){
            uint8_t symbol=p[0];U next=load32(p+1);p+=5;
            if constexpr(Check)assert(next<states);
            a=(next<<8)|symbol;
        }
        U t=0;
        for(auto& e:events){
            U u=load32(p);uint8_t symbol=p[4];U next=load32(p+5);p+=9;
            if constexpr(Check){assert(u>=t&&u<n&&next<states);t=u+1;}
            e={u,(next<<8)|symbol};
        }
    }
};
void prepare(const Model& m,std::vector<Jump>& table){
    table.resize(m.transition.size());
    for(U s=0;s<table.size();++s){
        U x=s;Jump j;
        for(unsigned int t=8;t-->0;){
            auto e=m.transition[x];j.bytes[t]=uint8_t(e);x=e>>8;
        }
        j.next=x;table[s]=j;
    }
}
template<bool J> void emit(const Model& m,const std::vector<Jump>& table,uint8_t* out){
    U s=m.start,t=0;
    auto defaults=[&](U stop){
        if constexpr(J)while(stop-t>=8){
            const auto& w=table[s];std::memcpy(out+m.n-t-8,w.bytes.data(),8);s=w.next;t+=8;
        }
        while(t<stop){U e=m.transition[s];out[m.n-1-t++]=uint8_t(e);s=e>>8;}
    };
    for(auto e:m.events){
        defaults(e.clock);out[m.n-1-t++]=uint8_t(e.edge);s=e.edge>>8;
    }
    defaults(m.n);
}
struct Stats{U block,k,arm;Q init=0,parse=0,jump=0,provide=0,first=0,verify=0,second=0,statebytes=0,events=0,table=0,wire=0,output=0,ref=0;};
struct Journal{std::vector<Stats> rows;
    ~Journal(){if(auto p=std::getenv("CLOCK_STAMPS")){std::ofstream f(p);
        f<<"block,k,arm,once_ns,parse_ns,jump_ns,provision_ns,first_ns,verify_ns,second_ns,state_bytes,event_bytes,jump_bytes,resident_wire_bytes,output_bytes,reference_bytes\n";
        for(auto q:rows)f<<q.block<<','<<q.k<<','<<q.arm<<','<<q.init<<','<<q.parse<<','<<q.jump<<','<<q.provide<<','<<q.first<<','<<q.verify<<','<<q.second<<','<<q.statebytes<<','<<q.events<<','<<q.table<<','<<q.wire<<','<<q.output<<','<<q.ref<<'\n';
    }}
};
static Journal journal;
struct Cache{Model m;std::vector<Jump> table;std::vector<uint8_t> wire,want;};
struct State{U k;bool provisioning;std::filesystem::path wires,refs;std::map<U,Cache> blocks;std::vector<uint8_t> output;};
extern "C" State* grammar_new(){
    auto k=std::getenv("CLOCK_K"),mode=std::getenv("CLOCK_MODE"),w=std::getenv("CLOCK_WIRES"),r=std::getenv("CLOCK_REFS");assert(k&&mode&&w&&r);
    return new State{U(std::atoi(k)),std::string(mode)=="provision",w,r,{},{}};
}
extern "C" void grammar_delete(State* s){delete s;}
template<bool J> void paid(const Model& m,const std::vector<Jump>& table,std::vector<uint8_t>& out,Stats& q){
    Q t=cpu();out.resize(m.n);Q start=cpu();emit<J>(m,table,out.data());Q end=cpu();
    q.provide=start-t;q.first=end-start;q.statebytes=m.transition.capacity()*sizeof(U);q.events=m.events.capacity()*sizeof(Event);
    q.table=table.capacity()*sizeof(Jump);q.output=out.capacity();
}
extern "C" const uint8_t* grammar_run(State* s,const uint16_t* caps,U records,U n,U mark,U arm){
    assert(caps&&arm<3);auto id=lookup(n,mark,records);Stats q{id,s->k,arm};
    auto [it,created]=s->blocks.try_emplace(id);Cache& c=it->second;
    bool supply=s->provisioning;bool unit=!arm;bool fromwire=supply&&arm<2;
    bool twice=!supply&&arm!=1;
    if(created){
        Q t=cpu();auto b=read_file(s->wires/(std::to_string(id)+".pred"+std::to_string(s->k)));
        c.m.parse<true>(b);assert(c.m.n==n);
        if(fromwire){c.wire=std::move(b);c.m=Model{};}
        else if(!supply&&!unit)prepare(c.m,c.table);
        if(twice)c.want=read_file(s->refs/("block-"+std::to_string(id)+".raw"));
        q.init=cpu()-t;
    }
    if(supply){
        Model fresh;const Model* m=&c.m;
        if(fromwire){Q t=cpu();fresh.parse<false>(c.wire);q.parse=cpu()-t;m=&fresh;}
        std::vector<Jump> table;std::vector<uint8_t> out;
        if(!unit){Q t=cpu();prepare(*m,table);q.jump=cpu()-t;}
        if(unit)paid<false>(*m,table,out,q);else paid<true>(*m,table,out,q);
        s->output=std::move(out);q.wire=c.wire.capacity();
    }else{
        if(unit)paid<false>(c.m,c.table,s->output,q);else paid<true>(c.m,c.table,s->output,q);
        if(twice){
            Q t=cpu();assert(s->output==c.want);Q start=cpu();
            if(unit)emit<false>(c.m,c.table,s->output.data());else emit<true>(c.m,c.table,s->output.data());
            Q end=cpu();q.verify=start-t;q.second=end-start;q.ref=c.want.capacity();
        }
    }
    if(std::getenv("CLOCK_STAMPS"))journal.rows.push_back(q);
    return s->output.data();
}
