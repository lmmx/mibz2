#define ASTRA_RANK_LIBRARY
#include "ranks.cpp"
struct InputRun {int byte,width;};
struct Interval {int lo,hi;};
#include "../../../forward/direct_native.hpp"
struct Owned {Byte *data;size_t len,cap;};
extern "C" Owned compact_inverse(const uint16_t*,size_t,const U*,U,U);
extern "C" void compact_release(Owned);
double now(){
    timespec t;clock_gettime(CLOCK_THREAD_CPUTIME_ID,&t);
    return t.tv_sec+1e-9*t.tv_nsec;
}
U high_water(){
    std::ifstream f("/proc/self/status");std::string word;
    while(f>>word){
        if(word=="VmHWM:"){U result;f>>result;return result;}
        std::getline(f,word);
    }
    throw std::runtime_error("VmHWM unavailable");
}
struct Cost {Stats z;N setup_peak,full_peak;double build,end;};
template<class Index>
Vec<Byte> run(const std::vector<Run>& runs,U mark,U n,const Section& s,Cost& costs){
    mem={};double t=now();
    auto e=build(runs);BasicConstructor<Index> c(e);c.index_edges();c.run(s);
    costs.build=now()-t;costs.z=c.stats;costs.setup_peak=mem.peak;t=now();
    auto out=c.indexed_marked(mark,n);costs.end=now()-t;costs.full_peak=mem.peak;
    return out;
}
template<bool Queue>
Vec<Byte> oracle_run(const Edges& given,const FreeTrace& plan,U endpoints,U p,U n,Cost& costs){
    double t=now();BasicConstructor<TreeIndex> c(given);c.replay<Queue>(plan,endpoints);
    costs.build=now()-t;costs.z=c.stats;t=now();
    auto out=c.indexed_marked(p,n);costs.end=now()-t;
    return out;
}
int main(int argc,char** argv){
    if(argc!=6 && argc!=8)throw std::runtime_error("match RUNS EXPECTED ORDINAL REPS OUTPUT [ARM SECTION]");
    std::ifstream in(argv[1]);
    U n,p,r,ns;in>>n>>p>>r>>ns;
    std::vector<Run> runs;std::vector<InputRun> direct;
    std::vector<uint16_t> caps;std::array<U,256> freq{};
    for(U i=0;i<r;++i){
        U c,w;in>>c>>w;runs.push_back({c,w});direct.push_back({int(c),int(w)});
        freq[c]+=w;
        while(w){U take=std::min(w,127u);caps.push_back(uint16_t((c<<8)+take));w-=take;}
    }
    std::ifstream gold(argv[2],std::ios::binary);
    Word expected((std::istreambuf_iterator<char>(gold)),{});
    std::ofstream csv(argv[5],std::ios::app);
#ifdef ASTRA_FREE_REPLAY
    const char* arms[]={"free_geometry","free_order","direct","compact"};
#else
    const char* arms[]={"tree","paged","direct","compact"};
#endif
    // Balanced four-treatment Williams order for each four repetitions.
    int order[4][4]={{0,1,3,2},{1,2,0,3},{2,3,1,0},{3,0,2,1}};
    for(U j=0;j<ns;++j){
        std::string name;U cnt;in>>name>>cnt;
        Section s(cnt);std::vector<Interval> ss;
        for(auto&[lo,hi]:s){in>>lo>>hi;ss.push_back({int(lo),int(hi)});}
        if(argc==8 && name!=argv[7])continue;
#ifdef ASTRA_FREE_REPLAY
        Edges given=build(runs);FreeTrace plan;U endpoints=0;
        for(auto[lo,hi]:s)endpoints+=hi-lo;
        if(argc!=8 || argv[6]==std::string(arms[0]) || argv[6]==std::string(arms[1])){
            BasicConstructor<PagedIndex,true> tracing(given);
            tracing.trace=&plan;tracing.index_edges();tracing.run(s);
        }
#endif
        for(int rep=0;rep<std::stoi(argv[4]);++rep)for(int i:order[rep%4]){
            if(argc==8 && std::string(arms[i])!=argv[6])continue;
            Cost costs{};double t=now();
            Vec<Byte> out;std::string d;Owned h{};
#ifdef ASTRA_FREE_REPLAY
            if(i==0)out=oracle_run<true>(given,plan,endpoints,p,n,costs);
            if(i==1)out=oracle_run<false>(given,plan,endpoints,p,n,costs);
#else
            if(i==0)out=run<TreeIndex>(runs,p,n,s,costs);
            if(i==1)out=run<PagedIndex>(runs,p,n,s,costs);
#endif
            if(i==2)d=direct_native::decode(n,p,direct,ss);
            if(i==3)h=compact_inverse(caps.data(),caps.size(),freq.data(),n,p);
            double elapsed=now()-t;
            bool exact=i<2 ? out.size()==n && std::equal(out.begin(),out.end(),expected.begin()):
                i==2 ? d.size()==n && std::equal(expected.begin(),expected.end(),reinterpret_cast<const Byte*>(d.data())):
                       h.len==n && std::equal(expected.begin(),expected.end(),h.data);
            if(!exact)throw std::runtime_error("marked bytes disagree");
            if(i==3)compact_release(h);
            auto z=costs.z;
            csv<<argv[3]<<','<<name<<','<<rep<<','<<arms[i]<<','<<elapsed<<','<<costs.build<<','<<costs.end<<','
               <<high_water()<<','<<costs.setup_peak<<','<<costs.full_peak<<','<<z.states<<','<<z.pops<<','<<z.hit<<','
               <<z.mask_reads<<','<<z.mask_writes<<','<<z.local_hops<<','<<z.edge_cmp<<','<<z.tree_cmp<<'\n';
        }
    }
}
