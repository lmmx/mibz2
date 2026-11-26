#define OBSERVED_PANEL_LIBRARY
#include "observed.cpp"
#include <sstream>
int main(int argc,char**argv){
    assert(argc==5);std::ifstream f(argv[1]);int wanted=std::stoi(argv[2]),h=std::stoi(argv[3]),layout=std::stoi(argv[4]);
    int id,n,p,a,b,m;
    while(f>>id>>n>>p>>a>>b>>m){
        std::vector<InputRun> raw(m);for(auto& x:raw)f>>x.byte>>x.width;
        if(id!=wanted)continue;
        Handle* subject=core_handle();Paid r;
        if(layout==2){
            std::vector<uint16_t> caps;std::array<uint32_t,256> freq{};
            for(auto x:raw){freq[x.byte]+=x.width;for(int w=x.width;w>0;w-=127)caps.push_back(x.byte*256+std::min(w,127));}
            std::array<uint32_t,257> c{};std::partial_sum(freq.begin(),freq.end(),c.begin()+1);
            int symbol=std::upper_bound(c.begin(),c.end(),a)-c.begin()-1;
            r.out=core_compact(subject,caps.data(),caps.size(),freq.data(),n,p,a,b,symbol);
        }else if(layout==0)r=construct<false>(subject,raw,{a,b},h,p,nullptr);
        else r=construct<true>(subject,raw,{a,b},h,p,nullptr);
        assert(r.out.len==unsigned(n));
        std::ifstream proc("/proc/self/status");std::string line;int high=0;
        while(std::getline(proc,line))if(line.starts_with("VmHWM:")){std::istringstream fields(line.substr(6));fields>>high;break;}
        assert(high>0);
        std::cout<<id<<','<<h<<','<<layout<<','<<high<<','<<r.out.capacity<<','<<r.s.gap_capacity_bytes<<'\n';
        core_drop(subject);return 0;
    }
    return 2;
}
