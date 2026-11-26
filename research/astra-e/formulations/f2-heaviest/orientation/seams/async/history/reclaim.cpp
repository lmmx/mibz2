#define ASTRA_HISTORY_LIBRARY
#include "history.cpp"
#include <sstream>
#include <sys/resource.h>

U vmhwm(){
    std::ifstream status("/proc/self/status");std::string line;
    while(std::getline(status,line)) if(line.starts_with("VmHWM:")) {
        std::istringstream fields(line.substr(6));U k=0;fields>>k;return k;
    }
    throw std::runtime_error("VmHWM unavailable");
}

template<bool Reclaim>
void real(const std::vector<Run>& runs,const Section& s,U mark,const Word& gold,const std::string& ordinal,
          const std::string& section,int lap,int pos){
    assert(!mem.live);mem={};
    auto start=std::clock();auto wall=std::chrono::steady_clock::now();auto e=build(runs);History<64,true,Reclaim> h(e,s);
    h.run();U bytes=mem.peak;auto built=std::clock();auto out=h.marked(mark,gold.size());auto end=std::clock();
    double elapsed=std::chrono::duration<double>(std::chrono::steady_clock::now()-wall).count();
    assert(std::equal(out.begin(),out.end(),gold.begin(),gold.end()));
    auto hs=h.hs;
    if(lap>=0)std::cout<<ordinal<<','<<section<<','<<lap<<','<<pos<<','<<(Reclaim?"reclaim":"packed")<<','
       <<double(end-start)/CLOCKS_PER_SEC<<','<<elapsed<<','<<double(built-start)/CLOCKS_PER_SEC<<','<<double(end-built)/CLOCKS_PER_SEC
       <<','<<bytes<<','<<mem.peak<<','<<hs.owner_bytes<<','<<hs.page_heads_bytes<<','<<hs.descriptors
       <<','<<hs.hist_seeks<<','<<hs.desc_examined<<','<<hs.gc_passes<<','<<hs.gc_examined<<','<<hs.gc_discarded<<','<<vmhwm()<<'\n';
}

int main(int argc,char** argv){
    assert(argc>=5 && argc<=6);
    std::ifstream in(argv[1]);U n,mark,r,ns;in>>n>>mark>>r>>ns;
    std::vector<Run> runs(r);for(auto& x:runs)in>>x.byte>>x.len;
    std::ifstream gf(argv[2],std::ios::binary);Word gold((std::istreambuf_iterator<char>(gf)),{});assert(gold.size()==n);
    for(U i=0;i<ns;++i){
        std::string name;U k;in>>name>>k;Section sec(k);for(auto& v:sec)in>>v.first>>v.second;
        if(argc==6){
            if(std::string(argv[5])=="reclaim") real<true>(runs,sec,mark,gold,argv[3],name,0,0);
            else real<false>(runs,sec,mark,gold,argv[3],name,0,0);
        } else for(int lap=-1;lap<std::stoi(argv[4]);++lap) for(int slot=0;slot<2;++slot){
            if(slot^(lap>=0 && (lap%4==1 || lap%4==2)))real<true>(runs,sec,mark,gold,argv[3],name,lap,slot);
            else real<false>(runs,sec,mark,gold,argv[3],name,lap,slot);
        }
    }
    assert(mem.live==0);
}
