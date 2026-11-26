#define main sparse_section_checker
#include "../../check.cpp"
#undef main
#include "packets.hpp"
void verify_packets(const Oracle& p,U mark,U q,bool panel){
    U n=p.last.size();std::vector<packets::Stats> stages;
    auto tree=packets::compute(p.runs,mark,q,stages);
    std::vector<unsigned char> output(n);auto done=modular::spell(tree,output.data(),n);
    U x=mark,lap=0;do{x=p.psi[x];++lap;}while(x!=mark);assert(lap==done.lap);
    for(U b:output){assert(b==p.f[x]);x=p.psi[x];}
    U j=0;
    for(auto c:stages){
        if(panel)std::cout<<q<<','<<j<<','<<c.n<<','<<c.kept<<','<<c.input<<','<<c.pieces<<','
          <<c.fragments<<','<<c.visits<<','<<c.joins<<','<<c.outputs<<','<<c.probes<<','
          <<c.peak_pending<<','<<c.max_depth<<','<<tree.grammar.size()<<','<<done.visits<<','<<done.aliases<<'\n';
        j++;
    }
}
int main(int argc,char** argv){
    if(argc==1){
        uint64_t cases=0,bytes=0;
        for(U a:{2,3})for(U n=1,ways=a;n<=(a==2?9:6);++n,ways*=a){
            for(U word=0;word<ways;word++){
                U v=word;std::vector<U> l(n);for(auto& b:l){b=v%a;v/=a;}
                Oracle p(l);for(U m=0;m<n;++m)for(U q:{2,3,4,5,8,16}){
                    verify_packets(p,m,q,false);cases++;bytes+=n;
                }
            }
        }
        std::cout<<"marked_radix_cases="<<cases<<" verified_bytes="<<bytes<<"\n";return 0;
    }
    std::ifstream f(argv[1],std::ios::binary);char magic[8];f.read(magic,8);
    assert(std::string(magic,8)=="CLKMASK1");U n=u32(f),mark=u32(f);
    std::vector<U> l(n);for(auto& b:l)b=static_cast<unsigned char>(f.get());assert(f);
    Oracle p(l);for(U q:{2,4,8,16})verify_packets(p,mark,q,true);
}
