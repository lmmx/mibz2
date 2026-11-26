#define XOR_ROUTE_LIBRARY
#include "route.cpp"
#include <array>

struct Audit {uint64_t colors=0,publications=0,powers=0;};
Audit audit;
struct Verified: StreamCut<false> {
    Cut observed;
    explicit Verified(const Root& in):StreamCut(in),observed(in) {
        assert(done==observed.done && m==observed.m && self==observed.self);
        assert(ingress==observed.ingress && color==observed.color);
        if(!done)++audit.colors;
    }
    Root apply(const Root& in,std::vector<Node>& grammar) {
        auto expected_grammar=grammar;
        auto expected=observed.apply(in,expected_grammar);
        auto value=StreamCut::apply(in,grammar);
        assert(value.r==expected.r && value.bytes==expected.bytes && value.mark==expected.mark);
        assert(grammar.size()==expected_grammar.size());
        for(U i=0;i<grammar.size();++i) {
            auto a=grammar[i],b=expected_grammar[i];
            assert(a.n==b.n && a.l==b.l && a.r==b.r && a.kind==b.kind);
        }
        ++audit.publications;return value;
    }
};
void check(const Root& in) {
    Stats stats;auto want=direct(in);auto got=decode_with<Verified>(in,stats);
    assert(want==got);
    for(auto pass:stats.passes)audit.powers+=pass.normalized;
    for(auto arm:{"greedy","exact","gaps","streamgaps"}) {
        Stats other;assert(want==execute(in,arm,other));
    }
}
int main(int argc,char** argv) {
    assert(argc>=2);uint64_t marks=0,columns=0;
    for(U n=1,N=3;n<=7;++n,N*=3)for(U code=0;code<N;++code) {
        std::array<U,3> count{},ids{};U z=code;Root raw;
        for(U t=0;t<n;++t){U k=z%3;z/=3;append(raw.r,k,1);count[k]++;}
        for(U k=0;k<3;++k)if(count[k]){ids[k]=raw.bytes.size();raw.bytes.push_back(17+89*k);}
        for(auto& r:raw.r)r.k=ids[r.k];
        for(U p=0;p<n;++p){raw.mark=p;check(raw);++marks;}++columns;
    }
    std::cout<<"{\"exhaustive_columns\":"<<columns<<",\"arbitrary_marks\":"<<marks
             <<",\"small_colors\":"<<audit.colors<<",\"small_publications\":"<<audit.publications
             <<",\"small_normalizations\":"<<audit.powers<<",\"files\":[";
    for(int i=1;i<argc;++i) {
        auto in=load(argv[i]);audit={};check(in);
        if(i!=1)std::cout<<',';
        std::cout<<"{\"case\":\""<<std::filesystem::path(argv[i]).parent_path().filename().string()
                 <<"\",\"colors\":"<<audit.colors<<",\"publications\":"<<audit.publications
                 <<",\"normalizations\":"<<audit.powers<<'}';
    }
    std::cout<<"]}\n";
}
