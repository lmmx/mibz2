#include "induce.hpp"
#include <fstream>
#include <iostream>
#include <sstream>
using namespace shift;
int main(int argc,char** argv){
    assert(argc==5 || argc==6);int wanted=std::stoi(argv[2]),depth=std::stoi(argv[3]);bool fused=std::stoi(argv[4]);
    std::ifstream file(argv[1]);int i,n,p,a,b,m;
    while(file>>i>>n>>p>>a>>b>>m){
        std::vector<InputRun> runs(m);for(auto& x:runs)file>>x.byte>>x.width;
        if(i!=wanted)continue;
        Stats st;Counts co(runs);Plan plan(runs,co,{a,b},depth,st);assert(plan.available);
        Index ix(runs,co,plan.levels.back(),st);std::string output;
        if(argc==6){ObjectImpl<true> obj(plan,ix,st,fused);output=obj.expand(p,n);}
        else{Object obj(plan,ix,st,fused);output=obj.expand(p,n);}
        assert(int(output.size())==n);
        std::ifstream proc("/proc/self/status");std::string line;int high_water=0;
        while(std::getline(proc,line))if(line.starts_with("VmHWM:")){
            std::istringstream values(line.substr(6));values>>high_water;break;
        }
        assert(high_water>0);
        std::cout<<i<<','<<depth<<','<<fused<<','<<high_water<<','<<output.size()<<'\n';return 0;
    }
    return 2;
}
