#define ASTRA_HISTORY_LIBRARY
#include "history.cpp"
#include <sys/resource.h>
#include <sstream>
struct InputRun {int byte,width;};
struct Interval {int lo,hi;};
#include "../../../forward/direct_native.hpp"

using Clock=std::chrono::steady_clock;
using Prob=std::chrono::duration<double>;
struct Readout {
    double construct=0,emit=0;
    N peak=0,with_output=0;
    N archive=0,headers=0,descriptors=0,lookups=0,examined=0,root_cases=0;
};
template<U P,bool Packed>
Word run_history(const std::vector<Run>& runs,const Section& s,U mark,U n,Readout& r) {
    auto start=std::clock();auto e=build(runs);History<P,Packed> h(e,s);h.run();r.peak=mem.peak;
    r.archive=h.hs.owner_bytes;r.headers=h.hs.page_heads_bytes;r.descriptors=h.hs.descriptors;
    r.lookups=h.hs.hist_seeks;r.examined=h.hs.desc_examined;r.root_cases=h.hs.root_hits;
    auto built=std::clock();auto out=h.marked(mark,n);auto end=std::clock();
    r.with_output=mem.peak;r.construct=double(built-start)/CLOCKS_PER_SEC;r.emit=double(end-built)/CLOCKS_PER_SEC;
    return Word(out.begin(),out.end());
}
Word run_tree(const std::vector<Run>& runs,const Section& s,U mark,U n,Readout& r) {
    auto start=std::clock();auto e=build(runs);Constructor x(e);x.run(s);r.peak=mem.peak;
    auto built=std::clock();auto out=x.marked(mark,n);auto end=std::clock();
    r.with_output=mem.peak;r.construct=double(built-start)/CLOCKS_PER_SEC;r.emit=double(end-built)/CLOCKS_PER_SEC;
    return Word(out.begin(),out.end());
}

U process_peak() {
    std::ifstream status("/proc/self/status");std::string line;
    while(std::getline(status,line)) if(line.starts_with("VmHWM:")){
        std::istringstream fields(line.substr(6));U peak=0;fields>>peak;return peak;
    }
    throw std::runtime_error("process peak unknown");
}
int main(int argc,char** argv) {
    assert(argc>=5 && argc<=7);
    std::ifstream in(argv[1]);U n,mark,r,secs;in>>n>>mark>>r>>secs;
    std::vector<Run> runs(r);std::vector<InputRun> direct(r);
    for(U i=0;i<r;++i){in>>runs[i].byte>>runs[i].len;direct[i]={int(runs[i].byte),int(runs[i].len)};}
    std::ifstream gf(argv[2],std::ios::binary);Word gold((std::istreambuf_iterator<char>(gf)),{});
    int rounds=std::stoi(argv[4]);
    const std::string chosen=argc>=6?argv[5]:"";
    for(U i=0;i<secs;++i){
        std::string name;U ns;in>>name>>ns;Section s(ns);std::vector<Interval> ds(ns);
        for(U j=0;j<ns;++j){in>>s[j].first>>s[j].second;ds[j]={int(s[j].first),int(s[j].second)};}
        for(int round=chosen.empty()?-1:0;round<rounds;++round){
            for(int pos=0;pos<4;++pos) {
                int arm=(pos+round+1)%4;
                if(round>=0 && round/4%2)arm=3-arm;
                const char* arms[]={"tree","history64","packed64","direct"};
                if(!chosen.empty() && chosen!=arms[arm])continue;
                assert(mem.live==0);mem={};Readout read;Word out;
                auto start=Clock::now();auto cpu=std::clock();
                if(arm==0)out=run_tree(runs,s,mark,n,read);
                if(arm==1)out=run_history<64,false>(runs,s,mark,n,read);
                if(arm==2)out=run_history<64,true>(runs,s,mark,n,read);
                if(arm==3) {
                    direct_native::Stats stats;
                    auto text=direct_native::decode(n,mark,direct,ds,&stats);
                    out.assign(text.begin(),text.end());
                    read.construct=(stats.setup_ms+stats.discover_ms+stats.order_ms+stats.dict_ms)/1000.;
                    read.emit=stats.emit_ms/1000.;
                }
                double elapsed=Prob(Clock::now()-start).count(),process=double(std::clock()-cpu)/CLOCKS_PER_SEC;
                assert(out==gold);assert(mem.live==0);
                rusage ru{};getrusage(RUSAGE_SELF,&ru);
                if(round>=0)
                    std::cout<<argv[3]<<','<<name<<','<<round<<','<<pos<<','<<arms[arm]<<','<<process<<','<<elapsed
                      <<','<<read.construct<<','<<read.emit<<','<<read.peak<<','<<read.with_output
                      <<','<<read.archive<<','<<read.headers<<','<<read.descriptors<<','<<read.lookups
                      <<','<<read.examined<<','<<(argc==7 ? long(process_peak()) : ru.ru_maxrss)<<'\n';
            }
        }
    }
}
