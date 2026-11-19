// Keep both published type namespaces unmodified.
#include <bits/stdc++.h>
#include <sys/resource.h>
#include <time.h>
#define EDGE_UNION_LIBRARY
#include "union.cpp"
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wreturn-type" // The included command-line main is renamed and never called.
namespace stock {
#define main prior_xor_route_main
#include "../formulations/f2-heaviest/orientation/collation/sweep/core-transport/origin/marker-insert/quotient/fold/iterated/cut/powers/xor/route.cpp"
#undef main
}
#pragma GCC diagnostic pop
void render(u id,const std::vector<stock::Node>& g,const Weights& pool,std::vector<uint8_t>& out){
    const auto& v=g[id];auto start=out.size();
    if(v.kind==stock::Node::Literal){const auto& s=pool.value[v.l];out.insert(out.end(),s.begin(),s.end());}
    else if(v.kind==stock::Node::Join){render(v.l,g,pool,out);render(v.r,g,pool,out);}
    else for(u i=0;i<v.r;i++)render(v.l,g,pool,out);
    assert(out.size()-start==v.n);
}
void test_cut(const Block& b,const Obj& o,const std::vector<Span>& published,u mark,const char* model){
    stock::Root raw;raw.mark=mark;
    for(auto r:published)raw.r.push_back({r.label,r.width});
    std::vector<stock::Node> grammar;std::unordered_map<u,u> leaves;
    for(u word:o.label){
        auto [it,inserted]=leaves.try_emplace(word,grammar.size());
        if(inserted)grammar.push_back({(u)b.words.value[word].size(),word,0,stock::Node::Literal});
        raw.bytes.push_back(it->second);
    }
    auto in=stock::seed(raw);
    unsigned depth=0;
    for(;;){
        stock::ParityCut<false,true> cut(in);if(cut.done)break;
        u m=cut.m,k=in.bytes.size(),r=in.r.size(),j=cut.parts.size(),e=cut.self,pre=cut.ingress,prior=grammar.size();
        u normalized=0,nq=0,nr=0,nk=0,splits=0;
        if(4*cut.ingress<cut.m){
            auto v=stock::interval_seeded(in);nq=v.queries;nr=v.r.size();nk=v.words.size();
            stock::Root power;power.mark=v.mark;power.r=std::move(v.r);
            std::map<std::pair<u,u>,u> nodes;
            for(auto sym:v.words){
                u w=sym.base;
                if(sym.n>1){auto [it,fresh]=nodes.try_emplace({w,sym.n},grammar.size());
                    if(fresh)grammar.push_back({grammar[w].n*sym.n,w,sym.n,stock::Node::Power});
                    w=it->second;
                }
                power.bytes.push_back(w);
            }
            in=std::move(power);cut=stock::ParityCut<false,true>(in);normalized=1;splits=cut.parts.size();
            assert(!cut.done&&cut.self==0);
        }
        assert(4*cut.ingress>=cut.m);u repeated=grammar.size()-prior;
        in=cut.apply(in,grammar);
        std::cout<<b.index<<','<<model<<','<<depth++<<','<<m<<','<<r<<','<<k<<','<<j<<','<<e<<','<<pre<<','<<normalized<<','
          <<nq<<','<<nr<<','<<nk<<','<<splits<<','<<cut.ingress<<','<<cut.m-cut.ingress<<','<<in.r.size()<<','<<in.bytes.size()<<','
          <<repeated<<','<<grammar.size()-prior-repeated<<'\n';
    }
    std::vector<uint8_t> lap;render(in.bytes[in.mark],grammar,b.words,lap);assert(!lap.empty()&&lap.size()<=b.n);
    for(u i=0;i<b.n;i++)assert(lap[i%lap.size()]==b.reference[i]);
}
int main(int argc,char** argv){
    assert(argc==5);std::ifstream edge(argv[1],std::ios::binary),word(argv[2],std::ios::binary),thin(argv[3],std::ios::binary),mask(argv[4],std::ios::binary);
    Outputs sink;std::ofstream null;u total=0;
    std::cout<<"block,model,depth,old_rows,old_runs,old_classes,certificate_fragments,self_mass,initial_ingress,power,normalization_queries,power_runs,power_classes,recertified_fragments,final_ingress,retained_rows,new_runs,new_classes,power_nodes,new_join_nodes\n";
    while(true){Block b;if(!next_block(b,edge,word,thin))break;
        V words(b.m);for(u i=0;i<b.m;i++)words[i]=b.spelled(i);
        auto o=evaluate(sink,b,"physical",words,b.next,words,true);
        std::vector<Span> r;bwt_from_live_geometry(b,o,true,&r);test_cut(b,o,r,b.mark,"physical");
        analyze_union(b,mask,null,sink,test_cut);total++;
    }
    assert(total==233);std::cerr<<total<<" physical and "<<total<<" positive-union complete terminal grammars matched every marked byte.\n";
}
