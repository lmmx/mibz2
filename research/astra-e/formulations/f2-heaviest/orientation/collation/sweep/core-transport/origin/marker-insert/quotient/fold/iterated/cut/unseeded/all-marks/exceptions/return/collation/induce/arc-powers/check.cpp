#define main check_sparse_sections
#include "../check.cpp"
#undef main
#include "arc.hpp"
void exact(std::vector<Arc> arcs,std::vector<U> bottom,U mark,std::string policy){
    modular::Exchange e;e.arcs=std::move(arcs);e.bottom=std::move(bottom);
    auto dest=arc_cut::bases(e);
    std::vector<U> label,back;
    U n=0;
    for(U i=0;i<e.arcs.size();++i){
        auto a=e.arcs[i];
        for(U t=0;t<a.width;t++){label.push_back(a.rule);back.push_back(dest[i]+t);}
        n+=a.width;
    }
    std::vector<U> front(n);for(U x=0;x<n;x++)front[back[x]]=x;
    arc_cut::Grammar g;std::vector<arc_cut::Layer> rows;
    U final=arc_cut::solve(std::move(e),mark,g,rows);
    std::vector<unsigned char> out;g.emit(final,out,n);U x=mark;
    for(U b:out){x=front[x];assert(b==label[x]);}
    U lap=0;x=mark;do{x=back[x];++lap;}while(x!=mark);assert(lap==g.len(final));
    if(!policy.empty()){
        U j=0;
        for(auto s:rows){
            std::cout<<j++<<','<<s.n<<','<<s.b<<','<<s.dead<<','<<s.powered<<','<<s.power_rows<<','
              <<s.power_arcs<<','<<s.pairs<<','<<s.cut<<','<<s.triples<<','<<s.final_arcs<<','
              <<s.radix_visits<<','<<g.nodes.size()<<','<<lap<<'\n';
        }
    }
}
int main(int argc,char** argv){
    if(argc==1){
        uint64_t traces=0,bytes=0;
        for(U n=1;n<=7;++n)for(U mask=0;mask<(1u<<(n-1));++mask){
            std::vector<Arc> a;U start=0;
            for(U x=1;x<=n;++x)if(x==n||(mask>>(x-1)&1)){a.push_back({x-start,U(a.size())%3});start=x;}
            std::vector<U> bottom(a.size());std::iota(bottom.begin(),bottom.end(),0);
            do{
                for(U mark=0;mark<n;++mark){exact(a,bottom,mark,"");++traces;bytes+=n;}
            }while(std::next_permutation(bottom.begin(),bottom.end()));
        }
        std::cout<<"arbitrary_exchange_marks="<<traces<<" checked_bytes="<<bytes<<"\n";return 0;
    }
    std::ifstream f(argv[1],std::ios::binary);char magic[8];f.read(magic,8);
    assert(std::string(magic,8)=="CLKMASK1");U n=u32(f),mark=u32(f);
    std::vector<U> l(n);for(auto& b:l)b=static_cast<unsigned char>(f.get());assert(f);Oracle p(l);
    auto e=modular::initial(p.runs);exact(std::move(e.arcs),std::move(e.bottom),mark,"real");
}
