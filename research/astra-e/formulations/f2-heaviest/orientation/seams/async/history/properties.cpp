#define ASTRA_HISTORY_LIBRARY
#include "history.cpp"
#include <random>

template<U P>
void compare(const Oracle& o,const Edges& e,const Section& s){
    Constructor base(e);base.run(s);
    History<P,true,true> w(e,s);w.run();
    assert(seams(base,o.f.size())==seams(w,o.f.size()));
    assert(base.stats.states==w.stats.states && base.stats.pops==w.stats.pops && base.stats.repush==w.stats.repush);
    Audit a(o,w,s);
    for(U p=0;p<o.f.size();++p){
        auto out=w.marked(p,o.f.size());auto gold=o.tape(p);assert(std::equal(out.begin(),out.end(),gold.begin(),gold.end()));
    }
}
int main(){
    std::mt19937 g(142904);U cells=0;N marks=0;
    for(U n:{15u,16u,17u,63u,64u,65u,127u,128u,129u,255u,256u,257u,511u}){
        for(U trial=0;trial<8;++trial){
            std::vector<U> p(n);std::iota(p.begin(),p.end(),0u);std::shuffle(p.begin(),p.end(),g);
            Word labels(n);for(auto& b:labels)b=g()%5;
            Oracle o(p,labels);auto e=o.arbitrary_edges();
            Section s;
            for(U i=0;i<n;++i)if(i==0 || (trial==0 || (i%64>=61 || i%64<2) || g()%29==0)){
                if(s.empty() || s.back().second!=i)s.emplace_back(i,i);
                s.back().second=i+1;
            }
            compare<16>(o,e,s);compare<64>(o,e,s);compare<256>(o,e,s);cells+=3;marks+=3*n;
        }
    }
    assert(!mem.live);
    std::cout<<cells<<" random-permutation/page/section cases, "<<marks<<" checked marks, exact seam and heap schedule versus frozen tree\n";
}
