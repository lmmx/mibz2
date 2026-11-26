#include <algorithm>
#include <cassert>
#include <cstdint>
#include <fstream>
#include <iostream>
#include <numeric>
#include <vector>

int main(int argc,char**argv){
    assert(argc==4);
    std::ifstream cap(argv[1],std::ios::binary),oracle(argv[2],std::ios::binary);
    assert(cap&&oracle);
    std::vector<uint8_t> last;
    for(int lo,hi;(lo=cap.get())!=EOF;){
        hi=cap.get();assert(hi!=EOF&&lo>=1&&lo<=127);
        last.insert(last.end(),lo,static_cast<uint8_t>(hi));
    }
    std::vector<uint32_t> psi(last.size());
    std::iota(psi.begin(),psi.end(),0);
    std::stable_sort(psi.begin(),psi.end(),[&](auto a,auto b){return last[a]<last[b];});
    uint32_t p=std::stoul(argv[3]);assert(p<last.size());
    auto x=p;uint32_t lap=0;
    for(uint32_t t=0;t<last.size();++t){
        x=psi[x];
        assert(oracle.get()==last[x]);
        if(x==p&&!lap)lap=t+1;
    }
    assert(oracle.peek()==EOF&&lap);
    std::cout<<last.size()<<','<<lap<<'\n';
}
