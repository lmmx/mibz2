// raw-lib.inc contains the frozen geometry treatments without executable mains.
#include "raw-lib.inc"
#include <cstdlib>
#include <array>

Root read_caps(const uint16_t* caps,U count,U n,U mark){
    Root in;in.mark=mark;in.r.reserve(count);
    std::array<U,256> freq{},ids{};
    U rows=0;
    for(U i=0;i<count;++i){
        U k=caps[i]>>8,w=caps[i]&255;
        assert(w);rows+=w;freq[k]+=w;append(in.r,k,w);
    }
    assert(rows==n&&mark<n);
    for(U k=0;k<256;++k)if(freq[k]){ids[k]=in.bytes.size();in.bytes.push_back(k);}
    for(auto& v:in.r)v.k=ids[v.k];
    return in;
}

struct PrefixStamp{U arm,n,mark,caps;uint64_t start=0,construct=0,publication=0;};
struct Journal{
    PrefixStamp current{};
    std::vector<PrefixStamp> all;
    Journal(){all.reserve(1024);}
    ~Journal(){
        if(const char* path=std::getenv("FUSED_STAMPS")){
            std::ofstream f(path);
            f<<"arm,rows,mark,cap_records,first_paid_ns,cut_only_ns\n";
            for(auto v:all)f<<v.arm<<','<<v.n<<','<<v.mark<<','<<v.caps<<','<<v.publication-v.start<<','<<v.publication-v.construct<<'\n';
        }
    }
};
static Journal journal;
template<class B> struct Timed:B{
    static const Root& first_start(const Root& in){
        if(!journal.current.construct)journal.current.construct=cpu();
        return in;
    }
    explicit Timed(const Root& in):B(first_start(in)){}
    Root apply(const Root& in,std::vector<Node>& nodes){
        auto out=B::apply(in,nodes);
        if(!journal.current.publication)journal.current.publication=cpu();
        return out;
    }
};
struct Output{std::vector<uint8_t> data;};
extern "C" Output* grammar_new(){return new Output;}
extern "C" void grammar_delete(Output* p){delete p;}
extern "C" const uint8_t* grammar_run(Output* p,const uint16_t* caps,U count,U n,U mark,U arm){
    journal.current=PrefixStamp{arm,n,mark,count,cpu(),0,0};
    auto in=read_caps(caps,count,n,mark);Stats s;
    if(!arm)p->data=decode_with<Timed<StreamCut<false>>>(in,s);
    else if(arm==1)p->data=decode_with<Timed<StreamFused<false>>>(in,s);
    else{assert(arm==2);p->data=decode_with<Timed<RawStream<false>>>(in,s);}
    assert(p->data.size()==n);
    // The first publication can be absent for a trivial self-loop. The ten
    // calibrated original blocks all require one; gate the calibration input.
    assert(journal.current.publication>=journal.current.construct&&journal.current.construct>=journal.current.start);
    journal.all.push_back(journal.current);
    return p->data.data();
}
