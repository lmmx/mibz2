#define CORE_SETUP_COUNTS
#include "core.hpp"
#include <fstream>
#include <iostream>
#include <sstream>
int main(int argc,char**argv){
    assert(argc==4);int wanted=std::stoi(argv[2]),h=std::stoi(argv[3]);std::ifstream f(argv[1]);
    int i,n,p,a,b,m;
    while(f>>i>>n>>p>>a>>b>>m){
        std::vector<core::InputRun> raw(m);for(auto& r:raw)f>>r.byte>>r.width;
        if(i!=wanted)continue;
        core::Counts counts(raw);core::Stats s;core::Plan plan(raw,counts,{a,b},h,s);
        auto tag_sort=s.sort_comparisons;
        if(s.accepted){
            core::Index ix(raw,counts,plan,s);core::Object o(plan,ix,s);
            auto out=o.expand(p,n);assert(int(out.size())==n);
        }
        std::ifstream proc("/proc/self/status");std::string line;int peak=0;
        while(std::getline(proc,line))if(line.starts_with("VmHWM:")){std::istringstream in(line.substr(6));in>>peak;break;}
        assert(peak>0);
        std::cout<<i<<','<<h<<','<<s.accepted<<','<<peak<<','<<s.forward_key_comparisons<<','
                 <<tag_sort<<','<<s.sort_comparisons-tag_sort<<','<<s.states<<','<<s.scans<<'\n';return 0;
    }
    return 2;
}
