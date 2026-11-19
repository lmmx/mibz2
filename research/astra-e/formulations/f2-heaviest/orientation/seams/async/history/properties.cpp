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
template<bool Reclaim>
void check_tape(const Edges& edges,const Section& section,const Constructor& tree,U mark,const Word& gold) {
    History<64,true,Reclaim> h(edges,section);h.run();
    assert(seams(h,gold.size())==seams(tree,gold.size()));
    assert(h.stats.states==tree.stats.states && h.stats.pops==tree.stats.pops &&
           h.stats.repush==tree.stats.repush && h.stats.shift==tree.stats.shift);
    auto output=h.marked(mark,gold.size());
    assert(std::equal(output.begin(),output.end(),gold.begin(),gold.end()));
}

void tapes(const char* raw,const char* output,const char* ordinal){
    std::ifstream in(raw);U rows,mark,r,sections;in>>rows>>mark>>r>>sections;
    std::vector<Run> runs(r);for(auto& x:runs)in>>x.byte>>x.len;
    auto edges=build(runs);
    std::ifstream gf(output,std::ios::binary);Word gold((std::istreambuf_iterator<char>(gf)),{});
    assert(gold.size()==rows);
    for(U i=0;i<sections;++i){
        std::string name;U k;in>>name>>k;Section s(k);U endpoints=0;
        for(auto& [lo,hi]:s){in>>lo>>hi;endpoints+=hi-lo;}
        Constructor tree(edges);tree.run(s);
        check_tape<false>(edges,s,tree,mark,gold);
        check_tape<true>(edges,s,tree,mark,gold);
        std::cout<<ordinal<<','<<name<<','<<rows<<','<<endpoints<<','<<tree.stats.states<<','
                 <<tree.stats.pops<<','<<tree.stats.repush<<','<<tree.stats.shift<<'\n';
    }
}
int main(int argc,char** argv){
    if(argc==4){tapes(argv[1],argv[2],argv[3]);return 0;}
    assert(argc==1);

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
