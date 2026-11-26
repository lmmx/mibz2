// Current coordinate is original physical rank compressed on each retained arithmetic progression.
#include <algorithm>
#include <array>
#include <bit>
#include <cassert>
#include <cstdint>
#include <fstream>
#include <iostream>
#include <numeric>
#include <vector>
using U=uint32_t;
std::vector<U> original(const char* cap){
    std::ifstream f(cap,std::ios::binary);std::vector<uint8_t> last;
    for(int a,b;(a=f.get())!=EOF;){b=f.get();assert(a&&a<=127&&b!=EOF);last.insert(last.end(),a,uint8_t(b));}
    std::vector<U> psi(last.size()),lf(last.size());std::iota(psi.begin(),psi.end(),0);
    std::stable_sort(psi.begin(),psi.end(),[&](U a,U b){return last[a]<last[b];});
    for(U i=0;i<psi.size();++i)lf[psi[i]]=i;
    return lf;
}
void study(std::vector<U> T,U mark,U q,U id){
    for(U level=0;T.size()>1;++level){
        U n=T.size(),phase=mark%q,classes=std::min(q,n),kept=1+(n-1-phase)/q;
        std::array<U,16> mask{},size{};
        for(U x=0;x<n;++x){mask[x%q]|=1u<<(T[x]%q);++size[x%q];}
        std::array<bool,16> cycle{};
        U singleton=0,single_rows=0,edges=0,mean_free=0,marked_prefix=0,period=0,other=0;
        for(U r=0;r<classes;++r){
            edges+=std::popcount(mask[r]);
            if(!std::has_single_bit(mask[r]))continue;
            ++singleton;single_rows+=size[r];
            U s=r;std::array<bool,16> seen{};
            while(std::has_single_bit(mask[s])&&!seen[s]){seen[s]=true;s=std::countr_zero(mask[s]);}
            if(s==r){
                cycle[r]=true;if(r==phase){period=std::count(seen.begin(),seen.end(),true);}
                assert(std::count(seen.begin(),seen.end(),true)==classes); // Each measured current permutation is primitive.
            }
        }
        for(U r=0;r<classes;++r)if(cycle[r]){
            mean_free+=size[r];
            if(!period)++other;
        }
        U cur=phase;
        while(std::has_single_bit(mask[cur])&&marked_prefix<classes){
            ++marked_prefix;cur=std::countr_zero(mask[cur]);if(cur==phase)break;
        }
        std::vector<U> next(kept),visits(kept);
        U traversed=0,lap=0;
        for(U x=phase;x<n;x+=q){
            U y=x,dt=0;
            do{y=T[y];++dt;}while(y%q!=phase);
            assert(y/q<kept);next[x/q]=y/q;++visits[y/q];traversed+=dt;
            if(period)assert(dt==period);
        }
        assert(traversed==n&&std::all_of(visits.begin(),visits.end(),[](U c){return c==1;}));
        for(U y=mark/q;;){y=next[y];++lap;if(y==mark/q)break;assert(lap<=kept);}
        assert(lap==kept);
        std::cout<<id<<','<<q<<','<<level<<','<<n<<','<<mark<<','<<phase<<','<<kept<<','<<classes<<','<<edges<<','<<singleton
            <<','<<single_rows<<','<<marked_prefix<<','<<period<<','<<other<<','<<mean_free<<','<<(n>=2*q)<<','<<traversed<<'\n';
        T=std::move(next);mark/=q;
    }
}
int main(int argc,char**argv){
    assert(argc==4);auto perm=original(argv[1]);
    for(U q:{2,4,8,16})study(perm,std::stoul(argv[2]),q,std::stoul(argv[3]));
}
