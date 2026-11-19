#define main check_sparse_sections
#include "../check.cpp"
#undef main
#include "modular.hpp"
void verify_modular(const Oracle& p,U mark,U q,bool panel){
    auto T=modular::induce(p.runs,mark,q);U n=p.last.size();
    std::vector<unsigned char> out(n);auto emit=modular::spell(T,out.data(),n);
    U x=mark,lap=0;do{x=p.psi[x];++lap;}while(x!=mark);assert(lap==emit.lap);
    for(U b:out){assert(b==p.f[x]);x=p.psi[x];}
    U total=0,round=0;
    for(auto c:T.layers){
        assert(c.pieces<=std::min(uint64_t(c.n),uint64_t(q)*c.old_bands));
        assert(c.final_bands<=c.kept&&c.kept< c.n);
        total+=c.strict+c.equal+c.self+c.partial;
        if(panel)std::cout<<q<<','<<round<<','<<c.n<<','<<c.kept<<','<<c.old_bands<<','<<c.pieces<<','
            <<c.strict<<','<<c.equal<<','<<c.self<<','<<c.partial<<','<<c.final_bands<<','<<lap<<','
            <<emit.visits<<','<<emit.copies<<','<<emit.aliases<<','<<T.grammar.size()<<'\n';
        round++;
    }
    assert(total<=n-1&&T.grammar.size()<=n-1);
}
int main(int argc,char** argv){
    if(argc==1){
        uint64_t cases=0,bytes=0,nondiv=0;
        for(U a:{2,3})for(U n=1,ways=a;n<=(a==2?9:6);++n,ways*=a){
            for(U word=0;word<ways;word++){
                U v=word;std::vector<U> l(n);for(auto& b:l){b=v%a;v/=a;}
                Oracle p(l);
                for(U m=0;m<n;m++){
                    U x=m,lap=0;do{x=p.psi[x];++lap;}while(x!=m);
                    for(U q:{2,3,4,5,8,16}){verify_modular(p,m,q,false);++cases;bytes+=n;nondiv+=n%lap!=0;}
                }
            }
        }
        std::cout<<"marked_radix_cases="<<cases<<" nondivisor="<<nondiv<<" verified_bytes="<<bytes<<"\n";
        return 0;
    }
    std::ifstream f(argv[1],std::ios::binary);char magic[8];f.read(magic,8);
    assert(std::string(magic,8)=="CLKMASK1");U n=u32(f),mark=u32(f);
    std::vector<U> l(n);for(auto& a:l)a=static_cast<unsigned char>(f.get());assert(f);
    Oracle p(l);for(U q:{2,4,8,16})verify_modular(p,mark,q,true);
    return 0;
}
