#include <algorithm>
#include <array>
#include <cassert>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <map>
#include <numeric>
#include <string>
#include <vector>
#include <time.h>
#include "ids.inc"
using U=uint32_t;using Q=uint64_t;using Byte=uint8_t;
constexpr U none=UINT32_MAX;
Q cpu(){timespec t;if(clock_gettime(CLOCK_THREAD_CPUTIME_ID,&t))std::abort();return Q(t.tv_sec)*1000000000+t.tv_nsec;}
template<class T> Q bytes(const std::vector<T>& v){return sizeof(T)*v.capacity();}
U read32(std::istream& f){std::array<Byte,4>b;f.read((char*)b.data(),4);assert(f);return U(b[0])|(U(b[1])<<8)|(U(b[2])<<16)|(U(b[3])<<24);}
struct Rule {U base,weight,post;Byte first;};
struct Run {U base,width;};
struct Jump {std::array<Byte,8> bytes;U next;};
struct Bank {std::vector<U> offsets;std::vector<Byte> text;Q capacity()const{return bytes(offsets)+bytes(text);}};
struct Input {
    U n=0,start=0,pop=0,origin=none,skip=0,empty_lap=0;
    std::vector<Rule> rules;
    std::vector<Run> incoming;
    std::vector<U> defaults;
    std::vector<Jump> jump;
    Bank bank;
    Q capacity()const {return bytes(rules)+bytes(incoming)+bytes(defaults)+bytes(jump)+bank.capacity();}
};
void read_input(Input& a,const std::filesystem::path& dir,U b) {
    auto p=dir/std::to_string(b);
    std::ifstream f(p.string()+".col",std::ios::binary),g(p.string()+".model",std::ios::binary);char magic[8];
    f.read(magic,8);assert(std::string(magic,8)=="EXCCOL01");
    U K=read32(f),R=read32(f);a.origin=read32(f);a.skip=read32(f);a.empty_lap=read32(f);
    a.rules.resize(K);a.incoming.resize(R);
    for(auto& r:a.rules) {
        r.base=read32(f);U count=read32(f);r.weight=read32(f);
        read32(f);r.first=f.get();f.get();r.post=read32(f);
        assert(r.base==a.pop&&r.weight&&count);a.pop+=count;
    }
    U total=0;for(auto& r:a.incoming){r.base=read32(f);r.width=read32(f);total+=r.width;}assert(total==a.pop&&f.peek()==EOF);
    g.read(magic,8);assert(std::string(magic,8)=="MODEDEF1");
    a.n=read32(g);a.start=read32(g);a.defaults.resize(read32(g));
    for(U& t:a.defaults)t=read32(g);
    assert(a.n&&a.start<a.defaults.size()&&g.peek()==EOF);
}
void jumps(Input& a) {
    a.jump.resize(a.defaults.size());
    for(U i=0;i<a.jump.size();++i){U s=i;Jump j;
        for(U k=8;k--;){U e=a.defaults[s];j.bytes[k]=Byte(e);s=e>>8;}j.next=s;a.jump[i]=j;}
}
void build_bank(const Input& a,Bank& b) {
    b.offsets.resize(a.rules.size()+1);U length=0;
    for(U i=0;i<a.rules.size();++i){b.offsets[i]=length;length+=a.rules[i].weight;}
    b.offsets.back()=length;b.text.resize(length);
    for(U i=0;i<a.rules.size();++i) {
        const auto r=a.rules[i];Byte* out=b.text.data()+b.offsets[i];
        out[0]=r.first;U s=r.post;
        for(U j=r.weight;j>1;){U e=a.defaults[s];out[--j]=Byte(e);s=e>>8;}
    }
}
void write_rule(const Input& a,Rule r,U lo,U hi,Byte* dest) {
    if(lo==hi)return;
    if(!lo)dest[0]=r.first;
    U bottom=std::max(1u,lo);if(bottom>=hi)return;
    U s=r.post,discard=r.weight-hi;
    while(discard>=8){s=a.jump[s].next;discard-=8;}
    while(discard--){s=a.defaults[s]>>8;}
    U j=hi;
    while(j-bottom>=8){const auto& w=a.jump[s];j-=8;std::memcpy(dest+j-lo,w.bytes.data(),8);s=w.next;}
    while(j>bottom){U e=a.defaults[s];dest[--j-lo]=Byte(e);s=e>>8;}
}
struct Cursor {U id,pos;};
template<class P,class Id>struct OrderScratch {
    std::vector<P> perm;std::vector<Id> cycle;
    Q capacity()const{return bytes(perm)+bytes(cycle);}
};
struct Workspace {
    std::vector<Cursor> cursor;
    OrderScratch<U,uint16_t> small;OrderScratch<Q,U> wide;
    Bank bank;std::vector<Byte> out;
    Q capacity()const{return bytes(cursor)+small.capacity()+wide.capacity()+bank.capacity()+bytes(out);}
};
struct Stamp {U block,arm,visits,literal;Q once=0,scatter=0,order=0,prepare=0,emit=0,inputs=0,scratch=0;};
template<class P,class Id>void invert(const Input& a,Workspace& w,OrderScratch<P,Id>& q,U arm,Stamp& t) {
    constexpr U shift=sizeof(P)*4;constexpr P mask=(P(1)<<shift)-1;
    Q begin=cpu();
    q.perm.resize(a.pop);w.cursor.resize(a.pop);
    for(U i=0;i<a.rules.size();++i){U base=a.rules[i].base;w.cursor[base]={i,base};}
    U destination=0;
    for(auto [base,width]:a.incoming) {
        auto& c=w.cursor[base];
        std::iota(q.perm.begin()+c.pos,q.perm.begin()+c.pos+width,(P(c.id)<<shift)|destination);
        c.pos+=width;destination+=width;
    }
    Q sorted=cpu();
    q.cycle.clear();q.cycle.reserve(a.pop);
    if(a.origin!=none){
        U x=a.origin;do {P e=q.perm[x];q.cycle.push_back(Id(e>>shift));x=U(e&mask);}while(x!=a.origin);
    }
    Q ordered=cpu();
    if(arm==1)build_bank(a,w.bank);
    const Bank& bank=arm==1?w.bank:a.bank;
    Q prepared=cpu();w.out.resize(a.n);auto out=w.out.data();
    U pos=0;
    if(a.origin==none) {
        assert(a.empty_lap&&a.empty_lap<=a.n);U s=a.start;
        for(U j=a.empty_lap;j--;){U e=a.defaults[s];out[j]=Byte(e);s=e>>8;}pos=a.empty_lap;
    }else {
        U first=q.cycle.front();auto rule=a.rules[first];assert(a.skip<=rule.weight);
        if(arm<2) {
            U part=rule.weight-a.skip;
            if(part)std::memcpy(out,bank.text.data()+bank.offsets[first]+a.skip,part);
            pos=part;
            for(size_t j=1;j<q.cycle.size();++j){
                U id=q.cycle[j],lo=bank.offsets[id],len=bank.offsets[id+1]-lo;
                std::memcpy(out+pos,bank.text.data()+lo,len);pos+=len;
            }
            if(a.skip)std::memcpy(out+pos,bank.text.data()+bank.offsets[first],a.skip);
        }else{
            write_rule(a,rule,a.skip,rule.weight,out);pos=rule.weight-a.skip;
            for(size_t j=1;j<q.cycle.size();++j){
                auto r=a.rules[q.cycle[j]];write_rule(a,r,0,r.weight,out+pos);pos+=r.weight;
            }
            write_rule(a,rule,0,a.skip,out+pos);
        }
        pos+=a.skip;
    }
    assert(pos&&pos<=a.n);
    for(U filled=pos;filled<a.n;){U take=std::min(filled,a.n-filled);std::memcpy(out+filled,out,take);filled+=take;}
    Q done=cpu();
    t.visits=q.cycle.size();t.literal=pos;t.scatter=sorted-begin;t.order=ordered-sorted;
    t.prepare=prepared-ordered;t.emit=done-prepared;t.inputs=a.capacity();t.scratch=w.capacity();
}
void compute(const Input& a,Workspace& w,U arm,Stamp& t) {
    if(a.pop<=65536&&a.rules.size()<=65536)invert(a,w,w.small,arm,t);
    else invert(a,w,w.wide,arm,t);
}
struct State {std::filesystem::path dir;std::map<U,Input> tables;Workspace scratch;std::vector<Stamp> obs;};
extern "C" State* grammar_new(){auto p=std::getenv("ORD_INPUTS");assert(p);return new State{p,{},{},{}};}
extern "C" void grammar_delete(State* s){
    if(auto p=std::getenv("ORD_STAMPS");p&&!s->obs.empty()){
        auto file=std::string(p)+"-"+std::to_string(s->obs.front().arm)+".csv";std::ofstream f(file);
        f<<"block,arm,visits,marked_lap,setup_ns,scatter_ns,order_ns,prepare_ns,emit_ns,side_bytes,scratch_bytes\n";
        for(auto t:s->obs)f<<t.block<<','<<t.arm<<','<<t.visits<<','<<t.literal<<','<<t.once<<','<<t.scatter<<','<<t.order<<','<<t.prepare<<','<<t.emit<<','<<t.inputs<<','<<t.scratch<<'\n';
    }
    delete s;
}
extern "C" const Byte* grammar_run(State* s,const uint16_t* caps,U count,U n,U mark,U arm){
    assert(caps&&arm<3);U id=lookup(n,mark,count);auto [it,created]=s->tables.try_emplace(id);
    auto& a=it->second;Stamp t{id,arm,0,0};
    if(created){Q begin=cpu();read_input(a,s->dir,id);assert(n==a.n);
        if(arm==0){
            build_bank(a,a.bank);
            if(a.origin!=none)std::vector<U>().swap(a.defaults);
        }else if(arm==2)jumps(a);
        t.once=cpu()-begin;
    }
    compute(a,s->scratch,arm,t);s->obs.push_back(t);return s->scratch.out.data();
}
