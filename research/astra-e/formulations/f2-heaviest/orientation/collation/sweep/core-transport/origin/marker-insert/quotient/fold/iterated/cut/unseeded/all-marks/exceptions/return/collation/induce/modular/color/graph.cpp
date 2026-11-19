#include "color.hpp"
#include <fstream>
#include <iostream>
#include <string>
using namespace lf_color;
int main(int argc,char**argv){
    assert(argc==5);std::ifstream f(argv[1],std::ios::binary);assert(f);
    std::vector<induction::Run> runs;std::array<U,256> count{};U n=0;
    unsigned char raw[2];
    while(f.read(reinterpret_cast<char*>(raw),2)){
        U w=raw[0],a=raw[1];assert(w&&w<=127);
        if(!runs.empty()&&runs.back().byte==a)runs.back().end+=w;
        else runs.push_back({n,n+w,0,a});
        n+=w;count[a]+=w;
    }
    assert(f.eof()&&f.gcount()==0&&n==std::stoul(argv[2]));
    std::array<U,256> ptr;U total=0;
    for(U a=0;a<256;a++){ptr[a]=total;total+=count[a];}
    for(auto& r:runs){r.base=ptr[r.byte];ptr[r.byte]+=r.end-r.begin;}
    auto g=graph(runs);auto p=choose(g),best=exact(g);U mark=std::stoul(argv[3]);
    Map greedy(g,p,mark),opt(g,best,mark);W upper=0,edge_count=0;
    for(U hi=0;hi<32;hi++)for(auto [lo,e]:g.edges[hi]){
        upper+=std::max(e.different,e.mass-e.different);edge_count++;
        std::cerr<<argv[4]<<','<<lo<<','<<hi<<','<<e.mass<<','<<e.different<<'\n';
    }
    W independent=0,bias_bound=0;
    for(auto t:g.tiles){
        if(t.j==no_level)continue;
        if(t.j==t.l){independent+=t.width;continue;}
        U lo=std::min(t.j,t.l),hi=std::max(t.j,t.l),z=t.j<t.l?t.y:t.x;
        auto diff=[&](W v){return joint(v,lo,0,hi,1)+joint(v,lo,1,hi,0);};
        W d=diff(W(z)+t.width)-diff(z); // max(d,w-d) ignores global parity reversal
        independent+=std::max(d,t.width-d);
        W bias=d>t.width-d?2*d-t.width:t.width-2*d;
        W bound=std::min<W>(t.width,W(2)<<lo);
        assert(bias<=bound);bias_bound+=bound;
    }
    auto bits=[](const Phases& a){U mask=0;for(U i=0;i<32;i++)mask|=U(a[i])<<i;return mask;};
    std::cout<<argv[4]<<','<<g.n<<','<<runs.size()<<','<<g.tiles.size()<<','
             <<std::count(g.active.begin(),g.active.end(),true)<<','<<edge_count<<','<<g.fixed<<','<<g.equal<<','
             <<g.n-g.fixed+g.equal<<','<<g.n-greedy.kept<<','<<g.n-opt.kept<<','<<upper<<','
             <<bits(p)<<','<<bits(best)<<','<<independent<<','<<bias_bound<<'\n';
}
